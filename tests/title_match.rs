//! Deciding whether a release names a given show.
//!
//! The numbers here come from 56,380 labelled decisions over 4,063 real nyaa names, in two corpora
//! fetched a show at a time and labelled by a substring oracle rather than by this matcher. Every
//! threshold from 0.72 to 0.83 was free of both false positives and false negatives on both, and
//! 0.78 is the middle of that window and what these tests assert against.
use sacha::Index;

/// The middle of the measured error free window.
const ACCEPT: f32 = 0.78;

fn score(name: &str, aliases: &[&str]) -> f32 {
    let names = vec![name.to_string()];
    let aliases: Vec<String> = aliases.iter().map(|a| a.to_string()).collect();
    Index::build(&names).title_scores(&aliases)[0]
}

#[test]
fn accepts_a_release_whose_title_is_shorter_than_the_alias() {
    // the catalogue spells the show at length and the release does not: the shorter side is the
    // normaliser, and the alias is the needle only when it is the shorter of the two
    for name in [
        "[Judas] Mushoku Tensei (Jobless Reincarnation) - S02E13",
        "[SubsPlease] Mushoku Tensei S3 - 11 (1080p) [4492A492].mkv",
    ] {
        let score = score(name, &["Mushoku Tensei: Jobless Reincarnation Season 2 (2024)"]);
        assert!(score >= ACCEPT, "{name} scored {score}");
    }
}

/// A cour marker is not part of the show's name, and neither is the year a catalogue appends.
#[test]
fn accepts_across_a_season_part_or_cour_marker() {
    for (name, alias) in [
        ("[Erai-raws] Spy x Family Cour 2 - 01", "Spy x Family (2022)"),
        ("[Erai-raws] Dr. Stone - New World Cour 2 - 01", "Dr. Stone Season 3 Part 2"),
        ("[SubsPlease] Sousou no Frieren 2nd Season - 05", "Sousou no Frieren"),
    ] {
        let score = score(name, &[alias]);
        assert!(score >= ACCEPT, "{name} vs {alias} scored {score}");
    }
}

/// Separating seasons is the season number and the airing window's job. This answers a different
/// question, and a sibling season really does name the same show.
#[test]
fn accepts_a_sibling_season() {
    let score = score("[SubsPlease] Dr. Stone - Science Future - 05 (1080p)", &["Dr. Stone Season 3 Part 2"]);
    assert!(score >= ACCEPT, "sibling season scored {score}");
}

/// The failure the api was rebuilt for: a raw query scored against parsed names, normalised by a
/// parsed query, put an unrelated show at 0.889.
#[test]
fn rejects_an_unrelated_show() {
    for (name, alias) in [
        ("[Judas] Sakamoto Days - S01E22v2", "dr. stone season 3 part 2"),
        ("[SubsPlease] Mission Yozakura Family - 05", "SPY x FAMILY"),
        ("[SubsPlease] Oshi no Ko S3 (01-11) (1080p) [Batch]", "Shingeki no Kyojin"),
        // a short alias scattered across a long name: free at frizbee's default gap penalty, which
        // reads this pair at 0.824, and 0.55 at the penalty this api scores titles with
        (
            "[ok] Shingeki no Kyojin - The Final Season (Attack on Titan) - ENDING NC [1080p] - TVアニメ「進撃の巨人」The Final SeasonノンクレジットED.mkv",
            "Oshi no Ko",
        ),
    ] {
        let score = score(name, &[alias]);
        assert!(score < ACCEPT, "{name} vs {alias} scored {score}");
    }
}

/// `Season 3 Part 2` leaves the word `part` behind as a title candidate, and a four character
/// candidate is contained in anything. Without the weight floor this pair reads 0.895.
#[test]
fn a_leftover_word_cannot_carry_a_match() {
    let name = "[Erai-raws] Shingeki no Kyojin Season 3 Part 2 - 01 ~ 10 [1080p][BATCH][Multiple Subtitle]";
    let score = score(name, &["Dr. Stone Season 3 Part 2"]);
    assert!(score < ACCEPT, "a stray `part` carried the match: {score}");
}

/// A release named only in Chinese answers to its Chinese alias and to nothing else, which is why
/// aliases are a list rather than a string.
#[test]
fn matches_within_a_script_not_across_one() {
    let name = "[GM-Team][国漫][凡人修仙传 慕兰之战][Fan Ren Xiu Xian Zhuan][2026][13][AVC][GB][1080P]";
    let native = score(name, &["凡人修仙传"]);
    let english = score(name, &["A Record of a Mortal's Journey to Immortality"]);
    assert!(native >= ACCEPT, "native alias scored {native}");
    assert!(english < ACCEPT, "english alias scored {english}");
    // and the two together find it, which is how a caller is meant to ask
    let both = score(name, &["A Record of a Mortal's Journey to Immortality", "凡人修仙传"]);
    assert!(both >= ACCEPT, "the alias list scored {both}");
}

/// A show whose title is spelled two ways. `Ranma ½` and `Ranma 1/2` share nothing but `ranma`
/// until the compatibility characters are folded, and read 0.549 without it.
#[test]
fn folds_the_characters_a_title_is_spelled_two_ways_in() {
    for name in [
        // 0.549 and 0.585 without the fold
        "[Exiled-Destiny] Ranma ½ OVA [Dual Audio]",
        "Ranma ½  -  11 | Ranma ½ (2024) [English Dub][1080p]",
        "Ranma1-2.2024.S01.REPACK.1080p.NF.WEB-DL.AAC2.0.DV.HDR.H.265-SasukeducK",
    ] {
        let score = score(name, &["Ranma 1/2"]);
        assert!(score >= ACCEPT, "{name} scored {score}");
    }
}

/// A japanese title inside a recorder's filename, where the parser reads the timestamp and loses the
/// show. The whole name is offered as a candidate for exactly this.
#[test]
fn finds_a_title_the_parser_missed() {
    let name = "2021年11月29日00時05分00秒-進撃の巨人「悔いなき選択―前編―」[字]_ＮＨＫ総合１・東京.ts";
    let score = score(name, &["進撃の巨人"]);
    assert!(score >= ACCEPT, "scored {score}");
}

/// The property the old recipe could not hold: the two sides are normalised the same way and scored
/// against themselves, so a score is a ratio and can never exceed 1.
#[test]
fn a_score_is_a_ratio_and_never_exceeds_one() {
    let names: Vec<String> = [
        "[EMBER] Shingeki no Kyojin (2023) (Season 4 Part 03+04) [BDRip] (Attack on Titan Season 4 Part 4) (Batch)",
        "[Judas] Sakamoto Days - S01E22v2",
        "[SubsPlease] Mushoku Tensei S3 - 11 (1080p) [4492A492].mkv",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let aliases: Vec<String> = ["dr. stone season 3 part 2", "Attack on Titan", "進撃の巨人"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    for score in Index::build(&names).title_scores(&aliases) {
        assert!((0.0..=1.0).contains(&score), "score out of range: {score}");
    }
}

/// An alias against a release that carries exactly that alias scores 1: the self score in the
/// denominator is computed on the same string that was matched.
#[test]
fn an_alias_against_itself_scores_one() {
    for alias in ["Sakamoto Days", "進撃の巨人", "Dr. Stone Season 3 Part 2"] {
        let score = score(alias, &[alias]);
        assert_eq!(score, 1.0, "{alias} scored {score}");
    }
}

#[test]
fn reports_the_pair_behind_a_score() {
    let names = vec!["[Erai-raws] Dr. Stone - New World Cour 2 - 01".to_string()];
    let aliases = vec!["Dr. Stone Season 3 Part 2".to_string()];
    let matches = Index::build(&names).title_matches(&aliases, ACCEPT);
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].index, 0);
    assert!(matches[0].title.contains("dr stone"), "{:?}", matches[0].title);
    assert_eq!(matches[0].alias, "dr stone");
    assert!(Index::build(&names).title_matches(&["Sakamoto Days".to_string()], ACCEPT).is_empty());
}

/// frizbee aborts on a needle longer than its score can represent, and in wasm that takes the module
/// with it. Both sides are needles here, so a very long RELEASE NAME is as dangerous as a long query.
#[test]
fn survives_a_name_and_an_alias_longer_than_the_matcher_accepts() {
    let long = "x".repeat(50_000);
    let names = vec![long.clone(), "[Judas] Sakamoto Days - S01E22v2".to_string()];
    let scores = Index::build(&names).title_scores(&[long.clone(), "Sakamoto Days".to_string()]);
    assert_eq!(scores.len(), 2);
    assert!(scores[1] >= ACCEPT, "the ordinary name still matched: {}", scores[1]);
}

#[test]
fn handles_empty_input() {
    assert!(Index::build(&[]).title_scores(&["anything".to_string()]).is_empty());
    let names = vec!["[Judas] Sakamoto Days - S01E22v2".to_string()];
    assert_eq!(Index::build(&names).title_scores(&[]), vec![0.0]);
    assert_eq!(Index::build(&names).title_scores(&["".to_string()]), vec![0.0]);
}
