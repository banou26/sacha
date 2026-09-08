//! The form a title is compared in.
//!
//! Both sides of a title match go through here, and that symmetry is the whole point: a score is
//! only interpretable against a self score computed on the same string, so the alias and the release
//! must be normalised the same way. Comparing a raw query against a parsed name is what let an
//! unrelated show score above 1.
//!
//! Two forms come out of a title. The KEY is the title with case, separators and spacing settled.
//! The BARE form additionally drops what names a part of the show rather than the show: a year a
//! catalogue appends, a season, part or cour marker either side adds, and an episode tail a release
//! carries. Both are offered, because a show whose real title ends in a number (`Thunder 3`) would
//! otherwise lose it.

use crate::normalize::normalize_separators;

/// The characters a title can be spelled two ways in, mapped to the spelling an alias uses.
///
/// Both are measured on real names: `Ranma ½` and `Ranma 1/2` are the same show and share nothing
/// but `ranma` without this (0.549 against 1.000 with it), and a Japanese recorder writes its
/// filenames in full-width latin, where an alias is written in half-width.
fn compat(c: char) -> Option<&'static str> {
    Some(match c {
        '½' => "1/2",
        '⅓' => "1/3",
        '⅔' => "2/3",
        '¼' => "1/4",
        '¾' => "3/4",
        '⅕' => "1/5",
        '⅙' => "1/6",
        '⅛' => "1/8",
        '\u{3000}' => " ",
        _ => return None,
    })
}

/// Case folded, separator normalised, whitespace collapsed.
pub fn key(text: &str) -> String {
    let widened: String = text
        .chars()
        .map(|c| match c {
            // full-width forms sit exactly 0xFEE0 above their ASCII twin
            '\u{FF01}'..='\u{FF5E}' => char::from_u32(c as u32 - 0xFEE0).unwrap_or(c).to_string(),
            other => compat(other).map(str::to_string).unwrap_or_else(|| other.to_string()),
        })
        .collect();
    let separated = normalize_separators(&widened);
    let lowered = separated.to_lowercase();
    let mut out = String::with_capacity(lowered.len());
    let mut at_space = true;
    for c in lowered.chars() {
        if c.is_whitespace() {
            if !at_space {
                out.push(' ');
                at_space = true;
            }
        } else {
            out.push(c);
            at_space = false;
        }
    }
    trim_edges(out.trim())
}

fn trim_edges(text: &str) -> String {
    text.trim_matches(|c: char| " -~:|,._".contains(c)).to_string()
}

fn core(token: &str) -> &str {
    token.trim_matches(|c: char| "()[]{}:,.-~|_!?\"'".contains(c))
}

fn digits(token: &str, max: usize) -> bool {
    !token.is_empty() && token.len() <= max && token.bytes().all(|b| b.is_ascii_digit())
}

/// `s2`, `season2`, `part2`, `cour2`
fn word_number(token: &str) -> bool {
    ["season", "part", "cour", "s"]
        .iter()
        .any(|w| token.strip_prefix(w).is_some_and(|rest| digits(rest, 2)))
}

fn ordinal(token: &str) -> bool {
    ["st", "nd", "rd", "th"]
        .iter()
        .any(|s| token.strip_suffix(s).is_some_and(|rest| digits(rest, 2)))
}

fn season_word(token: &str) -> bool {
    matches!(token, "season" | "seasons" | "part" | "parts" | "cour" | "cours" | "s")
}

/// `s01e07`
fn episode_code(token: &str) -> bool {
    let Some(rest) = token.strip_prefix('s') else { return false };
    let Some(e) = rest.find('e') else { return false };
    digits(&rest[..e], 2) && digits(&rest[e + 1..], 4)
}

fn parenthesised_year(token: &str) -> bool {
    let inner = core(token);
    inner != token
        && inner.len() == 4
        && digits(inner, 4)
        && (inner.starts_with("19") || inner.starts_with("20"))
}

/// The key with the parts that are not the show's name taken off.
pub fn bare(key: &str) -> String {
    let tokens: Vec<&str> = key.split_whitespace().collect();
    let mut kept: Vec<&str> = Vec::with_capacity(tokens.len());
    let mut i = 0;
    while i < tokens.len() {
        let token = tokens[i];
        let word = core(token);
        let next = tokens.get(i + 1).map(|t| core(t));

        if parenthesised_year(token)
            || word_number(word)
            || episode_code(word)
        {
            i += 1;
            continue;
        }
        // `season 2`, `part 2`, `cour 2`, and `2nd season`
        if (season_word(word) && next.is_some_and(|n| digits(n, 2)))
            || (ordinal(word) && next.is_some_and(season_word))
        {
            i += 2;
            continue;
        }
        kept.push(token);
        i += 1;
    }

    // an episode tail: `- 07`, `~ 07v2`, `- 01-06`
    while kept.len() >= 2 {
        let last = core(kept[kept.len() - 1]);
        let stem = last.split_once('v').map_or(last, |(a, _)| a);
        let stem = stem.split_once('-').map_or(stem, |(a, _)| a);
        if digits(stem, 4) && matches!(kept[kept.len() - 2], "-" | "~" | "|" | ":") {
            kept.truncate(kept.len() - 2);
        } else {
            break;
        }
    }

    trim_edges(&kept.join(" "))
}

/// How much of a title a string carries. A CJK character is worth two, because a four character
/// Chinese title says as much as an eight character latin one, and the floor below has to keep the
/// first while dropping `part`.
pub fn weight(text: &str) -> usize {
    text.chars().map(|c| if c.is_ascii() { 1 } else { 2 }).sum()
}
