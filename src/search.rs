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

use frizbee::{CaseMatching, Config, Matcher, UnicodeMatching};
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
        // Typo-free matching is roughly six times faster and loses nothing when
        // the query really is the title. Callers ranking loosely set None.
        Self { max_typos: Some(0), min_score: 1, limit: 0 }
    }
}

fn frizbee_config(max_typos: Option<u16>) -> Config {
    Config::default()
        .max_typos(max_typos)
        // Release names case titles inconsistently: `Shingeki No Kyojin` beside
        // `Shingeki no Kyojin`. Ignoring case is worth several percent of recall
        // and costs nothing here.
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
    parsed: Vec<crate::parser::ParseResult>,
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
            // A name that yields no title still has to be addressable, so fall
            // back to the raw string rather than dropping it from the index.
            if result.titles.as_ref().is_none_or(|t| t.is_empty()) {
                candidates.push(name.clone());
                owner.push(i as u32);
            }
            parsed.push(result);
        }

        Self { candidates, owner, parsed }
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
        let s = frizbee_config(max_typos).scoring;

        // frizbee's `max_per_char_bonus`, from the public fields
        let bonus = s.delimiter_bonus.max(s.capitalization_bonus);
        let amortized = bonus.div_ceil(2).max(bonus.saturating_sub(s.gap_open_penalty));
        let per_char_bonus = amortized.saturating_add(s.matching_case_bonus);
        // ...and its `max_one_time_bonus`
        let one_time_bonus = bonus - amortized;

        let max_per_char = s.match_score.saturating_add(per_char_bonus);
        if max_per_char == 0 {
            // a zero per-char score cannot overflow at any length
            return usize::MAX;
        }
        let headroom = u16::MAX
            .saturating_sub(s.prefix_bonus)
            .saturating_sub(s.exact_match_bonus)
            .saturating_sub(s.mismatch_penalty)
            .saturating_sub(one_time_bonus);
        (headroom / max_per_char) as usize
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

        // best[release] = (score, candidate index)
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

        // descending score, then input order, so ties stay reproducible
        hits.sort_by(|a, b| b.score.cmp(&a.score).then(a.index.cmp(&b.index)));
        if config.limit > 0 {
            hits.truncate(config.limit);
        }
        Ok(hits)
    }
}

// ---------------------------------------------------------------------------
// JS bindings
// ---------------------------------------------------------------------------

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
