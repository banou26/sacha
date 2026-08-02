//! Separator normalization.
//!
//! Scene releases write word boundaries as `.` or `_` rather than spaces:
//!
//! ```text
//! Shingeki.no.Kyojin.The.Last.Attack.Movie.2024.1080p.BDRip.x264
//! [WZF]Shingeki_no_Kyojin_Season_3_-_01-06[HD][X264-AAC]
//! ```
//!
//! The tokenizer keys metadata terms off whitespace, so without a space in
//! sight it recognizes nothing and the entire name collapses into one title.
//! Converting the dominant separator to a space up front fixes that.
//!
//! A dot is only a separator sometimes: `H.264`, `DDP5.1` and `13.5` all keep
//! theirs. Those are detected and left alone, and the substitution only happens
//! when the name really is separator-delimited rather than space-delimited.

use std::borrow::Cow;

use crate::common::{AUDIO_CODECS, CONTAINER_TERMS, VIDEO_CODECS};

/// Byte ranges covered by a known term that spells a dot, such as `H.264` or
/// `DDP5.1`. Driving this off the term lists rather than the shape of the text
/// is what tells `H.264` apart from `B.44`, which look identical.
fn term_ranges(input: &str) -> Vec<(usize, usize)> {
    // ASCII-only lowercasing keeps byte offsets aligned with `input`
    let haystack = input.to_ascii_lowercase();
    let mut ranges = Vec::new();
    for term in VIDEO_CODECS
        .iter()
        .chain(AUDIO_CODECS)
        .chain(CONTAINER_TERMS)
        .filter(|t| t.contains('.'))
    {
        let needle = term.to_ascii_lowercase();
        let mut from = 0;
        while let Some(offset) = haystack[from..].find(&needle) {
            let start = from + offset;
            ranges.push((start, start + needle.len()));
            from = start + needle.len();
        }
    }
    ranges
}

/// True when the dot at `idx` carries meaning and must survive: it belongs to a
/// known term, or it sits inside a short numeric group (`5.1`, `13.5`,
/// `44.1kHz`, `2022.09.18`).
///
/// The length of the digit run after the dot is what separates a channel count
/// from a separator: `AAC2.0` keeps its dot, `2024.1080p` does not.
fn is_load_bearing_dot(bytes: &[u8], idx: usize, terms: &[(usize, usize)]) -> bool {
    if terms.iter().any(|&(s, e)| idx >= s && idx < e) {
        return true;
    }
    let digits_after = bytes[idx + 1..]
        .iter()
        .take_while(|b| b.is_ascii_digit())
        .count();
    (1..=2).contains(&digits_after)
        && idx
            .checked_sub(1)
            .is_some_and(|i| bytes[i].is_ascii_digit())
}

/// Byte offsets of every dot that looks like a word separator.
fn separator_dots(input: &str) -> Vec<usize> {
    let bytes = input.as_bytes();
    let terms = term_ranges(input);
    input
        .char_indices()
        .filter(|&(i, c)| c == '.' && !is_load_bearing_dot(bytes, i, &terms))
        .map(|(i, _)| i)
        .collect()
}

/// Rewrites word separators to spaces. Returns the input unchanged when there
/// is nothing to rewrite.
///
/// Every separator dot is rewritten rather than only those in a name that looks
/// dot-delimited overall. Names mix the two styles freely, as in
/// `[Johker] Shingeki No Kyojin (Fan-Kaï) 1-23 - 1080p.MULTI.VF/VOSTFR.x264`,
/// where a dotted metadata tail hangs off a spaced title. Judging the name as a
/// whole leaves that tail stuck together.
pub fn normalize_separators(input: &str) -> Cow<'_, str> {
    let dots = separator_dots(input);
    let underscores = input.contains('_');
    if dots.is_empty() && !underscores {
        return Cow::Borrowed(input);
    }

    let mut out = String::with_capacity(input.len());
    let mut dot_iter = dots.iter().peekable();
    for (i, c) in input.char_indices() {
        let is_separator_dot = dot_iter.peek() == Some(&&i);
        if is_separator_dot {
            dot_iter.next();
        }
        match c {
            '_' => out.push(' '),
            '.' if is_separator_dot => out.push(' '),
            _ => out.push(c),
        }
    }
    Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_dot_delimited_names() {
        assert_eq!(
            normalize_separators("Shingeki.no.Kyojin.The.Last.Attack.Movie.2024.1080p.BDRip"),
            "Shingeki no Kyojin The Last Attack Movie 2024 1080p BDRip"
        );
    }

    #[test]
    fn splits_underscore_delimited_names() {
        assert_eq!(
            normalize_separators("Shingeki_no_Kyojin_Season_3_-_01-06"),
            "Shingeki no Kyojin Season 3 - 01-06"
        );
    }

    #[test]
    fn keeps_codec_and_channel_dots() {
        // the dot is part of the term in every one of these
        assert_eq!(normalize_separators("A.B.C.H.264.D.E.F"), "A B C H.264 D E F");
        assert_eq!(
            normalize_separators("Show.Name.2024.DDP5.1.H.265.AAC2.0.x264"),
            "Show Name 2024 DDP5.1 H.265 AAC2.0 x264"
        );
        assert_eq!(
            normalize_separators("Capitulo_13.5_HD_X264_AAC_1280x720_Sub_Esp"),
            "Capitulo 13.5 HD X264 AAC 1280x720 Sub Esp"
        );
    }

    #[test]
    fn keeps_dotted_dates_but_splits_year_from_resolution() {
        assert_eq!(
            normalize_separators("Show.2022.09.18.1080p.WEB"),
            "Show 2022.09.18 1080p WEB"
        );
        assert_eq!(normalize_separators("A.B.44.1kHz.C"), "A B 44.1kHz C");
    }

    #[test]
    fn leaves_space_delimited_names_alone() {
        for name in [
            "[Anime Time] Attack on Titan Movie 05 - The Last Attack [BD] [Dual Audio]",
            "Attack on Titan The Last Attack 2024 1080p CR WEB-DL DUAL AAC2.0 H 264-VARYG",
            "[Erai-raws] Shingeki no Kyojin: Kanketsu-hen - Movie [1080p CR WEBRip HEVC EAC3]",
        ] {
            assert!(matches!(normalize_separators(name), Cow::Borrowed(_)), "{name}");
        }
    }

    /// A dotted metadata tail on an otherwise spaced name still has to split.
    #[test]
    fn splits_a_dotted_tail_on_a_spaced_name() {
        assert_eq!(
            normalize_separators("[Johker] Shingeki No Kyojin Henshū 1-23 - 1080p.MULTI.VF/VOSTFR.x264"),
            "[Johker] Shingeki No Kyojin Henshū 1-23 - 1080p MULTI VF/VOSTFR x264"
        );
    }

    #[test]
    fn handles_non_ascii_and_edges() {
        assert_eq!(normalize_separators(""), "");
        assert_eq!(normalize_separators("."), " ");
        assert_eq!(normalize_separators("進撃の巨人.2024.1080p"), "進撃の巨人 2024 1080p");
        // a dot after a multi-byte char is still a separator
        assert_eq!(normalize_separators("Henshū.Fan.Kaï.1080p"), "Henshū Fan Kaï 1080p");
    }
}
