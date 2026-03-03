pub const RESOLUTIONS: &[u16] = &[320, 480, 540, 640, 720, 1080, 1440, 2160, 2880, 4320, 5120, 7680];

pub const VIDEO_CODECS: &[&str] = &[
    // Sorted by length descending to prevent short matches overriding longer ones
    "DIVX5", "DIVX6", "HEVC2",
    "H.264", "H.265", "X.264", "X.265",
    "MPEG-1", "MPEG-2",
    "12-BITS", "12 BITS", "12-BIT", "12 BIT", "12BITS", "12BIT",
    "10-BITS", "10 BITS", "10-BIT", "10 BIT", "10BITS", "10BIT",
    "8-BITS", "8 BITS", "8-BIT", "8 BIT", "8BITS", "8BIT",
    "Theora",
    "H264", "H265", "X264", "X265",
    "HEVC", "MP4V", "DIVX", "XVID",
    "AVC", "AV1", "VP8", "VP9",
];

pub const VIDEO_TERMS: &[&str] = &["HDR", "DV"];

pub const TYPE_TERMS: &[&str] = &["WEB-DL", "WEBRip", "WEB"];

pub const AUDIO_CODECS: &[&str] = &[
    // Sorted by length descending
    "DDP5.1", "E-AC-3",
    "FLACX4", "FLACX3", "FLACX2",
    "AACX4", "AACX3", "AACX2",
    "LOSSLESS", "VORBIS",
    "G.722", "G.711",
    "ALAC", "FLAC", "Opus", "DDP5",
    "EAC3", "AC3", "AAC", "AMR", "MP3", "OGG",
];

pub const AUDIO_TERMS: &[&str] = &["Dual-Audio", "Dual Audio"];

pub const CONTAINER_TERMS: &[&str] = &[
    "MPEG-2", "MPEG", "ADTS", "FLAC",
    "3GP", "MP4", "Ogg", "MOV", "MKV",
];

pub const SUBTITLE_TERMS: &[&str] = &[
    "multiple subtitle", "multi-subs", "multi subs", "multi-sub", "subtitle",
];

pub const BATCH_TERMS: &[&str] = &["batched", "batch"];

pub const SEASON_PART_TERMS: &[&str] = &["Season", "S"];

pub const AUDIO_LANGUAGE_TERMS: &[&str] = &["Optional Dual Audio", "Dual Audio"];

pub const AUDIO_LANGUAGE_TERMS_CASE_SENSITIVE: &[&str] = &["MULTi"];

pub const SOURCE_TERMS: &[(&str, &str)] = &[
    ("NF", "Netflix"),
    ("AMZ", "Amazon"),
];

pub const NORMALIZED_LANGUAGES: &[&str] = &[
    // Sorted by length descending
    "POR-BR", "SPA-LA",
    "SPA", "ARA", "FRE", "GER", "ITA", "JPN", "POR", "POL",
    "DUT", "NOB", "FIN", "TUR", "SWE", "GRE", "HEB", "RUM",
    "IND", "THA", "KOR", "DAN", "CHI", "VIE", "UKR", "HUN",
    "CES", "HRV", "MAY", "FIL", "RUS", "ENG",
    "VF", "FR",
];

pub const SUBTITLE_LANGUAGE_TERMS: &[&str] = &["VOSTFR"];
