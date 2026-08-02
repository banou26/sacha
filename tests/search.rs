//! Matching a query against parsed release names.

use sacha::{Index, SearchConfig};

fn names() -> Vec<String> {
    [
        "[SubsPlease] Shingeki no Kyojin Movie - The Last Attack (1080p) [40DD2396].mkv",
        "[Anime Time] Attack On Titan (Ultimate Collection) [BD] [Dual Audio][1080p] (Shingeki no Kyojin)",
        "[VCB-Studio] 进击的巨人 / Shingeki no Kyojin / 進撃の巨人 10-bit 1080p HEVC BDRip",
        "Shingeki.no.Kyojin.The.Last.Attack.Movie.2024.1080p.BDRip.x264-KuroNeko",
        "[Erai-raws] Sousou no Frieren - 01 [1080p][MultiSub]",
        "[MTBB] Kobayashi-san Chi no Maid Dragon S2 (BD 1080p)",
        "[NOOBDL] Атака Титанов (Shingeki no Kyojin Movie - The Last Attack) 1080p WEB-DL",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

fn loose() -> SearchConfig {
    SearchConfig { max_typos: None, ..Default::default() }
}

#[test]
fn ranks_the_right_releases_first() {
    let index = Index::build(&names());
    assert_eq!(index.len(), 7);

    let hits = index
        .search(&["Shingeki no Kyojin".to_string()], SearchConfig::default())
        .unwrap();
    let matched: Vec<u32> = hits.iter().map(|h| h.index).collect();
    // every Attack on Titan release, and neither of the other two shows
    assert!(matched.contains(&0) && matched.contains(&1) && matched.contains(&2));
    assert!(!matched.contains(&4), "Frieren should not match: {hits:?}");
    assert!(!matched.contains(&5), "Maid Dragon should not match: {hits:?}");
}

/// The point of taking a list of queries: no single alias finds every release.
#[test]
fn aliases_cover_releases_no_single_query_reaches() {
    let index = Index::build(&names());
    let romaji = index.search(&["Shingeki no Kyojin".to_string()], SearchConfig::default()).unwrap();
    let english = index.search(&["Attack on Titan".to_string()], SearchConfig::default()).unwrap();
    let both = index
        .search(
            &["Shingeki no Kyojin".to_string(), "Attack on Titan".to_string(), "進撃の巨人".to_string()],
            SearchConfig::default(),
        )
        .unwrap();
    assert!(both.len() >= romaji.len().max(english.len()), "{both:?}");
    // the Cyrillic release is only reachable through its bracketed romaji alias
    assert!(both.iter().any(|h| h.index == 6), "{both:?}");
}

#[test]
fn scores_every_release_when_typos_are_unbounded() {
    let index = Index::build(&names());
    let strict = index.search(&["Attack on Titan".to_string()], SearchConfig::default()).unwrap();
    let loose = index.search(&["Attack on Titan".to_string()], loose()).unwrap();
    assert!(loose.len() > strict.len(), "strict {strict:?}\nloose {loose:?}");
    assert_eq!(loose.len(), index.len());
    // still ranked, not just returned
    assert!(loose[0].score > loose[loose.len() - 1].score);
}

#[test]
fn hits_are_ordered_and_capped() {
    let index = Index::build(&names());
    let hits = index
        .search(&["Shingeki no Kyojin".to_string()], SearchConfig { limit: 2, ..loose() })
        .unwrap();
    assert_eq!(hits.len(), 2);
    assert!(hits[0].score >= hits[1].score);
}

#[test]
fn reports_the_matching_title_not_the_raw_name() {
    let index = Index::build(&names());
    let hits = index
        .search(&["Shingeki no Kyojin".to_string()], SearchConfig::default())
        .unwrap();
    let scene = hits.iter().find(|h| h.index == 3).expect("scene release should match");
    // the dotted name came out spaced, and without its metadata tail
    assert!(scene.title.starts_with("Shingeki no Kyojin"), "{:?}", scene.title);
    assert!(!scene.title.contains("BDRip"), "{:?}", scene.title);
}

/// frizbee aborts on a query longer than its score can represent, which in wasm
/// takes the module with it, so the length is checked instead.
#[test]
fn rejects_an_overlong_query_instead_of_aborting() {
    let index = Index::build(&names());
    let limit = Index::max_query_len(Some(0));
    // frizbee's own helper claims 10922 here; its assertion fires at 3639
    assert_eq!(limit, 3639, "guard must mirror the assertion, not the helper");
    assert!(index.search(&["x".repeat(limit)], SearchConfig::default()).is_ok());
    let err = index.search(&["x".repeat(limit + 1)], SearchConfig::default()).unwrap_err();
    assert!(err.contains("too long"), "{err}");
}

#[test]
fn handles_an_empty_index_and_empty_queries() {
    let empty = Index::build(&[]);
    assert!(empty.is_empty());
    assert_eq!(empty.search(&["anything".to_string()], SearchConfig::default()).unwrap().len(), 0);

    let index = Index::build(&names());
    assert_eq!(index.search(&[], SearchConfig::default()).unwrap().len(), 0);
    assert_eq!(index.search(&["".to_string()], SearchConfig::default()).unwrap().len(), 0);
}

#[test]
fn exposes_the_parse_behind_a_hit() {
    let index = Index::build(&names());
    let parsed = index.parsed(3).expect("in range");
    assert!(parsed.video_codec_terms.is_some());
    assert!(index.parsed(99).is_none());
}
