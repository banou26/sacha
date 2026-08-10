use std::sync::OnceLock;

/// The numeric source of truth for [`RESOLUTION_STRS`], which is what the
/// parser actually reads. Only the drift test between the two consults this, so
/// it is dead outside tests by design.
#[allow(dead_code)]
pub const RESOLUTIONS: &[u16] = &[320, 480, 540, 640, 720, 1080, 1440, 2160, 2880, 4320, 5120, 7680];

pub const VIDEO_CODECS: &[&str] = &[
    // sorted by length descending to prevent short terms overriding longer ones
    "DIVX5", "DIVX6", "HEVC2",
    "H.264", "H.265", "X.264",
    "12-BITS", "12 BITS", "12-BIT", "12 BIT", "12BITS", "12BIT",
    "10-BITS", "10 BITS", "10-BIT", "10 BIT", "10BITS", "10BIT",
    "8-BITS", "8 BITS", "8-BIT", "8 BIT", "8BITS", "8BIT",
    "MPEG-2", "MPEG-1",
    "Theora",
    "H.263",
    "H264", "H265", "X264", "X265",
    "HEVC", "MP4V", "DIVX", "XVID",
    "AV1", "AVC",
    "VP8", "VP9",
];

pub const AUDIO_CODECS: &[&str] = &[
    "FLACX4", "FLACX3", "FLACX2",
    "AACX4", "AACX3", "AACX2",
    "E-AC-3",
    "DDP5.1",
    "LOSSLESS",
    "Vorbis", "VORBIS",
    "DDP5", "DDP",
    "FLAC", "ALAC", "EAC3",
    "G.722", "G.711",
    "Opus",
    "AAC", "AMR", "MP3", "AC3", "OGG",
    "When",
];

pub const AUDIO_TERMS: &[&str] = &[
    "Dual-Audio", "Dual Audio",
];

pub const VIDEO_TERMS: &[&str] = &[
    "HDR", "DV",
];

pub const TYPE_TERMS: &[&str] = &[
    // sorted by length descending: the matcher takes the first hit, so "BD" must not shadow "BDRip"
    "BDRemux", "HDTVRip",
    "Blu-Ray", "Blu-ray",
    "BluRay", "BDRip", "DVDRip", "WEB-DL", "WEBRip",
    "WEB-RIP", "TVRip", "WEBDL", "REMUX", "HDTV",
    "BDMV", "DVD", "UHD", "WEB",
    "BD",
];

pub const CONTAINER_TERMS: &[&str] = &[
    "MPEG-2", "MPEG",
    "ADTS", "FLAC",
    "WEBM", "3GP", "MP4", "M4A", "M4V", "Ogg", "MOV", "MKV", "AVI",
];

pub const SUBTITLE_TERMS: &[&str] = &[
    "multiple subtitles", "multiple subtitle",
    "multi-subs", "multi subs", "multi-sub", "multisubs", "multisub",
    "subtitle",
];

/// Resolutions written as a shorthand rather than a line count.
pub const RESOLUTION_ALIASES: &[&str] = &["4320p", "2160p", "8K", "4K", "2K"];

pub const BATCH_TERMS: &[&str] = &[
    "batched", "batch",
];

// `Part` and `Cour` name a season's subdivision and were previously read as title words, so
// `Enen no Shouboutai: San no Shou Part 2` claimed to be the same media as Part 1.
pub const SEASON_PART_TERMS: &[&str] = &[
    "Season", "Part", "Cour", "シーズン", "S",
];

pub const AUDIO_LANGUAGE_TERMS: &[&str] = &[
    "Optional Dual Audio", "Dual Audio",
];

pub const AUDIO_LANGUAGE_TERMS_CASE_SENSITIVE: &[&str] = &[
    "MULTi",
];

#[allow(dead_code)]
pub struct LangMapping {
    pub code: &'static str,
    pub tag: &'static str,
}

pub const NORMALIZED_LANGUAGES: &[LangMapping] = &[
    LangMapping { code: "SPA-LA", tag: "es" },
    LangMapping { code: "POR-BR", tag: "pt" },
    LangMapping { code: "ENG", tag: "en" },
    LangMapping { code: "SPA", tag: "es" },
    LangMapping { code: "ARA", tag: "ar" },
    LangMapping { code: "FRE", tag: "fr" },
    LangMapping { code: "FR", tag: "fr" },
    LangMapping { code: "VF", tag: "fr" },
    LangMapping { code: "GER", tag: "de" },
    LangMapping { code: "ITA", tag: "it" },
    LangMapping { code: "JPN", tag: "ja" },
    LangMapping { code: "POR", tag: "pt" },
    LangMapping { code: "POL", tag: "pl" },
    LangMapping { code: "DUT", tag: "nl" },
    LangMapping { code: "NOB", tag: "nb" },
    LangMapping { code: "FIN", tag: "fi" },
    LangMapping { code: "TUR", tag: "tr" },
    LangMapping { code: "SWE", tag: "sv" },
    LangMapping { code: "GRE", tag: "el" },
    LangMapping { code: "HEB", tag: "he" },
    LangMapping { code: "RUM", tag: "rm" },
    LangMapping { code: "IND", tag: "id" },
    LangMapping { code: "THA", tag: "th" },
    LangMapping { code: "KOR", tag: "ko" },
    LangMapping { code: "DAN", tag: "da" },
    LangMapping { code: "CHI", tag: "zh" },
    LangMapping { code: "VIE", tag: "vi" },
    LangMapping { code: "UKR", tag: "uk" },
    LangMapping { code: "HUN", tag: "hu" },
    LangMapping { code: "CES", tag: "cs" },
    LangMapping { code: "HRV", tag: "hr" },
    LangMapping { code: "MAY", tag: "ms" },
    LangMapping { code: "FIL", tag: "fil" },
    LangMapping { code: "RUS", tag: "ru" },
];

pub const NORMALIZED_SUBTITLE_LANGUAGES: &[LangMapping] = &[
    LangMapping { code: "VOSTFR", tag: "fr" },
];

pub const SOURCE_TERMS: &[(&str, &str)] = &[
    ("NF", "Netflix"),
    ("AMZ", "Amazon"),
    ("AMZN", "Amazon"),
    ("CR", "Crunchyroll"),
    ("DSNP", "Disney+"),
    ("HULU", "Hulu"),
    ("ATVP", "Apple TV+"),
    ("HMAX", "HBO Max"),
    ("VRV", "VRV"),
    ("FUNI", "Funimation"),
    ("ADN", "Animation Digital Network"),
    ("B-Global", "Bilibili Global"),
    ("Baha", "Bahamut"),
    ("IQ", "iQIYI"),
];

/// Annotations that occupy a bracket on their own. They read as words but name
/// nothing, so they are dropped rather than kept as titles. Matched against the
/// whole bracket content, case-insensitively, never as a substring.
pub const NOTE_TERMS: &[&str] = &[
    "subs only", "sub only", "subbed", "dubbed", "hardsub", "hardsubs",
    "softsub", "softsubs", "eng sub", "eng subs", "no subs", "raw", "raws",
    "reseed", "fin", "end", "final", "complete", "completo", "ongoing",
    "airing", "repack", "fixed", "censored", "uncensored", "unknown",
    "new", "old", "part", "extra", "extras", "bonus", "misc", "other",
];

pub const DELIMITERS: &[(char, char)] = &[
    ('(', ')'),
    ('[', ']'),
    ('{', '}'),
    ('\u{300C}', '\u{300D}'), // 「」
    ('\u{300E}', '\u{300F}'), // 『』
    ('\u{3010}', '\u{3011}'), // 【】
    ('\u{FF08}', '\u{FF09}'), // （）
];

/// The brackets a release group writes its tag in.
///
/// Every delimiter above groups text, but only these three ever hold a group. The CJK corner and
/// lenticular brackets are part of a NAME: `【推しの子】第3期` and
/// `「お前ごときが魔王に勝てると思うな」と勇者パーティを追放されたので` are titles in full, and reading
/// the leading bracket as a tag threw the show's actual name away and promoted whatever followed
/// it to the title.
pub const GROUP_DELIMITERS: &[(char, char)] = &[('(', ')'), ('[', ']'), ('{', '}')];

// precomputed rather than formatted from `RESOLUTIONS` on the fly: the resolution parser runs at every character position and `to_string()` allocated a `String` per candidate
/// Decimal spellings of [`RESOLUTIONS`], in the same order. Precomputed because
/// the resolution parser runs at every character position and formatting these
/// on the fly allocated a `String` per candidate.
pub const RESOLUTION_STRS: &[&str] = &[
    "320", "480", "540", "640", "720", "1080", "1440", "2160", "2880", "4320", "5120", "7680",
];

/// Sorts by length descending so a short code cannot shadow a longer one that
/// starts with it.
fn by_length_desc(mut codes: Vec<&'static str>) -> Vec<&'static str> {
    codes.sort_by(|a, b| b.len().cmp(&a.len()));
    codes
}

// the three lists below are `OnceLock`-cached because all three are consulted at every character position while scanning a name, and allocating and sorting them per call was the bulk of parse time; do not go back to building them per call
/// All three lists below are consulted at every character position while
/// scanning a name, so they are built once rather than allocated and sorted per
/// call. That alone was the bulk of parse time.
///
/// Get all language codes for matching (sorted by length descending)
pub fn language_codes() -> &'static [&'static str] {
    static CODES: OnceLock<Vec<&'static str>> = OnceLock::new();
    CODES.get_or_init(|| by_length_desc(NORMALIZED_LANGUAGES.iter().map(|l| l.code).collect()))
}

/// Get all subtitle language codes for matching (sorted by length descending)
pub fn subtitle_language_codes() -> &'static [&'static str] {
    static CODES: OnceLock<Vec<&'static str>> = OnceLock::new();
    CODES.get_or_init(|| {
        by_length_desc(NORMALIZED_SUBTITLE_LANGUAGES.iter().map(|l| l.code).collect())
    })
}

/// Get all source term keys (sorted by length descending)
pub fn source_term_keys() -> &'static [&'static str] {
    static KEYS: OnceLock<Vec<&'static str>> = OnceLock::new();
    KEYS.get_or_init(|| by_length_desc(SOURCE_TERMS.iter().map(|(k, _)| *k).collect()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `RESOLUTION_STRS` mirrors `RESOLUTIONS` and has to stay in step with it.
    #[test]
    fn resolution_spellings_match_the_numbers() {
        let spelled: Vec<String> = RESOLUTIONS.iter().map(|r| r.to_string()).collect();
        assert_eq!(spelled, RESOLUTION_STRS);
    }

    /// Every list reached by a bucketed matcher, and whether it is matched
    /// case-sensitively.
    fn indexed_lists() -> Vec<(&'static str, Vec<&'static str>, bool)> {
        vec![
            ("TYPE_TERMS", TYPE_TERMS.to_vec(), false),
            ("AUDIO_CODECS", AUDIO_CODECS.to_vec(), false),
            ("AUDIO_TERMS", AUDIO_TERMS.to_vec(), false),
            ("CONTAINER_TERMS", CONTAINER_TERMS.to_vec(), false),
            ("VIDEO_CODECS", VIDEO_CODECS.to_vec(), false),
            ("VIDEO_TERMS", VIDEO_TERMS.to_vec(), false),
            ("BATCH_TERMS", BATCH_TERMS.to_vec(), false),
            ("SUBTITLE_TERMS", SUBTITLE_TERMS.to_vec(), false),
            ("SEASON_PART_TERMS", SEASON_PART_TERMS.to_vec(), false),
            ("RESOLUTION_ALIASES", RESOLUTION_ALIASES.to_vec(), false),
            ("AUDIO_LANGUAGE_TERMS", AUDIO_LANGUAGE_TERMS.to_vec(), false),
            ("AUDIO_LANGUAGE_TERMS_CASE_SENSITIVE", AUDIO_LANGUAGE_TERMS_CASE_SENSITIVE.to_vec(), true),
            ("language_codes", language_codes().to_vec(), true),
            ("subtitle_language_codes", subtitle_language_codes().to_vec(), false),
            ("source_term_keys", source_term_keys().to_vec(), true),
            ("RESOLUTION_STRS", RESOLUTION_STRS.to_vec(), false),
            ("NOTE_TERMS", NOTE_TERMS.to_vec(), false),
        ]
    }

    // an empty term would match at every position, and the bucketed matchers would silently skip it where the old full scan matched it; nothing else stops one from being added, so the absence is asserted rather than assumed
    /// An empty term would match at every position. Nothing else stops one from
    /// being added, and the bucketed matchers would silently skip it where the
    /// old full scan matched it, so the absence is asserted rather than assumed.
    #[test]
    fn no_term_list_contains_an_empty_string() {
        for (name, terms, _) in indexed_lists() {
            assert!(
                terms.iter().all(|t| !t.is_empty()),
                "{name} contains an empty term"
            );
        }
    }

    /// A bucket has to hold exactly the terms a full scan would have considered,
    /// in the order the list declares them. That is what makes bucketing produce
    /// the same winner as scanning, so the parser's output does not shift.
    #[test]
    fn buckets_match_a_full_scan_in_order() {
        for (name, terms, case_sensitive) in indexed_lists() {
            let index = if case_sensitive {
                TermIndex::exact(&terms)
            } else {
                TermIndex::icase(&terms)
            };
            for byte in 0u8..128 {
                let probe = String::from_utf8(vec![byte, b'x']).unwrap();
                let expected: Vec<&str> = terms
                    .iter()
                    .copied()
                    .filter(|t| {
                        let first = t.as_bytes()[0];
                        if case_sensitive {
                            first == byte
                        } else {
                            first.eq_ignore_ascii_case(&byte)
                        }
                    })
                    .collect();
                assert_eq!(
                    index.candidates(&probe),
                    expected.as_slice(),
                    "{name} bucket for byte {byte} diverges from a full scan"
                );
            }
            assert!(index.candidates("進撃の巨人").is_empty(), "{name} matched a CJK lead byte");
            assert!(index.candidates("").is_empty(), "{name} matched empty input");
        }
    }
}

/// A term list bucketed by the first byte of each term.
///
/// The matchers are called at nearly every character position, and each one
/// used to walk its whole list to reject on the first byte. Bucketing turns
/// that scan into an index: only the terms that can possibly match are visited,
/// in their original relative order, so the term that wins is unchanged.
pub struct TermIndex {
    flat: Vec<&'static str>,
    offsets: [u32; 257],
    case_sensitive: bool,
}

impl TermIndex {
    pub fn icase(terms: &[&'static str]) -> Self {
        Self::build(terms, false)
    }

    pub fn exact(terms: &[&'static str]) -> Self {
        Self::build(terms, true)
    }

    fn build(terms: &[&'static str], case_sensitive: bool) -> Self {
        let key = |t: &str| {
            let b = *t.as_bytes().first().unwrap_or(&0);
            if case_sensitive { b } else { b.to_ascii_lowercase() }
        };
        let mut counts = [0u32; 256];
        for t in terms {
            counts[key(t) as usize] += 1;
        }
        let mut offsets = [0u32; 257];
        let mut acc = 0;
        for b in 0..256 {
            offsets[b] = acc;
            acc += counts[b];
        }
        offsets[256] = acc;
        // counting sort, filling each bucket left to right, which keeps the terms in the order the original list declared them
        let mut cursor = offsets;
        let mut flat = vec![""; terms.len()];
        for t in terms {
            let k = key(t) as usize;
            flat[cursor[k] as usize] = t;
            cursor[k] += 1;
        }
        Self { flat, offsets, case_sensitive }
    }

    /// The terms that could start at `input`, in declaration order.
    #[inline]
    pub fn candidates(&self, input: &str) -> &[&'static str] {
        let Some(&b) = input.as_bytes().first() else {
            return &[];
        };
        let k = if self.case_sensitive { b } else { b.to_ascii_lowercase() } as usize;
        &self.flat[self.offsets[k] as usize..self.offsets[k + 1] as usize]
    }
}

macro_rules! term_index {
    ($name:ident, $kind:ident, $terms:expr) => {
        pub fn $name() -> &'static TermIndex {
            static I: OnceLock<TermIndex> = OnceLock::new();
            I.get_or_init(|| TermIndex::$kind($terms))
        }
    };
}

term_index!(ix_type_terms, icase, TYPE_TERMS);
term_index!(ix_audio_codecs, icase, AUDIO_CODECS);
term_index!(ix_audio_terms, icase, AUDIO_TERMS);
term_index!(ix_container_terms, icase, CONTAINER_TERMS);
term_index!(ix_video_codecs, icase, VIDEO_CODECS);
term_index!(ix_video_terms, icase, VIDEO_TERMS);
term_index!(ix_batch_terms, icase, BATCH_TERMS);
term_index!(ix_subtitle_terms, icase, SUBTITLE_TERMS);
term_index!(ix_season_part_terms, icase, SEASON_PART_TERMS);
term_index!(ix_resolution_aliases, icase, RESOLUTION_ALIASES);
term_index!(ix_audio_language_terms, icase, AUDIO_LANGUAGE_TERMS);
term_index!(ix_audio_language_exact, exact, AUDIO_LANGUAGE_TERMS_CASE_SENSITIVE);

pub fn ix_language_codes() -> &'static TermIndex {
    static I: OnceLock<TermIndex> = OnceLock::new();
    I.get_or_init(|| TermIndex::exact(language_codes()))
}

pub fn ix_subtitle_language_codes() -> &'static TermIndex {
    static I: OnceLock<TermIndex> = OnceLock::new();
    I.get_or_init(|| TermIndex::icase(subtitle_language_codes()))
}

pub fn ix_source_term_keys() -> &'static TermIndex {
    static I: OnceLock<TermIndex> = OnceLock::new();
    I.get_or_init(|| TermIndex::exact(source_term_keys()))
}
