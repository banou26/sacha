//! What comes out of a name, rather than only that it parses.

fn titles(input: &str) -> Vec<String> {
    sacha::parse(input).expect("parse").titles.unwrap_or_default()
}

fn group(input: &str) -> Option<String> {
    sacha::parse(input).expect("parse").groups?.first().cloned()
}

/// A name whose title only ever appears inside brackets used to produce no
/// title at all, because bracket text that was not metadata was discarded.
#[test]
fn recovers_a_title_that_only_exists_inside_brackets() {
    let t = titles("[FLsnow][名探偵プリキュア！/Star Detective Precure！][1080p][27][CHS&JPN]");
    assert!(
        t.iter().any(|s| s.contains("Star Detective Precure")),
        "expected the bracketed title, got {t:?}"
    );
    assert_eq!(
        group("[FLsnow][名探偵プリキュア！/Star Detective Precure！][1080p][27][CHS&JPN]").as_deref(),
        Some("FLsnow")
    );
}

/// Cross-language aliases live in trailing brackets and are what let a search
/// match a release whose main title is in another script.
#[test]
fn keeps_a_bracketed_alias_as_a_second_title() {
    let t = titles(
        "[NOOBDL] Атака Титанов: Последняя атака (Shingeki no Kyojin Movie - The Last Attack) 1080p WEB-DL x264",
    );
    assert!(t.iter().any(|s| s.contains("Атака")), "got {t:?}");
    assert!(t.iter().any(|s| s.contains("Shingeki no Kyojin")), "got {t:?}");
}

/// The joined-term split only fires when every piece is metadata, so an
/// ordinary hyphenated title keeps its hyphen.
#[test]
fn drops_annotations_hashes_and_joined_terms() {
    assert_eq!(titles("[whomst] Cyberpunk: Edgerunners (subs only)"), ["Cyberpunk: Edgerunners"]);
    let t = titles("[SubsPlease] Shingeki no Kyojin Movie (1080p) [40DD2396]");
    assert_eq!(t, ["Shingeki no Kyojin Movie"], "got {t:?}");
    let t = titles("[LoliHouse] Girls Band Cry [BDRip 1080p HEVC-10bit FLAC]");
    assert_eq!(t, ["Girls Band Cry"], "got {t:?}");
}

/// A hyphen inside a title is not a release-group marker: only a tag at the end
/// of the name is. `Kobayashi-san Chi no Maid Dragon` used to come apart into
/// titles `["Kobayashi", "no Maid Dragon"]` with group `san`.
#[test]
fn does_not_split_a_hyphenated_title() {
    for name in [
        "[Erai-raws] Kobayashi-san Chi no Maid Dragon [1080p][MultiSub]",
        "[Trix] Fate-Zero Remastered [BD 1080p]",
    ] {
        let t = titles(name);
        assert!(
            t.iter().any(|s| s.contains('-')),
            "hyphen lost in {t:?} for {name}"
        );
    }
}

/// A trailing group tag still parses, including when a version follows it.
#[test]
fn keeps_an_honorific_attached_to_the_title() {
    let r = sacha::parse("[Erai-raws] Kobayashi-san Chi no Maid Dragon [1080p][MultiSub]").unwrap();
    assert_eq!(r.titles.as_deref(), Some(&["Kobayashi-san Chi no Maid Dragon".to_string()][..]));
    assert_eq!(r.groups.as_deref(), Some(&["Erai-raws".to_string()][..]));
    assert_eq!(r.audio_language_terms, None);
}

/// Scene releases separate words with dots; the title has to come out spaced.
#[test]
fn still_reads_a_trailing_group_tag() {
    let r = sacha::parse("Digimon Beatbreak S01E41 1080p WEBRip x265-Rapta").unwrap();
    assert_eq!(r.groups.as_deref(), Some(&["Rapta".to_string()][..]));
    let r = sacha::parse("Cyberpunk Edgerunners WEB-DL 1080P EAC3 VOSTFR-LTPD v2").unwrap();
    assert_eq!(r.groups.as_deref(), Some(&["LTPD".to_string()][..]));
}

#[test]
fn classifies_streaming_sources_and_disc_types() {
    let r = sacha::parse("Digimon Beatbreak S01E41 1080p CR WEBRip 10bits x265-Rapta").unwrap();
    assert_eq!(r.source_terms.as_deref(), Some(&["CR".to_string()][..]));
    let r = sacha::parse("[Meakes] Attack on Titan (BD Remux 1080p x264 FLAC)").unwrap();
    assert!(r.type_terms.is_some_and(|t| t.iter().any(|s| s == "BD")));
    let r = sacha::parse("[MxT] Battle Through The Heavens (AMV) [4K HEVC AAC]").unwrap();
    assert_eq!(r.resolution_terms.as_deref(), Some(&["4K".to_string()][..]));
}

#[test]
fn splits_a_scene_release_title() {
    let t = titles("Shingeki.no.Kyojin.The.Last.Attack.Movie.2024.1080p.BDRip.x264-KuroNeko");
    assert!(
        t.iter().any(|s| s.starts_with("Shingeki no Kyojin The Last Attack")),
        "got {t:?}"
    );
}
