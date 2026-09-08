//! Matching parsed names against a query, via Smith-Waterman alignment.
//!
//! The pairing exists because neither half is much use alone here. A scraped
//! release name is mostly noise:
//!
//! ```text
//! [Judas] Shingeki no Kyojin: The Last Attack (Attack on Titan: The Last
//! Attack) v2 [1080p][HEVC x265 10bit][Dual-Audio][Multi-Subs] (Movie)
//! ```
//!
//! Parsing it first leaves the two title candidates that actually name the
//! show, and matching those rather than the raw string is both faster and more
//! accurate, because the group tag and the quality flags stop competing with
//! the title for alignment.
//!
//! Names are parsed once, when the index is built, so a scraped list crosses
//! the wasm boundary a single time no matter how many queries run against it.

use frizbee::{CaseMatching, Config, Matcher, Scoring, UnicodeMatching};
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// How a query is compared against the index.
#[derive(Debug, Clone, Copy)]
pub struct SearchConfig {
    /// Characters of the query allowed to go unmatched. `None` scores every
    /// entry instead of discarding those that do not contain the query in
    /// order, which is what you want when ranking rather than filtering.
    pub max_typos: Option<u16>,
    /// Smallest score worth returning.
    pub min_score: u16,
    /// Cap on returned rows; `0` means no cap.
    pub limit: usize,
}

impl Default for SearchConfig {
    fn default() -> Self {
        // typo-free matching is roughly six times faster and loses nothing when the query really is the title
        Self { max_typos: Some(0), min_score: 1, limit: 0 }
    }
}

fn frizbee_config(max_typos: Option<u16>) -> Config {
    Config::default()
        .max_typos(max_typos)
        // release names case titles inconsistently: `Shingeki No Kyojin` beside `Shingeki no Kyojin`
        .casing(CaseMatching::Ignore)
        .unicode(UnicodeMatching::Smart)
}

/// One release that matched, scored by its best title candidate.
#[derive(Debug, Clone, Serialize)]
pub struct SearchHit {
    /// Position of the release in the list the index was built from.
    pub index: u32,
    /// Best score across every (query, title candidate) pair.
    pub score: u16,
    /// The title candidate that scored best.
    pub title: String,
    /// Whether that candidate equals the query exactly.
    pub exact: bool,
}

/// A parsed list of release names, ready to be searched repeatedly.
pub struct Index {
    /// Title candidates from every release, flattened.
    candidates: Vec<String>,
    /// `owner[i]` is the release that `candidates[i]` came from.
    owner: Vec<u32>,
    /// `parsed[i]` is the parse of the i-th release, indexed by owner rather than by candidate.
    parsed: Vec<crate::parser::ParseResult>,
    /// The same titles in the form [`title_scores`](Index::title_scores) compares, with each one's
    /// self score, since a normalised score needs both sides measured against themselves.
    keys: Keys,
}

/// Title keys, flattened the same way as `candidates`.
#[derive(Default)]
struct Keys {
    text: Vec<String>,
    owner: Vec<u32>,
    selves: Vec<u16>,
}

impl Index {
    pub fn build(names: &[String]) -> Self {
        let mut candidates = Vec::new();
        let mut owner = Vec::new();
        let mut parsed = Vec::with_capacity(names.len());

        for (i, name) in names.iter().enumerate() {
            let result = crate::parser::parse(name).unwrap_or_default();
            for title in result.titles.iter().flatten() {
                candidates.push(title.clone());
                owner.push(i as u32);
            }
            // a name that yields no title still has to be addressable, so fall back to the raw string
            if result.titles.as_ref().is_none_or(|t| t.is_empty()) {
                candidates.push(name.clone());
                owner.push(i as u32);
            }
            parsed.push(result);
        }

        let keys = build_keys(names, &parsed);
        Self { candidates, owner, parsed, keys }
    }

    pub fn len(&self) -> usize {
        self.parsed.len()
    }

    pub fn is_empty(&self) -> bool {
        self.parsed.is_empty()
    }

    pub fn parsed(&self, index: usize) -> Option<&crate::parser::ParseResult> {
        self.parsed.get(index)
    }

    /// Longest query this configuration accepts. Beyond it frizbee's score
    /// overflows `u16` and it aborts, which in wasm takes the whole module
    /// down, so queries are checked rather than passed straight through.
    ///
    /// This mirrors frizbee's own assertion rather than calling its public
    /// `Scoring::max_needle_len`, which reports 10922 for the default scoring
    /// where the assertion actually fires at 3639. The helper leaves the prefix,
    /// exact-match and mismatch terms out of its headroom; the assertion
    /// subtracts all three.
    pub fn max_query_len(max_typos: Option<u16>) -> usize {
        needle_limit(&frizbee_config(max_typos).scoring)
    }


    /// Scores every release against every query, keeping each release's best
    /// pair. Passing a show's aliases together is the point: a release named
    /// only `進撃の巨人` and one named only `Attack on Titan` both belong to the
    /// same show, and no single query finds both.
    pub fn search(&self, queries: &[String], config: SearchConfig) -> Result<Vec<SearchHit>, String> {
        let limit = Self::max_query_len(config.max_typos);
        if let Some(bad) = queries.iter().find(|q| q.chars().count() > limit) {
            return Err(format!(
                "query too long: {} chars > {limit}",
                bad.chars().count()
            ));
        }

        let mut best: Vec<Option<(u16, usize, bool)>> = vec![None; self.parsed.len()];
        let frizbee = frizbee_config(config.max_typos);

        for query in queries {
            if query.is_empty() {
                continue;
            }
            let mut matcher = Matcher::new(query.as_str(), &frizbee);
            for m in matcher.match_list(&self.candidates) {
                let candidate = m.index as usize;
                let release = self.owner[candidate] as usize;
                let slot = &mut best[release];
                if slot.is_none_or(|(score, ..)| m.score > score) {
                    *slot = Some((m.score, candidate, m.exact));
                }
            }
        }

        let mut hits: Vec<SearchHit> = best
            .into_iter()
            .enumerate()
            .filter_map(|(release, slot)| {
                let (score, candidate, exact) = slot?;
                (score >= config.min_score).then(|| SearchHit {
                    index: release as u32,
                    score,
                    title: self.candidates[candidate].clone(),
                    exact,
                })
            })
            .collect();

        hits.sort_by(|a, b| b.score.cmp(&a.score).then(a.index.cmp(&b.index)));
        if config.limit > 0 {
            hits.truncate(config.limit);
        }
        Ok(hits)
    }
}


/// Longest needle the given scoring accepts. Beyond it frizbee's score overflows `u16` and it
/// aborts, which in wasm takes the whole module down, so both the queries and the keys are checked
/// rather than passed straight through.
///
/// This mirrors frizbee's own assertion rather than calling its public `Scoring::max_needle_len`,
/// which reports 10922 for the default scoring where the assertion actually fires at 3639. The
/// helper leaves the prefix, exact-match and mismatch terms out of its headroom; the assertion
/// subtracts all three.
fn needle_limit(s: &Scoring) -> usize {
    // frizbee's `max_per_char_bonus`
    let bonus = s.delimiter_bonus.max(s.capitalization_bonus);
    let amortized = bonus.div_ceil(2).max(bonus.saturating_sub(s.gap_open_penalty));
    let per_char_bonus = amortized.saturating_add(s.matching_case_bonus);
    // frizbee's `max_one_time_bonus`
    let one_time_bonus = bonus - amortized;

    let max_per_char = s.match_score.saturating_add(per_char_bonus);
    if max_per_char == 0 {
        return usize::MAX;
    }
    let headroom = u16::MAX
        .saturating_sub(s.prefix_bonus)
        .saturating_sub(s.exact_match_bonus)
        .saturating_sub(s.mismatch_penalty)
        .saturating_sub(one_time_bonus);
    (headroom / max_per_char) as usize
}

/// A title shorter than this carries too little to be a title. `part`, which a release name leaves
/// behind whenever it writes `Season 3 Part 2`, is worth 4 and is the reason the floor exists: as a
/// candidate it is trivially contained in any alias mentioning a part, and it scored 0.895 against
/// `Dr. Stone Season 3 Part 2` on a real Attack on Titan release.
const MIN_TITLE_WEIGHT: usize = 5;

/// How a title is scored, which is not how a query is ranked.
///
/// Two departures from [`frizbee_config`], both measured over 42,000 labelled decisions on 2,625
/// real nyaa names:
///
/// * `gap_extend_penalty` 6 rather than 1. A title contained in another aligns contiguously, while
///   an unrelated name only matches by scattering the query across it, and at the default penalty
///   scattering is nearly free: `oshi no ko` reads 0.824 against an Attack on Titan release. Raising
///   it moved the best negative from 0.861 to 0.711 and left every true positive above 0.90.
/// * no `exact_match_bonus`. The bonus is in the self score of a title and unreachable by a title
///   found inside a longer one, so it caps a real containment at 0.84 and squeezes the window every
///   threshold has to sit in.
fn title_config() -> Config {
    Config::default()
        .max_typos(None)
        .casing(CaseMatching::Ignore)
        // release names mix scripts freely, and a byte-wise match across a multi-byte character is
        // not a match: `Smart` picks the unicode path exactly when the needle needs it
        .unicode(UnicodeMatching::Smart)
        .scoring(Scoring { gap_extend_penalty: 6, exact_match_bonus: 0, ..Scoring::default() })
}

fn self_score(text: &str, config: &Config) -> u16 {
    if text.is_empty() {
        return 0;
    }
    Matcher::new(text, config).match_list(&[text]).first().map_or(0, |m| m.score)
}

fn push_key(into: &mut Vec<String>, text: String) {
    if crate::title_key::weight(&text) >= MIN_TITLE_WEIGHT && !into.contains(&text) {
        into.push(text);
    }
}

/// Every form of a name that might BE the show's title: each parsed title as a key and bare, plus
/// the whole name, which is the insurance against the parser reading the title wrongly. Dropping the
/// whole name costs five of 2,625 real releases their only match, all of them Japanese TV captures
/// whose title the parser loses inside a timestamp.
fn keys_of(name: &str, parsed: &crate::parser::ParseResult) -> Vec<String> {
    let mut out = Vec::new();
    for title in parsed.titles.iter().flatten() {
        let key = crate::title_key::key(title);
        let bare = crate::title_key::bare(&key);
        // A candidate that is ENTIRELY a season or part marker is not a title, and it matches far too
        // well: the parser leaves `Part 2` beside the real title of
        // `Shingeki no Kyojin Season 3 Part 2`, and `part 2` is contained in any alias that names a
        // part, so it scored 1.000 against `Dr. Stone Season 3 Part 2`. Nothing is left of it once the
        // markers come off, which is exactly what says it names no show.
        if bare.is_empty() {
            continue;
        }
        push_key(&mut out, bare);
        push_key(&mut out, key);
    }
    let whole = crate::title_key::key(name);
    push_key(&mut out, crate::title_key::bare(&whole));
    push_key(&mut out, whole);
    out
}

fn build_keys(names: &[String], parsed: &[crate::parser::ParseResult]) -> Keys {
    let config = title_config();
    let limit = needle_limit(&config.scoring);
    let mut keys = Keys::default();
    for (i, name) in names.iter().enumerate() {
        let empty = crate::parser::ParseResult::default();
        for key in keys_of(name, parsed.get(i).unwrap_or(&empty)) {
            if key.chars().count() > limit {
                continue;
            }
            keys.selves.push(self_score(&key, &config));
            keys.text.push(key);
            keys.owner.push(i as u32);
        }
    }
    keys
}

/// One release, scored against a show's aliases.
#[derive(Debug, Clone, Serialize)]
pub struct TitleMatch {
    /// Position of the release in the list the index was built from.
    pub index: u32,
    /// 0 to 1: how much of the shorter of the two titles the longer one contains.
    pub score: f32,
    /// The title of the release that scored best, in the form it was compared in.
    pub title: String,
    /// The alias that scored best, in the form it was compared in.
    pub alias: String,
}

impl Index {
    /// How well each release names this show, one score per release, in index order.
    ///
    /// Near 1 when the release names the show and well under 0.7 when it names a different one.
    /// Measured over 56,380 labelled decisions on 4,063 real nyaa names, in two corpora of different
    /// shows: every threshold from 0.72 to 0.90 was free of both false positives and false negatives
    /// on the first and 0.69 to 0.83 on the second, so 0.78 is the middle of what both allow.
    ///
    /// Both sides are normalised the same way and both are measured against themselves, so the
    /// number really is a ratio: the score of the best pair over the score that pair's SHORTER side
    /// gets against itself. The shorter side is the right normaliser because it asks the question
    /// that matters, is the smaller of these two titles contained in the larger one. Dividing by the
    /// alias alone instead rejects a release whose own title is shorter than the alias, which is most
    /// of them, since a catalogue spells a title `Mushoku Tensei: Jobless Reincarnation Season 2
    /// (2024)` where a release says `Mushoku Tensei`: 149 of 2,625 real releases were lost that way.
    ///
    /// Pass every alias there is, in every script. A release named only `進撃の巨人` scores near 1
    /// against the Japanese title and near 0 against the romaji one, so an alias list missing it does
    /// not merely rank such a release low, it drops it.
    ///
    /// A SIBLING SEASON scores near 1 and is supposed to: `Dr. Stone - Science Future - 05` really
    /// does name the show that `Dr. Stone Season 3 Part 2` belongs to. Separating seasons is the
    /// season and airing window's job, and this never substitutes for them.
    pub fn title_scores(&self, aliases: &[String]) -> Vec<f32> {
        let mut best = vec![0f32; self.parsed.len()];
        self.walk_titles(aliases, |release, score, _, _| {
            if score > best[release] {
                best[release] = score;
            }
        });
        best
    }

    /// Like [`title_scores`](Index::title_scores), but keeps the pair behind each score and returns
    /// only the releases at or above `min_score`, best first.
    pub fn title_matches(&self, aliases: &[String], min_score: f32) -> Vec<TitleMatch> {
        let mut best: Vec<Option<(f32, usize, usize)>> = vec![None; self.parsed.len()];
        self.walk_titles(aliases, |release, score, query, key| {
            let slot = &mut best[release];
            if slot.is_none_or(|(current, ..)| score > current) {
                *slot = Some((score, query, key));
            }
        });

        let queries = self.query_keys(aliases);
        let mut hits: Vec<TitleMatch> = best
            .into_iter()
            .enumerate()
            .filter_map(|(release, slot)| {
                let (score, query, key) = slot?;
                (score >= min_score).then(|| TitleMatch {
                    index: release as u32,
                    score,
                    title: self.keys.text[key].clone(),
                    alias: queries[query].clone(),
                })
            })
            .collect();
        hits.sort_by(|a, b| b.score.total_cmp(&a.score).then(a.index.cmp(&b.index)));
        hits
    }

    /// The aliases in the form the keys are in. An alias gets the same two readings a name does, so
    /// `Dr. Stone Season 3 Part 2` reaches a release that says `Dr. Stone - New World`.
    fn query_keys(&self, aliases: &[String]) -> Vec<String> {
        let limit = needle_limit(&title_config().scoring);
        let mut out = Vec::new();
        for alias in aliases {
            let key = crate::title_key::key(alias);
            push_key(&mut out, crate::title_key::bare(&key));
            push_key(&mut out, key);
        }
        out.retain(|q| q.chars().count() <= limit);
        out
    }

    /// Scores every (alias, key) pair in both directions and hands each result to `take`.
    ///
    /// Both directions, because Smith-Waterman is not symmetric: the needle is what has to be
    /// accounted for, so a long alias against a short title has to open gaps that the short title
    /// against the long alias never opens. Running the pair only one way costs 29 of 2,625 real
    /// releases their match one way round and 497 the other.
    fn walk_titles(&self, aliases: &[String], mut take: impl FnMut(usize, f32, usize, usize)) {
        let config = title_config();
        let queries = self.query_keys(aliases);
        if queries.is_empty() || self.keys.text.is_empty() {
            return;
        }
        let selves: Vec<u16> = queries.iter().map(|q| self_score(q, &config)).collect();

        for (qi, query) in queries.iter().enumerate() {
            if selves[qi] == 0 {
                continue;
            }
            let mut matcher = Matcher::new(query.as_str(), &config);
            for m in matcher.match_list(&self.keys.text) {
                let ki = m.index as usize;
                let denominator = selves[qi].min(self.keys.selves[ki].max(1));
                let ratio = (m.score as f32 / denominator as f32).min(1.0);
                take(self.keys.owner[ki] as usize, ratio, qi, ki);
            }
        }
        // only where the key could be the shorter side: where it is the longer one the forward pass
        // has already aligned the shorter needle, and building a matcher per key is the whole cost
        let longest = queries.iter().map(|q| crate::title_key::weight(q)).max().unwrap_or(0);
        for (ki, key) in self.keys.text.iter().enumerate() {
            if self.keys.selves[ki] == 0 || crate::title_key::weight(key) > longest {
                continue;
            }
            let mut matcher = Matcher::new(key.as_str(), &config);
            for m in matcher.match_list(&queries) {
                let qi = m.index as usize;
                if selves[qi] == 0 {
                    continue;
                }
                let denominator = selves[qi].min(self.keys.selves[ki]);
                let ratio = (m.score as f32 / denominator as f32).min(1.0);
                take(self.keys.owner[ki] as usize, ratio, qi, ki);
            }
        }
    }
}

#[wasm_bindgen(typescript_custom_section)]
const TS_SEARCH: &'static str = r#"
export interface SearchOptions {
  /** Query characters allowed to go unmatched. `null` scores every entry
   *  instead of dropping those that do not contain the query in order.
   *  Defaults to 0. */
  maxTypos?: number | null;
  /** Smallest score worth returning. Defaults to 1. */
  minScore?: number;
  /** Cap on returned rows. 0 or omitted means no cap. */
  limit?: number;
}

export interface TitleMatch {
  /** Position in the list the index was built from. */
  index: number;
  /** 0 to 1: how much of the shorter of the two titles the longer one contains. */
  score: number;
  /** The release title that scored best, in the form it was compared in. */
  title: string;
  /** The alias that scored best, in the form it was compared in. */
  alias: string;
}

export interface SearchHit {
  /** Position in the list the index was built from. */
  index: number;
  /** Best score across every (query, title candidate) pair. */
  score: number;
  /** The title candidate that scored best. */
  title: string;
  /** Whether that candidate equals the query exactly. */
  exact: boolean;
}
"#;

/// A parsed list of release names that can be searched repeatedly.
///
/// Build it once from the names you scraped, then query it as often as you
/// like; the names are parsed on construction and stay on the wasm side.
#[wasm_bindgen(js_name = ReleaseIndex)]
pub struct JsReleaseIndex {
    inner: Index,
}

#[wasm_bindgen(js_class = ReleaseIndex)]
impl JsReleaseIndex {
    #[wasm_bindgen(constructor)]
    pub fn new(names: Vec<String>) -> JsReleaseIndex {
        JsReleaseIndex { inner: Index::build(&names) }
    }

    /// Number of releases in the index.
    #[wasm_bindgen(getter)]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// The full parse of one release, or `undefined` when out of range.
    pub fn parsed(&self, index: usize) -> Result<JsValue, JsValue> {
        match self.inner.parsed(index) {
            None => Ok(JsValue::UNDEFINED),
            Some(r) => serde_wasm_bindgen::to_value(r).map_err(|e| JsValue::from_str(&e.to_string())),
        }
    }

    /// How well each release names this show, one score per release, in index order.
    ///
    /// Near 1 when the release names the show and well under 0.7 when it names a different one.
    /// Measured over 56,380 labelled decisions on 4,063 real nyaa names, in two corpora of different
    /// shows: 0.72 to 0.90 was free of both false positives and false negatives on the first and
    /// 0.69 to 0.83 on the second, so 0.78 is the middle of what both allow.
    ///
    /// Pass every alias there is, in every script: a release named only `進撃の巨人` scores near 1
    /// against the Japanese title and near 0 against the romaji one.
    #[wasm_bindgen(js_name = titleScores)]
    pub fn title_scores(&self, aliases: Vec<String>) -> Vec<f32> {
        self.inner.title_scores(&aliases)
    }

    /// The same scores with the pair behind each one, keeping only releases at or above
    /// `minScore`, best first.
    #[wasm_bindgen(js_name = titleMatches, unchecked_return_type = "TitleMatch[]")]
    pub fn title_matches(&self, aliases: Vec<String>, min_score: Option<f32>) -> Result<JsValue, JsValue> {
        let hits = self.inner.title_matches(&aliases, min_score.unwrap_or(0.0));
        serde_wasm_bindgen::to_value(&hits).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Ranks the index against one or more aliases of the same show, best
    /// first. Pass every alias you have: a release named only `進撃の巨人` and
    /// one named only `Attack on Titan` are the same show, and no single query
    /// finds both.
    #[wasm_bindgen(unchecked_return_type = "SearchHit[]")]
    pub fn search(
        &self,
        queries: Vec<String>,
        #[wasm_bindgen(unchecked_param_type = "SearchOptions | null | undefined")] options: Option<
            js_sys::Object,
        >,
    ) -> Result<JsValue, JsValue> {
        let config = read_options(options.as_ref())?;
        let hits = self.inner.search(&queries, config).map_err(|e| JsValue::from_str(&e))?;
        serde_wasm_bindgen::to_value(&hits).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

/// Parses `names` and ranks them against `queries` in one call.
///
/// Equivalent to building a [`ReleaseIndex`](JsReleaseIndex) and searching it
/// once, without a handle to keep or free. Prefer it for a list you query once
/// and discard; keep the index when several queries run against one list, since
/// the parse is the whole cost and the index pays it only once.
#[wasm_bindgen(js_name = searchNames, unchecked_return_type = "SearchHit[]")]
pub fn js_search_names(
    names: Vec<String>,
    queries: Vec<String>,
    #[wasm_bindgen(unchecked_param_type = "SearchOptions | null | undefined")] options: Option<
        js_sys::Object,
    >,
) -> Result<JsValue, JsValue> {
    let config = read_options(options.as_ref())?;
    let hits = Index::build(&names)
        .search(&queries, config)
        .map_err(|e| JsValue::from_str(&e))?;
    serde_wasm_bindgen::to_value(&hits).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Parses `names` and scores each one against `aliases` in one call, 0 to 1 per name, in order.
///
/// Equivalent to building a [`ReleaseIndex`](JsReleaseIndex) and calling `titleScores` once.
#[wasm_bindgen(js_name = titleScores)]
pub fn js_title_scores(names: Vec<String>, aliases: Vec<String>) -> Vec<f32> {
    Index::build(&names).title_scores(&aliases)
}

/// Longest query the given options accept, in characters.
#[wasm_bindgen(js_name = maxQueryLen)]
pub fn js_max_query_len(
    #[wasm_bindgen(unchecked_param_type = "SearchOptions | null | undefined")] options: Option<
        js_sys::Object,
    >,
) -> Result<usize, JsValue> {
    Ok(Index::max_query_len(read_options(options.as_ref())?.max_typos))
}

fn get(options: Option<&js_sys::Object>, key: &str) -> Result<Option<JsValue>, JsValue> {
    let Some(object) = options else { return Ok(None) };
    let value = js_sys::Reflect::get(object, &JsValue::from_str(key))?;
    Ok((!value.is_undefined()).then_some(value))
}

fn read_options(options: Option<&js_sys::Object>) -> Result<SearchConfig, JsValue> {
    let mut config = SearchConfig::default();

    if let Some(value) = get(options, "maxTypos")? {
        config.max_typos = if value.is_null() {
            None
        } else {
            let n = value
                .as_f64()
                .ok_or_else(|| JsValue::from_str("`maxTypos` must be a number or null"))?;
            if !n.is_finite() || n < 0.0 || n > u16::MAX as f64 {
                return Err(JsValue::from_str("`maxTypos` is out of range"));
            }
            Some(n as u16)
        };
    }
    if let Some(value) = get(options, "minScore")? {
        let n = value
            .as_f64()
            .ok_or_else(|| JsValue::from_str("`minScore` must be a number"))?;
        if !n.is_finite() || n < 0.0 || n > u16::MAX as f64 {
            return Err(JsValue::from_str("`minScore` is out of range"));
        }
        config.min_score = n as u16;
    }
    if let Some(value) = get(options, "limit")? {
        let n = value
            .as_f64()
            .ok_or_else(|| JsValue::from_str("`limit` must be a number"))?;
        if !n.is_finite() || n < 0.0 {
            return Err(JsValue::from_str("`limit` is out of range"));
        }
        config.limit = n as usize;
    }
    Ok(config)
}
