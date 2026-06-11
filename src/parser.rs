use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, take_while1, take_while_m_n},
    character::complete::{char, satisfy},
    combinator::recognize,
    multi::many1,
    sequence::tuple,
};

use crate::common::*;

/// Case-insensitive tag match: matches the input against the pattern ignoring case,
/// but returns the original text from the input (preserving original casing).
fn istr<'a>(pattern: &'a str) -> impl FnMut(&'a str) -> IResult<&'a str, &'a str> {
    move |input: &'a str| {
        let pat_len = pattern.len();
        if input.len() < pat_len {
            return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
        }
        if input[..pat_len].eq_ignore_ascii_case(pattern) {
            Ok((&input[pat_len..], &input[..pat_len]))
        } else {
            Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)))
        }
    }
}

// ---------------------------------------------------------------------------
// Token types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum MetadataKind {
    Group,
    Title,
    Resolution,
    VideoCodec,
    AudioCodec,
    AudioTerm,
    VideoTerm,
    TypeTerm,
    ContainerTerm,
    SubtitleTerm,
    BatchTerm,
    SeasonTerm,
    EpisodeTerm,
    AudioLanguage,
    SubtitleLanguage,
    SourceTerm,
    Date,
    VersionTerm,
}

#[derive(Debug, Clone)]
pub struct MetadataToken {
    pub kind: MetadataKind,
    pub value: String,
}

#[derive(Debug, Clone)]
enum RootToken {
    Metadata(Vec<MetadataToken>),
    Data(String),
}

// ---------------------------------------------------------------------------
// Primitive parsers
// ---------------------------------------------------------------------------

fn is_whitespace(c: char) -> bool {
    c == ' ' || c == '\t'
}

fn ws(input: &str) -> IResult<&str, &str> {
    take_while1(is_whitespace)(input)
}

fn ws0(input: &str) -> IResult<&str, &str> {
    Ok(take_while_m_n(0, usize::MAX, is_whitespace)(input)?)
}

fn digit_char(input: &str) -> IResult<&str, char> {
    satisfy(|c| c.is_ascii_digit())(input)
}

fn digits1(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_ascii_digit())(input)
}

// ---------------------------------------------------------------------------
// Term matchers - try each term from a list (longest first)
// ---------------------------------------------------------------------------

fn match_term_icase<'a>(terms: &'static [&'static str], input: &'a str) -> IResult<&'a str, &'a str> {
    for term in terms {
        if let Ok(result) = istr(term)(input) {
            return Ok(result);
        }
    }
    Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)))
}

fn match_term_exact<'a>(terms: &'static [&'static str], input: &'a str) -> IResult<&'a str, &'a str> {
    for term in terms {
        if let Ok(result) = tag::<&str, &str, nom::error::Error<&str>>(*term)(input) {
            return Ok(result);
        }
    }
    Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)))
}

fn match_term_list_icase<'a>(terms: &[&str], input: &'a str) -> IResult<&'a str, &'a str> {
    for term in terms {
        let pat_len = term.len();
        if input.len() >= pat_len && input[..pat_len].eq_ignore_ascii_case(term) {
            return Ok((&input[pat_len..], &input[..pat_len]));
        }
    }
    Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)))
}

// ---------------------------------------------------------------------------
// Resolution parser: e.g. "1080p", "1080px", "1080", "1080x720p"
// ---------------------------------------------------------------------------

fn resolution_number(input: &str) -> IResult<&str, &str> {
    for res in RESOLUTIONS {
        let s = itoa_buf(*res);
        if input.starts_with(s.as_str()) {
            let len = s.len();
            return Ok((&input[len..], &input[..len]));
        }
    }
    Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)))
}

fn itoa_buf(n: u16) -> String {
    let mut buf = String::new();
    let _ = std::fmt::Write::write_fmt(&mut buf, format_args!("{}", n));
    buf
}

fn resolution_primitive(input: &str) -> IResult<&str, String> {
    let (rest, num) = resolution_number(input)?;
    // try "px" suffix
    if let Ok((rest2, sfx)) = istr("px")(rest) {
        return Ok((rest2, format!("{}{}", num, sfx)));
    }
    // try "p" suffix
    if let Ok((rest2, sfx)) = istr("p")(rest) {
        return Ok((rest2, format!("{}{}", num, sfx)));
    }
    // bare number
    Ok((rest, num.to_string()))
}

fn resolution_token(input: &str) -> IResult<&str, String> {
    let (rest, first) = resolution_primitive(input)?;
    // try "x" separator for e.g. "1080x720p"
    if let Ok((rest2, _)) = istr("x")(rest) {
        if let Ok((rest3, second)) = resolution_primitive(rest2) {
            return Ok((rest3, format!("{}x{}", first, second)));
        }
    }
    Ok((rest, first))
}

// ---------------------------------------------------------------------------
// Version parser: "v1", "V2", etc.
// ---------------------------------------------------------------------------

fn version_token(input: &str) -> IResult<&str, String> {
    let (rest, v) = satisfy(|c| c == 'v' || c == 'V')(input)?;
    let (rest, d) = digit_char(rest)?;
    Ok((rest, format!("{}{}", v, d)))
}

// ---------------------------------------------------------------------------
// Date parser: "2022", "2022.01.15", "15.01.2022"
// ---------------------------------------------------------------------------

fn four_digits(input: &str) -> IResult<&str, &str> {
    take_while_m_n(4, 4, |c: char| c.is_ascii_digit())(input)
}

fn two_digits(input: &str) -> IResult<&str, &str> {
    take_while_m_n(2, 2, |c: char| c.is_ascii_digit())(input)
}

fn date_token(input: &str) -> IResult<&str, String> {
    // try YYYY.MM.DD
    if let Ok((rest, result)) = recognize(tuple((
        four_digits,
        char('.'),
        two_digits,
        char('.'),
        two_digits,
    )))(input) {
        return Ok((rest, result.to_string()));
    }
    // try DD.MM.YYYY
    if let Ok((rest, result)) = recognize(tuple((
        two_digits,
        char('.'),
        two_digits,
        char('.'),
        four_digits,
    )))(input) {
        return Ok((rest, result.to_string()));
    }
    // bare year (4 digits)
    let (rest, y) = four_digits(input)?;
    Ok((rest, y.to_string()))
}

// ---------------------------------------------------------------------------
// Episode range parser: "01 ~ 10", "01-10", "01 - 10"
// ---------------------------------------------------------------------------

fn episode_token(input: &str) -> IResult<&str, String> {
    let (rest, start) = digits1(input)?;
    let (rest, _) = ws0(rest)?;
    let (rest, sep) = alt((char('~'), char('-')))(rest)?;
    let (rest, _) = ws0(rest)?;
    let (rest, end) = digits1(rest)?;
    Ok((rest, format!("{} {} {}", start, sep, end)))
}

// ---------------------------------------------------------------------------
// Season parser: "Season 01", "S01"
// ---------------------------------------------------------------------------

fn season_token(input: &str) -> IResult<&str, String> {
    let (rest, part) = match_term_icase(SEASON_PART_TERMS, input)?;
    let (rest, digits_and_spaces) = take_while1(|c: char| c.is_ascii_digit() || c == ' ')(rest)?;
    if !digits_and_spaces.chars().any(|c| c.is_ascii_digit()) {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
    }
    let trimmed_end = digits_and_spaces.trim_end();
    Ok((rest, format!("{}{}", part, trimmed_end)))
}

// ---------------------------------------------------------------------------
// Non-delimited group: "-GroupName" at end of string or before space/dot
// ---------------------------------------------------------------------------

fn non_delimited_group(input: &str) -> IResult<&str, String> {
    let (rest, _) = char('-')(input)?;
    let (rest, name) = take_while1(|c: char| c != '.' && !is_whitespace(c) && !is_closing_delim(c))(rest)?;
    if name.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
    }
    Ok((rest, name.to_string()))
}

fn is_closing_delim(c: char) -> bool {
    DELIMITERS.iter().any(|(_, close)| *close == c)
}

fn is_opening_delim(c: char) -> bool {
    DELIMITERS.iter().any(|(open, _)| *open == c)
}

// ---------------------------------------------------------------------------
// Metadata token matchers - each returns a MetadataToken
// ---------------------------------------------------------------------------

fn parse_version(input: &str) -> IResult<&str, MetadataToken> {
    let (rest, v) = version_token(input)?;
    Ok((rest, MetadataToken { kind: MetadataKind::VersionTerm, value: v }))
}

fn parse_type_term(input: &str) -> IResult<&str, MetadataToken> {
    let (rest, v) = match_term_icase(TYPE_TERMS, input)?;
    Ok((rest, MetadataToken { kind: MetadataKind::TypeTerm, value: v.to_string() }))
}

fn parse_audio_codec(input: &str) -> IResult<&str, MetadataToken> {
    let (rest, v) = match_term_icase(AUDIO_CODECS, input)?;
    Ok((rest, MetadataToken { kind: MetadataKind::AudioCodec, value: v.to_string() }))
}

fn parse_audio_term(input: &str) -> IResult<&str, MetadataToken> {
    let (rest, v) = match_term_icase(AUDIO_TERMS, input)?;
    Ok((rest, MetadataToken { kind: MetadataKind::AudioTerm, value: v.to_string() }))
}

fn parse_container_term(input: &str) -> IResult<&str, MetadataToken> {
    let (rest, v) = match_term_icase(CONTAINER_TERMS, input)?;
    Ok((rest, MetadataToken { kind: MetadataKind::ContainerTerm, value: v.to_string() }))
}

fn parse_video_codec(input: &str) -> IResult<&str, MetadataToken> {
    let (rest, v) = match_term_icase(VIDEO_CODECS, input)?;
    Ok((rest, MetadataToken { kind: MetadataKind::VideoCodec, value: v.to_string() }))
}

fn parse_video_term(input: &str) -> IResult<&str, MetadataToken> {
    let (rest, v) = match_term_icase(VIDEO_TERMS, input)?;
    Ok((rest, MetadataToken { kind: MetadataKind::VideoTerm, value: v.to_string() }))
}

fn parse_resolution(input: &str) -> IResult<&str, MetadataToken> {
    let (rest, v) = resolution_token(input)?;
    Ok((rest, MetadataToken { kind: MetadataKind::Resolution, value: v }))
}

fn parse_batch_term(input: &str) -> IResult<&str, MetadataToken> {
    let (rest, v) = match_term_icase(BATCH_TERMS, input)?;
    Ok((rest, MetadataToken { kind: MetadataKind::BatchTerm, value: v.to_string() }))
}

fn parse_subtitle_term(input: &str) -> IResult<&str, MetadataToken> {
    let (rest, v) = match_term_icase(SUBTITLE_TERMS, input)?;
    Ok((rest, MetadataToken { kind: MetadataKind::SubtitleTerm, value: v.to_string() }))
}

fn parse_season_term(input: &str) -> IResult<&str, MetadataToken> {
    let (rest, v) = season_token(input)?;
    Ok((rest, MetadataToken { kind: MetadataKind::SeasonTerm, value: v }))
}

fn parse_subtitle_language(input: &str) -> IResult<&str, MetadataToken> {
    let codes = subtitle_language_codes();
    let (rest, v) = match_term_list_icase(&codes, input)?;
    Ok((rest, MetadataToken { kind: MetadataKind::SubtitleLanguage, value: v.to_string() }))
}

fn parse_audio_language(input: &str) -> IResult<&str, MetadataToken> {
    // Try audio language terms first (case insensitive) - these are longer
    if let Ok((rest, v)) = match_term_icase(AUDIO_LANGUAGE_TERMS, input) {
        return Ok((rest, MetadataToken { kind: MetadataKind::AudioLanguage, value: v.to_string() }));
    }
    // Try case-sensitive terms
    if let Ok((rest, v)) = match_term_exact(AUDIO_LANGUAGE_TERMS_CASE_SENSITIVE, input) {
        return Ok((rest, MetadataToken { kind: MetadataKind::AudioLanguage, value: v.to_string() }));
    }
    // Try language codes (case insensitive)
    let codes = language_codes();
    let (rest, v) = match_term_list_icase(&codes, input)?;
    Ok((rest, MetadataToken { kind: MetadataKind::AudioLanguage, value: v.to_string() }))
}

fn parse_source_term(input: &str) -> IResult<&str, MetadataToken> {
    let keys = source_term_keys();
    let (rest, v) = match_term_list_icase(&keys, input)?;
    Ok((rest, MetadataToken { kind: MetadataKind::SourceTerm, value: v.to_string() }))
}

fn parse_date(input: &str) -> IResult<&str, MetadataToken> {
    let (rest, v) = date_token(input)?;
    Ok((rest, MetadataToken { kind: MetadataKind::Date, value: v }))
}

fn parse_episode(input: &str) -> IResult<&str, MetadataToken> {
    let (rest, v) = episode_token(input)?;
    Ok((rest, MetadataToken { kind: MetadataKind::EpisodeTerm, value: v }))
}

fn parse_group_from_hyphen(input: &str) -> IResult<&str, MetadataToken> {
    let (rest, v) = non_delimited_group(input)?;
    Ok((rest, MetadataToken { kind: MetadataKind::Group, value: v }))
}

// ---------------------------------------------------------------------------
// Ordered metadata token attempt - matches the TypeScript ordering
// ---------------------------------------------------------------------------

/// The ordered list of metadata parsers tried inside delimited blocks.
/// Order matters: longer/more-specific patterns must come before shorter ones.
fn try_metadata_token(input: &str) -> IResult<&str, MetadataToken> {
    // Same order as the TypeScript metadataTokenValue array
    alt((
        parse_version,
        parse_type_term,
        parse_audio_codec,
        parse_audio_term,
        parse_container_term,
        parse_video_codec,
        parse_video_term,
        parse_resolution,
        parse_batch_term,
        parse_subtitle_term,
    ))(input)
    .or_else(|_: nom::Err<nom::error::Error<&str>>| {
        alt((
            parse_season_term,
            parse_subtitle_language,
            parse_audio_language,
            parse_source_term,
            parse_date,
        ))(input)
    })
}

// ---------------------------------------------------------------------------
// Delimited metadata block: [content], (content), {content}, etc.
// ---------------------------------------------------------------------------

fn parse_delimited_block(input: &str) -> IResult<&str, RootToken> {
    for &(open, close) in DELIMITERS {
        if let Ok((rest, tokens)) = parse_one_delimited(input, open, close) {
            return Ok((rest, tokens));
        }
    }
    Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)))
}

fn parse_one_delimited<'a>(input: &'a str, open: char, close: char) -> IResult<&'a str, RootToken> {
    let (rest, _) = char(open)(input)?;

    // Extract everything up to the matching close delimiter
    let close_pos = rest.find(close).ok_or_else(|| {
        nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag))
    })?;

    let inner = &rest[..close_pos];
    let after_close = &rest[close_pos + close.len_utf8()..];

    // Parse the inner content for metadata tokens
    let tokens = parse_inner_metadata(inner);

    Ok((after_close, RootToken::Metadata(tokens)))
}

/// Parse the content inside delimiters, extracting metadata tokens and skipping non-metadata text.
fn parse_inner_metadata(mut input: &str) -> Vec<MetadataToken> {
    let mut tokens = Vec::new();

    while !input.is_empty() {
        // Skip leading whitespace
        if let Ok((rest, _)) = ws(input) {
            input = rest;
            if input.is_empty() {
                break;
            }
        }

        // Try to match a metadata token
        if let Ok((rest, token)) = try_metadata_token(input) {
            // Check that the token is followed by whitespace, end of input, or closing delim
            // This prevents partial matches mid-word
            if rest.is_empty()
                || rest.starts_with(|c: char| is_whitespace(c))
                || rest.starts_with(|c: char| is_closing_delim(c))
            {
                tokens.push(token);
                input = rest;
                continue;
            }
        }

        // Skip one character and continue
        let mut chars = input.chars();
        chars.next();
        input = chars.as_str();
    }

    tokens
}

// ---------------------------------------------------------------------------
// Non-delimited metadata: "-GroupName", " WEB-DL", version, container at word boundary
// ---------------------------------------------------------------------------

/// Check if position is a valid token boundary: the current match can end here
/// because the next thing is whitespace, end, a delimiter, a hyphen (group start),
/// or another recognizable metadata token (e.g. version immediately after season "S01v2").
fn is_token_boundary(rest: &str) -> bool {
    if rest.is_empty() {
        return true;
    }
    let c = rest.chars().next().unwrap();
    if is_whitespace(c) || is_opening_delim(c) || is_closing_delim(c) {
        return true;
    }
    // '-' can start a group token
    if c == '-' {
        return true;
    }
    // Check if remaining starts with a recognizable metadata token
    if parse_version(rest).is_ok() || try_metadata_token(rest).is_ok() {
        return true;
    }
    false
}

fn parse_non_delimited_metadata(input: &str) -> IResult<&str, RootToken> {
    // 1. group from hyphen prefix (at start or after space)
    if let Ok((rest, tok)) = parse_group_from_hyphen(input) {
        return Ok((rest, RootToken::Metadata(vec![tok])));
    }

    // 2. whitespace-separated metadata term
    if let Ok((rest_ws, _)) = ws(input) {
        // Try metadata tokens
        if let Ok((rest, tok)) = try_metadata_token(rest_ws) {
            if is_token_boundary(rest) {
                return Ok((rest, RootToken::Metadata(vec![tok])));
            }
        }
        // Try episode token
        if let Ok((rest, tok)) = parse_episode(rest_ws) {
            if is_token_boundary(rest) {
                return Ok((rest, RootToken::Metadata(vec![tok])));
            }
        }
        // Try group from hyphen
        if let Ok((rest, tok)) = parse_group_from_hyphen(rest_ws) {
            return Ok((rest, RootToken::Metadata(vec![tok])));
        }
    }

    // 3. Non-separated tokens (version, container at start without space)
    if let Ok((rest, tok)) = parse_version(input) {
        return Ok((rest, RootToken::Metadata(vec![tok])));
    }
    if let Ok((rest, tok)) = parse_container_term(input) {
        if is_token_boundary(rest) {
            return Ok((rest, RootToken::Metadata(vec![tok])));
        }
    }

    Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)))
}

// ---------------------------------------------------------------------------
// Non-metadata (data) token: everything until a metadata token or end
// ---------------------------------------------------------------------------

fn parse_data_token(input: &str) -> IResult<&str, RootToken> {
    if input.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Eof)));
    }

    let mut pos = 0;

    while pos < input.len() {
        let remaining = &input[pos..];

        // Check if a metadata token starts here
        if parse_delimited_block(remaining).is_ok() || parse_non_delimited_metadata(remaining).is_ok() {
            break;
        }

        // Advance by one character
        let c = remaining.chars().next().unwrap();
        pos += c.len_utf8();
    }

    if pos == 0 {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
    }

    Ok((&input[pos..], RootToken::Data(input[..pos].to_string())))
}

// ---------------------------------------------------------------------------
// Top-level token parser
// ---------------------------------------------------------------------------

fn parse_token(input: &str) -> IResult<&str, RootToken> {
    alt((
        parse_delimited_block,
        parse_non_delimited_metadata,
        parse_data_token,
    ))(input)
}

fn parse_all_tokens(input: &str) -> IResult<&str, Vec<RootToken>> {
    many1(parse_token)(input)
}

// ---------------------------------------------------------------------------
// Post-processing: promote first single-string metadata to group
// ---------------------------------------------------------------------------

fn is_non_word(c: char) -> bool {
    c.is_ascii_punctuation() || c.is_whitespace() || c.is_control() || c.is_ascii_control()
        || matches!(c, '\u{00A0}'..='\u{00BF}') // symbols/punctuation block
}

fn trim_non_word(s: &str) -> String {
    let trimmed = s.trim();
    let start = trimmed.find(|c: char| !is_non_word(c)).unwrap_or(0);
    let end = trimmed.rfind(|c: char| !is_non_word(c)).map(|i| i + 1).unwrap_or(0);
    if start >= end {
        String::new()
    } else {
        trimmed[start..end].to_string()
    }
}

// ---------------------------------------------------------------------------
// ParseResult - the public output type
// ---------------------------------------------------------------------------

use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParseResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub titles: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub episode_terms: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_terms: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_codec_terms: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_codec_terms: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_language_terms: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groups: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_terms: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_terms: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle_terms: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_terms: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle_language_terms: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_terms: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_terms: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub season_terms: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_terms: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dates: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution_terms: Option<Vec<String>>,
}

impl ParseResult {
    fn push(&mut self, kind: &MetadataKind, value: String) {
        let field = match kind {
            MetadataKind::Group => &mut self.groups,
            MetadataKind::Title => &mut self.titles,
            MetadataKind::Resolution => &mut self.resolution_terms,
            MetadataKind::VideoCodec => &mut self.video_codec_terms,
            MetadataKind::AudioCodec => &mut self.audio_codec_terms,
            MetadataKind::AudioTerm => &mut self.audio_terms,
            MetadataKind::VideoTerm => &mut self.video_terms,
            MetadataKind::TypeTerm => &mut self.type_terms,
            MetadataKind::ContainerTerm => &mut self.container_terms,
            MetadataKind::SubtitleTerm => &mut self.subtitle_terms,
            MetadataKind::BatchTerm => &mut self.batch_terms,
            MetadataKind::SeasonTerm => &mut self.season_terms,
            MetadataKind::EpisodeTerm => &mut self.episode_terms,
            MetadataKind::AudioLanguage => &mut self.audio_language_terms,
            MetadataKind::SubtitleLanguage => &mut self.subtitle_language_terms,
            MetadataKind::SourceTerm => &mut self.source_terms,
            MetadataKind::Date => &mut self.dates,
            MetadataKind::VersionTerm => &mut self.version_terms,
        };
        field.get_or_insert_with(Vec::new).push(value);
    }
}

// ---------------------------------------------------------------------------
// Public parse function
// ---------------------------------------------------------------------------

pub fn parse(input: &str) -> Result<ParseResult, String> {
    let (_, tokens) = parse_all_tokens(input)
        .map_err(|e| format!("Parser errored: {:?}", e))?;

    let mut result = ParseResult::default();

    // Process tokens, mirroring the TypeScript logic:
    // - First metadata block with a single plain string → promote to group
    // - DATA tokens → titles (trimmed, filtered)
    // - METADATA tokens → categorized

    let mut first = true;
    for token in &tokens {
        match token {
            RootToken::Metadata(items) => {
                if first && items.is_empty() {
                    // First token was a delimited block with no recognized metadata
                    // The original TS code promotes this to a group when exactly 1 string
                    // We need to handle the case where the inner content was just a plain string
                    // This is handled below via the special first-token logic
                }

                if first {
                    first = false;
                    // Special case: if first metadata block is empty (all content was
                    // unrecognized strings inside brackets), extract the raw text as group
                    if items.is_empty() {
                        // Reconstruct from input: find the first delimited block content
                        if let Some(group_text) = extract_first_bracket_content(input) {
                            let trimmed = group_text.trim().to_string();
                            if !trimmed.is_empty() {
                                result.push(&MetadataKind::Group, trimmed);
                            }
                        }
                        continue;
                    }
                }

                for item in items {
                    result.push(&item.kind, item.value.clone());
                }
            }
            RootToken::Data(text) => {
                first = false;
                let trimmed = trim_non_word(text);
                // Filter: must have at least 1 non-punctuation/non-whitespace char
                let has_word = trimmed.chars().any(|c| !is_non_word(c));
                if !trimmed.is_empty() && has_word {
                    result.push(&MetadataKind::Title, trimmed);
                }
            }
        }
    }

    Ok(result)
}

/// Extract the content of the first bracketed section from the raw input.
fn extract_first_bracket_content(input: &str) -> Option<String> {
    for &(open, close) in DELIMITERS {
        if input.starts_with(open) {
            let rest = &input[open.len_utf8()..];
            if let Some(end) = rest.find(close) {
                return Some(rest[..end].to_string());
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_silly_cyberpunk() {
        let r = parse("[silly] Cyberpunk Edgerunners (WEB-DL 1080p HEVC E-AC-3) [Dual-Audio]").unwrap();
        assert_eq!(r.groups.as_deref(), Some(&["silly".to_string()][..]));
        assert_eq!(r.titles.as_deref(), Some(&["Cyberpunk Edgerunners".to_string()][..]));
        assert_eq!(r.resolution_terms.as_deref(), Some(&["1080p".to_string()][..]));
        assert_eq!(r.video_codec_terms.as_deref(), Some(&["HEVC".to_string()][..]));
        assert_eq!(r.audio_codec_terms.as_deref(), Some(&["E-AC-3".to_string()][..]));
        assert_eq!(r.audio_terms.as_deref(), Some(&["Dual-Audio".to_string()][..]));
        assert_eq!(r.type_terms.as_deref(), Some(&["WEB-DL".to_string()][..]));
    }

    #[test]
    fn test_dkb_cyberpunk() {
        let r = parse("[DKB] Cyberpunk Edgerunners - Season 01 [1080p][HEVC x265 10bit][Dual-Audio][Multi-Subs][batch]").unwrap();
        assert_eq!(r.groups.as_deref(), Some(&["DKB".to_string()][..]));
        assert_eq!(r.titles.as_deref(), Some(&["Cyberpunk Edgerunners".to_string()][..]));
        assert_eq!(r.resolution_terms.as_deref(), Some(&["1080p".to_string()][..]));
        assert_eq!(r.video_codec_terms.as_deref(), Some(&["HEVC".to_string(), "x265".to_string(), "10bit".to_string()][..]));
        assert_eq!(r.audio_terms.as_deref(), Some(&["Dual-Audio".to_string()][..]));
        assert_eq!(r.subtitle_terms.as_deref(), Some(&["Multi-Subs".to_string()][..]));
        assert_eq!(r.season_terms.as_deref(), Some(&["Season 01".to_string()][..]));
        assert_eq!(r.batch_terms.as_deref(), Some(&["batch".to_string()][..]));
    }

    #[test]
    fn test_ember_cyberpunk() {
        let r = parse("[EMBER] Cyberpunk: Edgerunners (2022) (Season 1) [WEBRip] [1080p Dual Audio HEVC 10 bits] (Cyberpunk Edgerunners) (Batch)").unwrap();
        assert_eq!(r.groups.as_deref(), Some(&["EMBER".to_string()][..]));
        assert_eq!(r.dates.as_deref(), Some(&["2022".to_string()][..]));
        assert_eq!(r.titles.as_deref(), Some(&["Cyberpunk: Edgerunners".to_string()][..]));
        assert_eq!(r.resolution_terms.as_deref(), Some(&["1080p".to_string()][..]));
        assert_eq!(r.video_codec_terms.as_deref(), Some(&["HEVC".to_string(), "10 bits".to_string()][..]));
        assert_eq!(r.audio_terms.as_deref(), Some(&["Dual Audio".to_string()][..]));
        assert_eq!(r.season_terms.as_deref(), Some(&["Season 1".to_string()][..]));
        assert_eq!(r.batch_terms.as_deref(), Some(&["Batch".to_string()][..]));
        assert_eq!(r.type_terms.as_deref(), Some(&["WEBRip".to_string()][..]));
    }

    #[test]
    fn test_no_delimiters() {
        let r = parse("Cyberpunk Edgerunners WEB-DL 1080P HDR DV EAC3 VF VOSTFR-LTPD v2").unwrap();
        assert_eq!(r.groups.as_deref(), Some(&["LTPD".to_string()][..]));
        assert_eq!(r.titles.as_deref(), Some(&["Cyberpunk Edgerunners".to_string()][..]));
        assert_eq!(r.resolution_terms.as_deref(), Some(&["1080P".to_string()][..]));
        assert_eq!(r.video_terms.as_deref(), Some(&["HDR".to_string(), "DV".to_string()][..]));
        assert_eq!(r.audio_codec_terms.as_deref(), Some(&["EAC3".to_string()][..]));
        assert_eq!(r.audio_language_terms.as_deref(), Some(&["VF".to_string()][..]));
        assert_eq!(r.subtitle_language_terms.as_deref(), Some(&["VOSTFR".to_string()][..]));
        assert_eq!(r.type_terms.as_deref(), Some(&["WEB-DL".to_string()][..]));
        assert_eq!(r.version_terms.as_deref(), Some(&["v2".to_string()][..]));
    }

    #[test]
    fn test_multi_lang() {
        let r = parse("[Erai-raws] Cyberpunk - Edgerunners - 01 ~ 10 [1080p][Multiple Subtitle] [ENG][POR-BR][SPA-LA][SPA][ARA][FRE][GER][ITA][JPN][POR][POL][DUT][NOB][FIN][TUR][SWE][GRE][HEB][RUM][IND][THA][KOR][DAN][CHI][VIE][UKR][HUN][CES][HRV][MAY][FIL]").unwrap();
        assert_eq!(r.groups.as_deref(), Some(&["Erai-raws".to_string()][..]));
        assert_eq!(r.titles.as_deref(), Some(&["Cyberpunk - Edgerunners".to_string()][..]));
        assert_eq!(r.episode_terms.as_deref(), Some(&["01 ~ 10".to_string()][..]));
        assert_eq!(r.resolution_terms.as_deref(), Some(&["1080p".to_string()][..]));
        assert_eq!(r.subtitle_terms.as_deref(), Some(&["Multiple Subtitle".to_string()][..]));
        let langs = r.audio_language_terms.as_ref().unwrap();
        assert_eq!(langs.len(), 31);
        assert_eq!(langs[0], "ENG");
        assert_eq!(langs[1], "POR-BR");
        assert_eq!(langs[30], "FIL");
    }

    #[test]
    fn test_season_s01v2() {
        let r = parse("Cyberpunk - Edgerunners - S01v2 - MULTi 1080p WEB DV H.265 -NanDesuKa (NF)").unwrap();
        assert_eq!(r.groups.as_deref(), Some(&["NanDesuKa".to_string()][..]));
        assert_eq!(r.titles.as_deref(), Some(&["Cyberpunk - Edgerunners".to_string()][..]));
        assert_eq!(r.resolution_terms.as_deref(), Some(&["1080p".to_string()][..]));
        assert_eq!(r.video_codec_terms.as_deref(), Some(&["H.265".to_string()][..]));
        assert_eq!(r.video_terms.as_deref(), Some(&["DV".to_string()][..]));
        assert_eq!(r.audio_language_terms.as_deref(), Some(&["MULTi".to_string()][..]));
        assert_eq!(r.season_terms.as_deref(), Some(&["S01".to_string()][..]));
        assert_eq!(r.type_terms.as_deref(), Some(&["WEB".to_string()][..]));
        assert_eq!(r.version_terms.as_deref(), Some(&["v2".to_string()][..]));
        assert_eq!(r.source_terms.as_deref(), Some(&["NF".to_string()][..]));
    }

    #[test]
    fn test_whomst() {
        let r = parse("[whomst] Cyberpunk: Edgerunners (subs only)").unwrap();
        assert_eq!(r.groups.as_deref(), Some(&["whomst".to_string()][..]));
        assert_eq!(r.titles.as_deref(), Some(&["Cyberpunk: Edgerunners".to_string()][..]));
    }

    #[test]
    fn test_trix_1080() {
        let r = parse("[Trix] Cyberpunk: Edgerunners (2022) [Optional Dual Audio] [Multi Subs] (1080p AV1)").unwrap();
        assert_eq!(r.groups.as_deref(), Some(&["Trix".to_string()][..]));
        assert_eq!(r.dates.as_deref(), Some(&["2022".to_string()][..]));
        assert_eq!(r.titles.as_deref(), Some(&["Cyberpunk: Edgerunners".to_string()][..]));
        assert_eq!(r.resolution_terms.as_deref(), Some(&["1080p".to_string()][..]));
        assert_eq!(r.video_codec_terms.as_deref(), Some(&["AV1".to_string()][..]));
        assert_eq!(r.audio_language_terms.as_deref(), Some(&["Optional Dual Audio".to_string()][..]));
        assert_eq!(r.subtitle_terms.as_deref(), Some(&["Multi Subs".to_string()][..]));
    }

    #[test]
    fn test_trix_720() {
        let r = parse("[Trix] Cyberpunk: Edgerunners (2022) [Optional Dual Audio] [Multi Subs] (720p AV1)").unwrap();
        assert_eq!(r.groups.as_deref(), Some(&["Trix".to_string()][..]));
        assert_eq!(r.resolution_terms.as_deref(), Some(&["720p".to_string()][..]));
    }
}
