#![allow(dead_code)]
use crate::combinators::*;
use crate::common::*;

/// Delimiter pairs for metadata blocks.
const DELIMITERS: &[(char, char)] = &[
    ('(', ')'),
    ('[', ']'),
    ('{', '}'),
    ('\u{300C}', '\u{300D}'), // 「」 Corner bracket
    ('\u{300E}', '\u{300F}'), // 『』 White corner bracket
    ('\u{3010}', '\u{3011}'), // 【】 Black lenticular bracket
    ('\u{FF08}', '\u{FF09}'), // （） Fullwidth parenthesis
];

fn closing_delimiter(open: char) -> Option<char> {
    DELIMITERS.iter().find(|(o, _)| *o == open).map(|(_, c)| *c)
}

fn is_opening_delimiter(c: char) -> bool {
    DELIMITERS.iter().any(|(o, _)| *o == c)
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Groups,
    Titles,
    VersionTerms,
    TypeTerms,
    AudioCodecTerms,
    AudioTerms,
    ContainerTerms,
    VideoCodecTerms,
    VideoTerms,
    ResolutionTerms,
    BatchTerms,
    SubtitleTerms,
    SeasonTerms,
    SubtitleLanguageTerms,
    AudioLanguageTerms,
    SourceTerms,
    Dates,
    EpisodeTerms,
}

#[derive(Debug, Clone)]
struct Token {
    token_type: TokenType,
    value: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ParseResult {
    pub titles: Option<Vec<String>>,
    pub groups: Option<Vec<String>>,
    pub episode_terms: Option<Vec<String>>,
    pub container_terms: Option<Vec<String>>,
    pub video_codec_terms: Option<Vec<String>>,
    pub audio_codec_terms: Option<Vec<String>>,
    pub audio_language_terms: Option<Vec<String>>,
    pub version_terms: Option<Vec<String>>,
    pub batch_terms: Option<Vec<String>>,
    pub subtitle_terms: Option<Vec<String>>,
    pub type_terms: Option<Vec<String>>,
    pub subtitle_language_terms: Option<Vec<String>>,
    pub audio_terms: Option<Vec<String>>,
    pub video_terms: Option<Vec<String>>,
    pub season_terms: Option<Vec<String>>,
    pub source_terms: Option<Vec<String>>,
    pub dates: Option<Vec<String>>,
    pub resolution_terms: Option<Vec<String>>,
}

fn push_to(opt: &mut Option<Vec<String>>, val: String) {
    match opt {
        Some(v) => v.push(val),
        None => *opt = Some(vec![val]),
    }
}

// ── Individual metadata token parsers ──

/// Parse version token: v followed by digit (e.g. "v2")
fn parse_version_token<'a>(state: ParseState<'a>) -> crate::combinators::ParseResult<'a, Token> {
    let (_, s) = ichar('v', state)?;
    let (d, s) = digit(s)?;
    let value = format!("v{}", d);
    Ok((Token { token_type: TokenType::VersionTerms, value }, s))
}

/// Parse type terms: WEB-DL, WEBRip, WEB
fn parse_type_term<'a>(state: ParseState<'a>) -> crate::combinators::ParseResult<'a, Token> {
    let (matched, s) = choice_istr(TYPE_TERMS, state)?;
    Ok((Token { token_type: TokenType::TypeTerms, value: matched.to_string() }, s))
}

/// Parse audio codec
fn parse_audio_codec<'a>(state: ParseState<'a>) -> crate::combinators::ParseResult<'a, Token> {
    let (matched, s) = choice_istr(AUDIO_CODECS, state)?;
    Ok((Token { token_type: TokenType::AudioCodecTerms, value: matched.to_string() }, s))
}

/// Parse audio term
fn parse_audio_term<'a>(state: ParseState<'a>) -> crate::combinators::ParseResult<'a, Token> {
    let (matched, s) = choice_istr(AUDIO_TERMS, state)?;
    Ok((Token { token_type: TokenType::AudioTerms, value: matched.to_string() }, s))
}

/// Parse container term
fn parse_container_term<'a>(state: ParseState<'a>) -> crate::combinators::ParseResult<'a, Token> {
    let (matched, s) = choice_istr(CONTAINER_TERMS, state)?;
    Ok((Token { token_type: TokenType::ContainerTerms, value: matched.to_string() }, s))
}

/// Parse video codec
fn parse_video_codec<'a>(state: ParseState<'a>) -> crate::combinators::ParseResult<'a, Token> {
    let (matched, s) = choice_istr(VIDEO_CODECS, state)?;
    Ok((Token { token_type: TokenType::VideoCodecTerms, value: matched.to_string() }, s))
}

/// Parse video term
fn parse_video_term<'a>(state: ParseState<'a>) -> crate::combinators::ParseResult<'a, Token> {
    let (matched, s) = choice_istr(VIDEO_TERMS, state)?;
    Ok((Token { token_type: TokenType::VideoTerms, value: matched.to_string() }, s))
}

/// Parse resolution: number + optional p/px, optional x + number + p/px
fn parse_resolution<'a>(state: ParseState<'a>) -> crate::combinators::ParseResult<'a, Token> {
    // Try each known resolution number
    let mut best: Option<(String, ParseState<'a>)> = None;

    for &res in RESOLUTIONS {
        let res_str = res.to_string();
        if let Ok((matched, s)) = istr(&res_str, state.clone()) {
            let mut val = matched.to_string();
            let mut s = s;

            // Try "px" suffix
            if let Ok((sfx, s2)) = istr("px", s.clone()) {
                val.push_str(sfx);
                s = s2;
            } else if let Ok((sfx, s2)) = ichar('p', s.clone()) {
                val.push(sfx);
                s = s2;
            }

            // Try "x" + resolution
            if let Ok((_, sx)) = ichar('x', s.clone()) {
                for &res2 in RESOLUTIONS {
                    let res2_str = res2.to_string();
                    if let Ok((m2, s2)) = istr(&res2_str, sx.clone()) {
                        let mut val2 = format!("{}x{}", val, m2);
                        let mut s2 = s2;
                        if let Ok((sfx, s3)) = istr("px", s2.clone()) {
                            val2.push_str(sfx);
                            s2 = s3;
                        } else if let Ok((sfx, s3)) = ichar('p', s2.clone()) {
                            val2.push(sfx);
                            s2 = s3;
                        }
                        // Prefer longest match
                        if best.as_ref().map_or(true, |(v, _)| val2.len() > v.len()) {
                            best = Some((val2, s2));
                        }
                    }
                }
            }

            if best.as_ref().map_or(true, |(v, _)| val.len() > v.len()) {
                best = Some((val, s));
            }
        }
    }

    if let Some((value, s)) = best {
        Ok((Token { token_type: TokenType::ResolutionTerms, value }, s))
    } else {
        Err(state)
    }
}

/// Parse batch term
fn parse_batch_term<'a>(state: ParseState<'a>) -> crate::combinators::ParseResult<'a, Token> {
    let (matched, s) = choice_istr(BATCH_TERMS, state)?;
    Ok((Token { token_type: TokenType::BatchTerms, value: matched.to_string() }, s))
}

/// Parse subtitle term
fn parse_subtitle_term<'a>(state: ParseState<'a>) -> crate::combinators::ParseResult<'a, Token> {
    let (matched, s) = choice_istr(SUBTITLE_TERMS, state)?;
    Ok((Token { token_type: TokenType::SubtitleTerms, value: matched.to_string() }, s))
}

/// Parse season term: "Season" or "S" followed by digits (and optional whitespace between)
fn parse_season_term<'a>(state: ParseState<'a>) -> crate::combinators::ParseResult<'a, Token> {
    let (part, s) = choice_istr(SEASON_PART_TERMS, state)?;
    let mut val = part.to_string();
    let mut s = s;

    // optional whitespace
    if let Ok((ws, s2)) = skip_whitespace(s.clone()) {
        if !ws.is_empty() {
            val.push_str(ws);
        }
        s = s2;
    }

    // Must have at least one digit
    let (digs, s) = digits(s)?;
    val.push_str(digs);

    // Consume trailing digits/whitespace
    let mut s = s;
    loop {
        if let Ok((c, s2)) = digit(s.clone()) {
            val.push(c);
            s = s2;
        } else if let Ok((_, s2)) = whitespace(s.clone()) {
            // Only consume whitespace if followed by a digit
            if let Ok((_, _)) = digit(s2.clone()) {
                val.push(' ');
                s = s2;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    Ok((Token { token_type: TokenType::SeasonTerms, value: val }, s))
}

/// Parse subtitle language term
fn parse_subtitle_language<'a>(state: ParseState<'a>) -> crate::combinators::ParseResult<'a, Token> {
    let (matched, s) = choice_istr(SUBTITLE_LANGUAGE_TERMS, state)?;
    Ok((Token { token_type: TokenType::SubtitleLanguageTerms, value: matched.to_string() }, s))
}

/// Parse audio language term (case-insensitive keys + case-sensitive)
fn parse_audio_language<'a>(state: ParseState<'a>) -> crate::combinators::ParseResult<'a, Token> {
    // Try case-insensitive language terms first (longer)
    if let Ok((matched, s)) = choice_istr(AUDIO_LANGUAGE_TERMS, state.clone()) {
        return Ok((Token { token_type: TokenType::AudioLanguageTerms, value: matched.to_string() }, s));
    }
    // Try case-sensitive
    if let Ok((matched, s)) = choice_str(AUDIO_LANGUAGE_TERMS_CASE_SENSITIVE, state.clone()) {
        return Ok((Token { token_type: TokenType::AudioLanguageTerms, value: matched.to_string() }, s));
    }
    // Try normalized language codes
    let (matched, s) = choice_istr(NORMALIZED_LANGUAGES, state)?;
    Ok((Token { token_type: TokenType::AudioLanguageTerms, value: matched.to_string() }, s))
}

/// Parse source term
fn parse_source_term<'a>(state: ParseState<'a>) -> crate::combinators::ParseResult<'a, Token> {
    let source_keys: Vec<&str> = SOURCE_TERMS.iter().map(|(k, _)| *k).collect();
    let (matched, s) = choice_istr(&source_keys, state)?;
    Ok((Token { token_type: TokenType::SourceTerms, value: matched.to_string() }, s))
}

/// Parse date: YYYY.MM.DD or DD.MM.YYYY or YYYY (4 digits)
fn parse_date<'a>(state: ParseState<'a>) -> crate::combinators::ParseResult<'a, Token> {
    let (d1, s) = digits(state.clone())?;
    if d1.len() == 4 {
        // Could be YYYY.MM.DD
        if let Ok((_, s2)) = char_exact('.', s.clone()) {
            if let Ok((d2, s3)) = digits(s2) {
                if d2.len() == 2 {
                    if let Ok((_, s4)) = char_exact('.', s3) {
                        if let Ok((d3, s5)) = digits(s4) {
                            if d3.len() == 2 {
                                let val = format!("{}.{}.{}", d1, d2, d3);
                                return Ok((Token { token_type: TokenType::Dates, value: val }, s5));
                            }
                        }
                    }
                }
            }
        }
        // Just YYYY
        return Ok((Token { token_type: TokenType::Dates, value: d1.to_string() }, s));
    }
    if d1.len() == 2 {
        // DD.MM.YYYY
        if let Ok((_, s2)) = char_exact('.', s) {
            if let Ok((d2, s3)) = digits(s2) {
                if d2.len() == 2 {
                    if let Ok((_, s4)) = char_exact('.', s3) {
                        if let Ok((d3, s5)) = digits(s4) {
                            if d3.len() == 4 {
                                let val = format!("{}.{}.{}", d1, d2, d3);
                                return Ok((Token { token_type: TokenType::Dates, value: val }, s5));
                            }
                        }
                    }
                }
            }
        }
    }
    Err(state)
}

/// Parse episode range: digits ~ digits or digits - digits
fn parse_episode_range<'a>(state: ParseState<'a>) -> crate::combinators::ParseResult<'a, Token> {
    let (d1, s) = digits(state.clone())?;
    let (_, s) = skip_whitespace(s)?;
    let (sep, s) = {
        if let Ok(r) = char_exact('~', s.clone()) {
            Ok(r)
        } else {
            char_exact('-', s.clone())
        }
    }?;
    let (_, s) = skip_whitespace(s)?;
    let (d2, s) = digits(s)?;
    let val = format!("{} {} {}", d1, sep, d2);
    Ok((Token { token_type: TokenType::EpisodeTerms, value: val }, s))
}

/// All metadata token parsers in priority order (matching the TS implementation).
/// Order matters: longer/more specific tokens first.
type MetadataParser = for<'a> fn(ParseState<'a>) -> crate::combinators::ParseResult<'a, Token>;

const METADATA_PARSERS: &[MetadataParser] = &[
    parse_version_token,
    parse_type_term,
    parse_audio_codec,
    parse_audio_term,
    parse_container_term,
    parse_video_codec,
    parse_video_term,
    parse_resolution,
    parse_batch_term,
    parse_subtitle_term,
    parse_season_term,
    parse_subtitle_language,
    parse_audio_language,
    parse_source_term,
    parse_date,
];

fn try_metadata_token<'a>(state: ParseState<'a>) -> crate::combinators::ParseResult<'a, Token> {
    for parser in METADATA_PARSERS {
        if let Ok(result) = parser(state.clone()) {
            return Ok(result);
        }
    }
    Err(state)
}

/// Check if the next character is whitespace or a closing delimiter.
fn at_boundary(close: char, state: &ParseState) -> bool {
    let rem = state.remaining();
    if rem.is_empty() {
        return true;
    }
    if let Some(c) = rem.chars().next() {
        return c.is_whitespace() || c == close;
    }
    true
}

/// Parse metadata tokens inside a delimited block.
/// Returns a list of recognized tokens (unrecognized content is dropped).
fn parse_delimited_content<'a>(close: char, state: ParseState<'a>) -> (Vec<Token>, ParseState<'a>) {
    let mut tokens = Vec::new();
    let mut s = state;

    loop {
        // Stop at closing delimiter or end of input
        if s.is_empty() {
            break;
        }
        if let Some(c) = s.remaining().chars().next() {
            if c == close {
                break;
            }
        }

        // Skip whitespace
        if let Ok((_, s2)) = whitespace1(s.clone()) {
            s = s2;
            continue;
        }

        // Try a metadata token
        if let Ok((token, s2)) = try_metadata_token(s.clone()) {
            // Verify boundary: next char must be whitespace or close delimiter
            if at_boundary(close, &s2) {
                tokens.push(token);
                s = s2;
                continue;
            }
        }

        // Skip one character (unrecognized)
        if let Ok((_, s2)) = any_char(s.clone()) {
            s = s2;
        } else {
            break;
        }
    }

    (tokens, s)
}

/// Parse a delimited metadata block: [content] or (content) etc.
fn parse_delimited_metadata<'a>(state: ParseState<'a>) -> crate::combinators::ParseResult<'a, (Vec<Token>, bool)> {
    let rem = state.remaining();
    let open = rem.chars().next().ok_or_else(|| state.clone())?;
    let close = closing_delimiter(open).ok_or_else(|| state.clone())?;

    // Consume opening delimiter
    let s = state.advance(open.len_utf8());

    // Parse content
    let (tokens, s) = parse_delimited_content(close, s);

    // Consume closing delimiter
    let (_, s) = char_exact(close, s)?;

    // If no metadata tokens found, the content might be a group name or title-ish
    let is_metadata = !tokens.is_empty();
    Ok(((tokens, is_metadata), s))
}

/// Parse a non-delimited group: "-GroupName" at word boundary
fn parse_non_delimited_group<'a>(state: ParseState<'a>) -> crate::combinators::ParseResult<'a, Token> {
    let (_, s) = char_exact('-', state)?;
    let rem = s.remaining();
    let byte_len: usize = rem.chars()
        .take_while(|c| !c.is_whitespace() && *c != '.')
        .map(|c| c.len_utf8())
        .sum();
    if byte_len == 0 {
        return Err(s);
    }
    let matched = &s.input[s.pos..s.pos + byte_len];
    let s = s.advance(byte_len);
    Ok((Token { token_type: TokenType::Groups, value: matched.to_string() }, s))
}

/// Non-word characters regex equivalent for trimming titles
fn is_non_word(c: char) -> bool {
    c.is_whitespace()
        || c.is_ascii_punctuation()
        || c.is_control()
        || matches!(c, '\u{00A0}'..='\u{00BF}')
}

fn trim_non_word(s: &str) -> &str {
    let s = s.trim();
    let s = s.trim_matches(|c: char| is_non_word(c) && c != ':' && c != '!' && c != '?');
    s.trim()
}

// ── Main parser ──

enum RootSegment {
    Delimited { tokens: Vec<Token>, is_metadata: bool, raw: String },
    NonDelimited { tokens: Vec<Token> },
    Data(String),
}

pub fn parse(input: &str) -> ParseResult {
    let mut state = ParseState::new(input);
    let mut segments: Vec<RootSegment> = Vec::new();

    while !state.is_empty() {
        // Try delimited metadata
        if is_opening_delimiter(state.remaining().chars().next().unwrap_or('\0')) {
            let start_pos = state.pos;
            if let Ok(((tokens, is_metadata), s)) = parse_delimited_metadata(state.clone()) {
                let raw = input[start_pos + 1..s.pos.saturating_sub(1)].to_string();
                segments.push(RootSegment::Delimited { tokens, is_metadata, raw });
                state = s;
                continue;
            }
        }

        // Try non-delimited metadata tokens (preceded by whitespace)
        if state.remaining().starts_with(' ') || state.remaining().starts_with('\t') {
            let pre_ws = state.clone();
            if let Ok((_, s_after_ws)) = whitespace1(state.clone()) {
                // Try episode range first
                if let Ok((tok, s2)) = parse_episode_range(s_after_ws.clone()) {
                    // Verify boundary
                    if s2.is_empty() || s2.remaining().chars().next().map_or(true, |c| c.is_whitespace() || is_opening_delimiter(c)) {
                        segments.push(RootSegment::NonDelimited { tokens: vec![tok] });
                        state = s2;
                        continue;
                    }
                }
                // Try metadata token after whitespace
                if let Ok((tok, s2)) = try_metadata_token(s_after_ws.clone()) {
                    // Verify it ends at a boundary
                    let next_c = s2.remaining().chars().next();
                    if s2.is_empty() || next_c.map_or(true, |c| c.is_whitespace() || is_opening_delimiter(c)) {
                        segments.push(RootSegment::NonDelimited { tokens: vec![tok] });
                        state = s2;
                        continue;
                    }
                    // If followed by `-`, the metadata token is valid and what follows is a group
                    if next_c == Some('-') {
                        let mut toks = vec![tok];
                        if let Ok((group_tok, s3)) = parse_non_delimited_group(s2.clone()) {
                            if s3.is_empty() || s3.remaining().chars().next().map_or(true, |c2| c2.is_whitespace() || is_opening_delimiter(c2)) {
                                toks.push(group_tok);
                                segments.push(RootSegment::NonDelimited { tokens: toks });
                                state = s3;
                                continue;
                            }
                        }
                        // Metadata token at boundary before dash even if group didn't parse
                        segments.push(RootSegment::NonDelimited { tokens: toks });
                        state = s2;
                        continue;
                    }
                    // Check for concatenated tokens: e.g., S01v2 -> season S01 + version v2
                    if let Ok((tok2, s3)) = try_metadata_token(s2.clone()) {
                        let next_c2 = s3.remaining().chars().next();
                        if s3.is_empty() || next_c2.map_or(true, |c| c.is_whitespace() || is_opening_delimiter(c)) {
                            segments.push(RootSegment::NonDelimited { tokens: vec![tok, tok2] });
                            state = s3;
                            continue;
                        }
                        // Could also be followed by group
                        if next_c2 == Some('-') {
                            let mut toks = vec![tok, tok2];
                            if let Ok((group_tok, s4)) = parse_non_delimited_group(s3.clone()) {
                                if s4.is_empty() || s4.remaining().chars().next().map_or(true, |c2| c2.is_whitespace() || is_opening_delimiter(c2)) {
                                    toks.push(group_tok);
                                    segments.push(RootSegment::NonDelimited { tokens: toks });
                                    state = s4;
                                    continue;
                                }
                            }
                        }
                    }
                }
                // Try non-delimited group: " -GroupName"
                if let Ok((tok, s2)) = parse_non_delimited_group(s_after_ws.clone()) {
                    if s2.is_empty() || s2.remaining().chars().next().map_or(true, |c| c.is_whitespace() || is_opening_delimiter(c)) {
                        segments.push(RootSegment::NonDelimited { tokens: vec![tok] });
                        state = s2;
                        continue;
                    }
                }
            }
            // Fall through: this whitespace + chars is part of a data segment
            let _ = pre_ws;
        }

        // Try non-delimited group at start of string or after delimiter
        if let Ok((tok, s2)) = parse_non_delimited_group(state.clone()) {
            if s2.is_empty() || s2.remaining().chars().next().map_or(true, |c| c.is_whitespace() || is_opening_delimiter(c)) {
                segments.push(RootSegment::NonDelimited { tokens: vec![tok] });
                state = s2;
                continue;
            }
        }

        // Collect data characters until next delimiter or metadata-like pattern
        let start = state.pos;
        // Advance at least one character
        if let Ok((_, s2)) = any_char(state.clone()) {
            state = s2;
        } else {
            break;
        }
        // Keep consuming until we hit a delimiter or end
        loop {
            if state.is_empty() {
                break;
            }
            let c = state.remaining().chars().next().unwrap();
            if is_opening_delimiter(c) {
                break;
            }

            // Check if whitespace followed by metadata token or group
            if c.is_whitespace() {
                if let Ok((_, s_after_ws)) = whitespace1(state.clone()) {
                    // Check for episode range
                    if let Ok((_, s2)) = parse_episode_range(s_after_ws.clone()) {
                        if s2.is_empty() || s2.remaining().chars().next().map_or(true, |ch| ch.is_whitespace() || is_opening_delimiter(ch)) {
                            break;
                        }
                    }
                    // Check for metadata token
                    if let Ok((_, s2)) = try_metadata_token(s_after_ws.clone()) {
                        let next_ch = s2.remaining().chars().next();
                        if s2.is_empty() || next_ch.map_or(true, |ch| ch.is_whitespace() || is_opening_delimiter(ch)) {
                            break;
                        }
                        // Metadata followed by `-` (group) or another token (e.g., S01v2)
                        if next_ch == Some('-') {
                            break;
                        }
                        if let Ok((_, s3)) = try_metadata_token(s2.clone()) {
                            let next_ch2 = s3.remaining().chars().next();
                            if s3.is_empty() || next_ch2.map_or(true, |ch| ch.is_whitespace() || is_opening_delimiter(ch) || ch == '-') {
                                break;
                            }
                        }
                    }
                    // Check for group
                    if let Ok((_, s2)) = parse_non_delimited_group(s_after_ws.clone()) {
                        if s2.is_empty() || s2.remaining().chars().next().map_or(true, |ch| ch.is_whitespace() || is_opening_delimiter(ch)) {
                            break;
                        }
                    }
                }
            }

            if let Ok((_, s2)) = any_char(state.clone()) {
                state = s2;
            } else {
                break;
            }
        }
        let data = &input[start..state.pos];
        if !data.is_empty() {
            segments.push(RootSegment::Data(data.to_string()));
        }
    }

    // Process segments into result
    let mut result = ParseResult::default();
    let mut is_first = true;

    for segment in segments {
        match segment {
            RootSegment::Delimited { tokens, is_metadata, raw } => {
                if is_metadata && !tokens.is_empty() {
                    for token in tokens {
                        add_token(&mut result, token);
                    }
                } else if is_first {
                    // First delimited block with no metadata = group
                    let trimmed = raw.trim().to_string();
                    if !trimmed.is_empty() {
                        push_to(&mut result.groups, trimmed);
                    }
                }
                // Non-metadata delimited blocks after first are ignored (they become titles via data)
                is_first = false;
            }
            RootSegment::NonDelimited { tokens } => {
                for token in tokens {
                    add_token(&mut result, token);
                }
                is_first = false;
            }
            RootSegment::Data(data) => {
                let trimmed = trim_non_word(&data);
                if !trimmed.is_empty() && trimmed.chars().any(|c| c.is_alphanumeric()) {
                    push_to(&mut result.titles, trimmed.to_string());
                }
                is_first = false;
            }
        }
    }

    result
}

fn add_token(result: &mut ParseResult, token: Token) {
    let val = token.value;
    match token.token_type {
        TokenType::Groups => push_to(&mut result.groups, val),
        TokenType::Titles => push_to(&mut result.titles, val),
        TokenType::VersionTerms => push_to(&mut result.version_terms, val),
        TokenType::TypeTerms => push_to(&mut result.type_terms, val),
        TokenType::AudioCodecTerms => push_to(&mut result.audio_codec_terms, val),
        TokenType::AudioTerms => push_to(&mut result.audio_terms, val),
        TokenType::ContainerTerms => push_to(&mut result.container_terms, val),
        TokenType::VideoCodecTerms => push_to(&mut result.video_codec_terms, val),
        TokenType::VideoTerms => push_to(&mut result.video_terms, val),
        TokenType::ResolutionTerms => push_to(&mut result.resolution_terms, val),
        TokenType::BatchTerms => push_to(&mut result.batch_terms, val),
        TokenType::SubtitleTerms => push_to(&mut result.subtitle_terms, val),
        TokenType::SeasonTerms => push_to(&mut result.season_terms, val),
        TokenType::SubtitleLanguageTerms => push_to(&mut result.subtitle_language_terms, val),
        TokenType::AudioLanguageTerms => push_to(&mut result.audio_language_terms, val),
        TokenType::SourceTerms => push_to(&mut result.source_terms, val),
        TokenType::Dates => push_to(&mut result.dates, val),
        TokenType::EpisodeTerms => push_to(&mut result.episode_terms, val),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parse() {
        let result = parse("[silly] Cyberpunk Edgerunners (WEB-DL 1080p HEVC E-AC-3) [Dual-Audio]");
        assert_eq!(result.groups, Some(vec!["silly".to_string()]));
        assert_eq!(result.titles, Some(vec!["Cyberpunk Edgerunners".to_string()]));
        assert_eq!(result.resolution_terms, Some(vec!["1080p".to_string()]));
        assert_eq!(result.video_codec_terms, Some(vec!["HEVC".to_string()]));
        assert_eq!(result.audio_codec_terms, Some(vec!["E-AC-3".to_string()]));
        assert_eq!(result.audio_terms, Some(vec!["Dual-Audio".to_string()]));
        assert_eq!(result.type_terms, Some(vec!["WEB-DL".to_string()]));
    }

    #[test]
    fn test_season_and_batch() {
        let result = parse("[DKB] Cyberpunk Edgerunners - Season 01 [1080p][HEVC x265 10bit][Dual-Audio][Multi-Subs][batch]");
        assert_eq!(result.groups, Some(vec!["DKB".to_string()]));
        assert_eq!(result.titles, Some(vec!["Cyberpunk Edgerunners".to_string()]));
        assert_eq!(result.resolution_terms, Some(vec!["1080p".to_string()]));
        assert_eq!(result.video_codec_terms, Some(vec!["HEVC".to_string(), "x265".to_string(), "10bit".to_string()]));
        assert_eq!(result.audio_terms, Some(vec!["Dual-Audio".to_string()]));
        assert_eq!(result.subtitle_terms, Some(vec!["Multi-Subs".to_string()]));
        assert_eq!(result.season_terms, Some(vec!["Season 01".to_string()]));
        assert_eq!(result.batch_terms, Some(vec!["batch".to_string()]));
    }

    #[test]
    fn test_date_and_season() {
        let result = parse("[EMBER] Cyberpunk: Edgerunners (2022) (Season 1) [WEBRip] [1080p Dual Audio HEVC 10 bits] (Cyberpunk Edgerunners) (Batch)");
        assert_eq!(result.groups, Some(vec!["EMBER".to_string()]));
        assert_eq!(result.dates, Some(vec!["2022".to_string()]));
        assert_eq!(result.resolution_terms, Some(vec!["1080p".to_string()]));
        assert_eq!(result.video_codec_terms, Some(vec!["HEVC".to_string(), "10 bits".to_string()]));
        assert_eq!(result.audio_terms, Some(vec!["Dual Audio".to_string()]));
        assert_eq!(result.season_terms, Some(vec!["Season 1".to_string()]));
        assert_eq!(result.batch_terms, Some(vec!["Batch".to_string()]));
        assert_eq!(result.type_terms, Some(vec!["WEBRip".to_string()]));
    }

    #[test]
    fn test_non_delimited_group_and_version() {
        let result = parse("Cyberpunk Edgerunners WEB-DL 1080P HDR DV EAC3 VF VOSTFR-LTPD v2");
        assert_eq!(result.groups, Some(vec!["LTPD".to_string()]));
        assert_eq!(result.titles, Some(vec!["Cyberpunk Edgerunners".to_string()]));
        assert_eq!(result.resolution_terms, Some(vec!["1080P".to_string()]));
        assert_eq!(result.video_terms, Some(vec!["HDR".to_string(), "DV".to_string()]));
        assert_eq!(result.audio_codec_terms, Some(vec!["EAC3".to_string()]));
        assert_eq!(result.audio_language_terms, Some(vec!["VF".to_string()]));
        assert_eq!(result.subtitle_language_terms, Some(vec!["VOSTFR".to_string()]));
        assert_eq!(result.type_terms, Some(vec!["WEB-DL".to_string()]));
        assert_eq!(result.version_terms, Some(vec!["v2".to_string()]));
    }

    #[test]
    fn test_multi_language() {
        let result = parse("[Erai-raws] Cyberpunk - Edgerunners - 01 ~ 10 [1080p][Multiple Subtitle] [ENG][POR-BR][SPA-LA][SPA][ARA][FRE][GER][ITA][JPN][POR][POL][DUT][NOB][FIN][TUR][SWE][GRE][HEB][RUM][IND][THA][KOR][DAN][CHI][VIE][UKR][HUN][CES][HRV][MAY][FIL]");
        assert_eq!(result.groups, Some(vec!["Erai-raws".to_string()]));
        assert_eq!(result.titles, Some(vec!["Cyberpunk - Edgerunners".to_string()]));
        assert_eq!(result.episode_terms, Some(vec!["01 ~ 10".to_string()]));
        assert_eq!(result.resolution_terms, Some(vec!["1080p".to_string()]));
        assert_eq!(result.subtitle_terms, Some(vec!["Multiple Subtitle".to_string()]));
        let langs = result.audio_language_terms.unwrap();
        assert_eq!(langs.len(), 31);
        assert_eq!(langs[0], "ENG");
        assert_eq!(langs[1], "POR-BR");
    }

    #[test]
    fn test_s01v2_multi() {
        let result = parse("Cyberpunk - Edgerunners - S01v2 - MULTi 1080p WEB DV H.265 -NanDesuKa (NF)");
        assert_eq!(result.groups, Some(vec!["NanDesuKa".to_string()]));
        assert_eq!(result.titles, Some(vec!["Cyberpunk - Edgerunners".to_string()]));
        assert_eq!(result.resolution_terms, Some(vec!["1080p".to_string()]));
        assert_eq!(result.video_codec_terms, Some(vec!["H.265".to_string()]));
        assert_eq!(result.video_terms, Some(vec!["DV".to_string()]));
        assert_eq!(result.audio_language_terms, Some(vec!["MULTi".to_string()]));
        assert_eq!(result.season_terms, Some(vec!["S01".to_string()]));
        assert_eq!(result.type_terms, Some(vec!["WEB".to_string()]));
        assert_eq!(result.version_terms, Some(vec!["v2".to_string()]));
        assert_eq!(result.source_terms, Some(vec!["NF".to_string()]));
    }

    #[test]
    fn test_subs_only() {
        let result = parse("[whomst] Cyberpunk: Edgerunners (subs only)");
        assert_eq!(result.groups, Some(vec!["whomst".to_string()]));
        assert_eq!(result.titles, Some(vec!["Cyberpunk: Edgerunners".to_string()]));
    }

    #[test]
    fn test_optional_dual_audio() {
        let result = parse("[Trix] Cyberpunk: Edgerunners (2022) [Optional Dual Audio] [Multi Subs] (1080p AV1)");
        assert_eq!(result.groups, Some(vec!["Trix".to_string()]));
        assert_eq!(result.dates, Some(vec!["2022".to_string()]));
        assert_eq!(result.titles, Some(vec!["Cyberpunk: Edgerunners".to_string()]));
        assert_eq!(result.resolution_terms, Some(vec!["1080p".to_string()]));
        assert_eq!(result.video_codec_terms, Some(vec!["AV1".to_string()]));
        assert_eq!(result.audio_language_terms, Some(vec!["Optional Dual Audio".to_string()]));
        assert_eq!(result.subtitle_terms, Some(vec!["Multi Subs".to_string()]));
    }

    #[test]
    fn test_720p() {
        let result = parse("[Trix] Cyberpunk: Edgerunners (2022) [Optional Dual Audio] [Multi Subs] (720p AV1)");
        assert_eq!(result.groups, Some(vec!["Trix".to_string()]));
        assert_eq!(result.resolution_terms, Some(vec!["720p".to_string()]));
        assert_eq!(result.video_codec_terms, Some(vec!["AV1".to_string()]));
    }
}
