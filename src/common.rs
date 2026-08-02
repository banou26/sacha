use std::sync::OnceLock;

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
    // sorted by length descending
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
    // sorted by length descending: the matcher takes the first hit, so "BD"
    // must not shadow "BDRip"
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

pub const SEASON_PART_TERMS: &[&str] = &[
    "Season", "S",
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
}
