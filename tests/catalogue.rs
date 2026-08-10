//! A CATALOGUE title, not a release name.
//!
//! Both go through the same parser, and the rules that read a release name were taking catalogue
//! titles apart: 106 of 2271 real AniList and ani.zip titles had a word moved into `groups`, and one
//! had no title left at all. The names below are all real, and every assertion here is a title the
//! parser used to throw away.

fn titles(input: &str) -> Vec<String> {
    sacha::parse(input).expect("parses").titles.unwrap_or_default()
}

fn groups(input: &str) -> Vec<String> {
    sacha::parse(input).expect("parses").groups.unwrap_or_default()
}

fn seasons(input: &str) -> Vec<String> {
    sacha::parse(input).expect("parses").season_terms.unwrap_or_default()
}

/// The lenticular brackets are part of the show's name. Reading them as a group tag left the season
/// standing alone as the title: `{"titles":["第3期"],"groups":["推しの子"]}`.
#[test]
fn a_cjk_bracket_is_part_of_the_name() {
    assert_eq!(titles("【推しの子】第3期"), ["推しの子"]);
    assert_eq!(seasons("【推しの子】第3期"), ["第3期"]);
    assert!(groups("【推しの子】第3期").is_empty());

    assert_eq!(titles("【推しの子】"), ["推しの子"]);
    assert_eq!(
        titles("「お前ごときが魔王に勝てると思うな」と勇者パーティを追放されたので、王都で気ままに暮らしたい"),
        [
            "お前ごときが魔王に勝てると思うな",
            "と勇者パーティを追放されたので、王都で気ままに暮らしたい"
        ]
    );
}

/// A square bracket IS how a group signs a release, so the two readings collide. Reading it as a
/// group has to leave a title behind, and here it does not.
#[test]
fn a_leading_square_bracket_that_is_the_whole_name() {
    assert_eq!(titles("[Oshi no Ko] 3rd Season"), ["Oshi no Ko"]);
    assert_eq!(seasons("[Oshi no Ko] 3rd Season"), ["3rd Season"]);
    assert!(groups("[Oshi no Ko] 3rd Season").is_empty());
}

/// The same bracket on a real release still reads as the group, because a title follows it.
#[test]
fn a_leading_square_bracket_that_is_a_group() {
    assert_eq!(groups("[SubsPlease] Oshi no Ko - 01 (1080p) [ABCD1234].mkv"), ["SubsPlease"]);
    assert_eq!(groups("[MTBB] Made in Abyss S2 - The Golden City of the Scorching Sun - 05"), ["MTBB"]);
    assert_eq!(groups("[whomst] Cyberpunk: Edgerunners (subs only)"), ["whomst"]);
    assert_eq!(titles("[whomst] Cyberpunk: Edgerunners (subs only)"), ["Cyberpunk: Edgerunners"]);
}

/// A hyphen glued to the name being read belongs to it. `Hana-Kimi` came out as `Hana`, and the
/// honorific suffixes went the same way: `-san`, `-chan`, `-sensei`, `-kun`, `-hime`.
#[test]
fn a_hyphenated_name_keeps_its_last_word() {
    for name in [
        "BAKI-DOU",
        "Hana-Kimi",
        "Akane-banashi",
        "Ganbare! Nakamura-kun!!",
        "A Misanthrope Teaches a Class for Demi-Humans",
        "Tensei Shitara Slime Datta Ken: Soukai no Namida-hen",
        "Welcome to Demon School! Iruma-kun Season 4",
    ] {
        assert!(groups(name).is_empty(), "{name} lost a word to groups: {:?}", groups(name));
    }
    assert_eq!(titles("BAKI-DOU"), ["BAKI-DOU"]);
    assert_eq!(titles("Hana-Kimi"), ["Hana-Kimi"]);
    assert_eq!(titles("Welcome to Demon School! Iruma-kun Season 4"), ["Welcome to Demon School! Iruma-kun"]);
}

/// A real tag still reads, because it follows the release metadata rather than a name.
#[test]
fn a_real_trailing_tag_still_reads() {
    assert_eq!(
        groups("Cyberpunk Edgerunners WEB-DL 1080P HDR DV EAC3 VF VOSTFR-LTPD v2"),
        ["LTPD"]
    );
    assert_eq!(
        groups("Cyberpunk - Edgerunners - S01v2 - MULTi 1080p WEB DV H.265 -NanDesuKa (NF)"),
        ["NanDesuKa"]
    );
    assert_eq!(
        groups("The.Infernal.Machine.2022.2160p.WEBRip.DD5.1.HDR.X.265-EVO[TGx]"),
        ["EVO[TGx"]
    );
}

/// `-サイレン-` is a subtitle in a decoration, not a tag. No release group signs off with a hyphen.
#[test]
fn a_decorated_subtitle_is_not_a_tag() {
    for name in [
        "PSYREN -サイレン-",
        "新劇場版 銀魂 -吉原大炎上-",
        "UniteUp! -Uni:Birth-",
        "黒執事 -緑の魔女編-",
        "GROTESQQQUE -グロテスク-",
    ] {
        assert!(groups(name).is_empty(), "{name} read a decoration as a group: {:?}", groups(name));
        assert!(!titles(name).is_empty(), "{name} lost its title");
    }
}

/// A closing bracket is a token boundary just as whitespace is, which is how `第3期` above is read
/// at all: japanese titles write no space after one.
#[test]
fn a_term_may_be_glued_to_a_closing_bracket() {
    assert_eq!(seasons("【推しの子】第3期"), ["第3期"]);
    assert_eq!(seasons("(Some Show)Season 2"), ["Season 2"]);
}

/// The whole point: nothing here may take a word out of a name.
#[test]
fn no_catalogue_title_loses_a_word() {
    for name in [
        "鬼滅の刃",
        "転生したらスライムだった件 第4期",
        "Sousou no Frieren",
        "Mission: Yozakura Family Season 2 Part 2",
        "Kaguya-sama: Love Is War -Stairway to Adulthood-",
        "Ryoumin 0-Nin Start no Henkyou Ryoushu-sama",
        "アイカツ！×プリパラ THE MOVIE -出会いのキセキ-",
    ] {
        assert!(groups(name).is_empty(), "{name} -> groups {:?}", groups(name));
        assert!(!titles(name).is_empty(), "{name} produced no title");
    }
}
