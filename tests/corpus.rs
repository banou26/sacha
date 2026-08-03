//! Runs the parser over the checked-in corpora and over non-ASCII names that previously panicked.

use std::panic::{catch_unwind, AssertUnwindSafe};

fn try_parse(input: &str) -> Result<(), String> {
    match catch_unwind(AssertUnwindSafe(|| sacha::parse(input))) {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(e)) => Err(format!("returned an error: {e}")),
        Err(_) => Err("panicked".to_string()),
    }
}

fn check_corpus(label: &str, body: &str) {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let failures: Vec<String> = body
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| try_parse(l).err().map(|why| format!("  {why}: {l}")))
        .collect();
    std::panic::set_hook(hook);

    assert!(
        failures.is_empty(),
        "{} of the {label} corpus failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn anime_corpus_parses() {
    check_corpus("anime", include_str!("anime.txt"));
}

#[test]
fn movies_corpus_parses() {
    check_corpus("movies", include_str!("movies.txt"));
}

// tests/nyaa.txt is a live sample from nyaa.si across the anime, audio and live-action categories; about a third of it is non-ASCII, which the two hand-written corpora barely cover
#[test]
fn nyaa_corpus_parses() {
    check_corpus("nyaa", include_str!("nyaa.txt"));
}

#[test]
fn every_name_yields_a_title() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let empty: Vec<&str> = include_str!("nyaa.txt")
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter(|l| {
            catch_unwind(AssertUnwindSafe(|| sacha::parse(l)))
                .ok()
                .and_then(Result::ok)
                .is_none_or(|r| r.titles.is_none_or(|t| t.is_empty()))
        })
        .collect();
    std::panic::set_hook(hook);

    assert!(
        empty.is_empty(),
        "{} names produced no title:\n{}",
        empty.len(),
        empty.join("\n")
    );
}

#[test]
fn non_ascii_names_parse() {
    let names = [
        "[Johker] Shingeki No Kyojin Henshū (Fan-Kaï) 1-23 - 1080p.MULTI.VF/VOSTFR.x264",
        "[VCB-Studio] 进击的巨人 / Shingeki no Kyojin / 進撃の巨人 10-bit 1080p HEVC BDRip",
        "[动漫国字幕组&恶魔岛字幕组&VCB-Studio] 进击的巨人 / Attack on Titan 10-bit 1080p HEVC BDRip",
        "[NOOBDL & FortunaTV] Атака Титанов: Последняя атака 1080p WEB-DL x264",
        "[Kazuma-kun] (WEB) [FLAC] TVアニメ「進撃の巨人 The Final Season」Original Sound Track",
        "[Xspitfire911] Shingeki No Kyojin Intégrale + OAV BDRIP 1080p X265 10bit VOSTFR",
        "Music Collection of Attack on Titan/進撃の巨人/Shingeki no Kyojin/进击的巨人 V2.0",
        "Shingeki.no.Kyojin.Chronicle.Movie.2020.1080p.BDRip.x264 進撃の巨人 〜クロニクル〜",
        "[Furretar] 台粤配 - 进击的巨人 [國語][粵語][国语][中配版] (進擊的巨人)",
        // its trailing word character is itself multi-byte; without that it looks like an ordinary ASCII name and gets pruned
        "Yuru Camp△",
        "進撃の巨人",
        "Pokémon",
    ];
    for name in names {
        assert!(try_parse(name).is_ok(), "failed on: {name}");
    }
}
