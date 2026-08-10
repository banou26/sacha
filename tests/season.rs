//! Every season spelling the metadata sources actually use.
//!
//! Counted over 4159 real titles from AniList and ani.zip: `Season N` 176, `Nth Season` 81,
//! `第N季` 62, `第N期` 43, `Part N` 31, `S<N>` 27, `シーズンN` 11, `Cour N` 6, `N기` 1.
//! Only `Season N` and `S<N>` used to parse, so a consumer comparing two media saw no season
//! at all for the other half and merged a show with its own next season.

fn seasons(input: &str) -> Vec<String> {
    sacha::parse(input).expect("parses").season_terms.unwrap_or_default()
}

fn titles(input: &str) -> Vec<String> {
    sacha::parse(input).expect("parses").titles.unwrap_or_default()
}

#[test]
fn reads_the_english_prefix_form() {
    assert_eq!(seasons("That Time I Got Reincarnated as a Slime Season 4"), ["Season 4"]);
    assert_eq!(seasons("Youjo Senki S2"), ["S2"]);
    assert_eq!(seasons("Jujutsu Kaisen: The Culling Game Part 1"), ["Part 1"]);
    assert_eq!(seasons("Dr. STONE SCIENCE FUTURE Cour 3"), ["Cour 3"]);
}

#[test]
fn reads_the_english_ordinal_form() {
    assert_eq!(seasons("Sousou no Frieren 2nd Season"), ["2nd Season"]);
    assert_eq!(seasons("Tensei Shitara Ken Deshita 2nd Season"), ["2nd Season"]);
    assert_eq!(seasons("Tensei Shitara Slime Datta Ken 4th Season"), ["4th Season"]);
    assert_eq!(seasons("Kanojo Okarishimasu 1st Season"), ["1st Season"]);
    assert_eq!(seasons("Osomatsu-san 3rd Season"), ["3rd Season"]);
}

#[test]
fn reads_the_cjk_forms() {
    assert_eq!(seasons("転生したら剣でした 第2期"), ["第2期"]);
    assert_eq!(seasons("幼女战记 第二季"), ["第二季"]);
    assert_eq!(seasons("小书痴的下克上 第四季"), ["第四季"]);
    assert_eq!(seasons("おでかけ子ザメ シーズン2"), ["シーズン2"]);
    assert_eq!(seasons("전생했더니 슬라임이었던 건에 대하여 4기"), ["4기"]);
}

#[test]
fn a_season_is_taken_out_of_the_title() {
    assert_eq!(titles("Sousou no Frieren 2nd Season"), ["Sousou no Frieren"]);
    assert_eq!(titles("転生したら剣でした 第2期"), ["転生したら剣でした"]);
    assert_eq!(titles("幼女战记 第二季"), ["幼女战记"]);
}

// A season number is an identity claim, so a false positive silently merges two different shows.
// Each of these looks like a season to one of the three new parsers and is not one.
#[test]
fn does_not_invent_a_season() {
    assert!(seasons("Death Note").is_empty());
    assert!(seasons("Cowboy Bebop").is_empty());
    // 話 and 集 count episodes, not seasons
    assert!(seasons("進撃の巨人 第1話").is_empty());
    assert!(seasons("鬼滅の刃 第25集").is_empty());
    // the ordinal word has to end where the number claims it does
    assert!(seasons("The 2nd Seasoning of Life").is_empty());
    // a bare ordinal names nothing: 284 of the 4159 real titles end in a number and most are not seasons
    assert!(seasons("Yami Shibai 17").is_empty());
    assert!(seasons("Kidou Keisatsu Patlabor EZY File 1").is_empty());
    // `S` needs its digits: `Part` and `Cour` must not swallow an ordinary word
    assert!(seasons("Part-Time Hero").is_empty());
    assert!(seasons("Cours de Francais").is_empty());
}

// The prefix and CJK spellings of one season appear side by side in fansub names, and the
// consumer wants both rather than whichever the tokenizer reached first.
#[test]
fn keeps_every_spelling_a_name_carries() {
    let both = seasons("[SweetSub] 小书痴的下克上 第四季 / Honzuki no Gekokujou S04 - 15 [WebRip]");
    assert!(both.contains(&"第四季".to_string()), "got {both:?}");
    assert!(both.contains(&"S04".to_string()), "got {both:?}");
}

// Regression: the season used to be left inside the title, so `Part 2` and `Part 1` of one show
// compared as different titles rather than as the same title in different seasons.
#[test]
fn a_release_name_keeps_its_other_fields() {
    let r = sacha::parse("[Erai-raws] Mushoku Tensei - Isekai Ittara Honki Dasu Part 2 - 01 ~ 12 [1080p]")
        .expect("parses");
    assert_eq!(r.season_terms.as_deref(), Some(&["Part 2".to_string()][..]));
    assert_eq!(r.titles.as_deref(), Some(&["Mushoku Tensei - Isekai Ittara Honki Dasu".to_string()][..]));
    assert_eq!(r.groups.as_deref(), Some(&["Erai-raws".to_string()][..]));
    assert_eq!(r.resolution_terms.as_deref(), Some(&["1080p".to_string()][..]));
}
