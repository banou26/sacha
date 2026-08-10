use nom::{
    IResult,
    branch::alt,
    bytes::complete::{take_while1, take_while_m_n},
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
        // `get` rather than `[..]`: a multi-byte char straddling `pattern.len()` would panic
        let Some(head) = input.get(..pattern.len()) else {
            return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
        };
        if head.eq_ignore_ascii_case(pattern) {
            Ok((&input[pattern.len()..], head))
        } else {
            Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)))
        }
    }
}

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

/// Finds the first term in `idx` that starts at `input`, ignoring case.
/// Only the bucket sharing the input's first byte is visited, so the term that
/// wins is the same one the full scan would have found.
fn match_indexed_icase<'a>(idx: &TermIndex, input: &'a str) -> IResult<&'a str, &'a str> {
    for term in idx.candidates(input) {
        if let Ok(result) = istr(term)(input) {
            return Ok(result);
        }
    }
    Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)))
}

/// Case-sensitive counterpart of `match_indexed_icase`.
fn match_indexed_exact<'a>(idx: &TermIndex, input: &'a str) -> IResult<&'a str, &'a str> {
    for term in idx.candidates(input) {
        if input.get(..term.len()) == Some(*term) {
            return Ok((&input[term.len()..], &input[..term.len()]));
        }
    }
    Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)))
}

fn resolution_number(input: &str) -> IResult<&str, &str> {
    for s in RESOLUTION_STRS {
        if input.starts_with(s) {
            return Ok((&input[s.len()..], &input[..s.len()]));
        }
    }
    Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)))
}

fn resolution_primitive(input: &str) -> IResult<&str, String> {
    let (rest, num) = resolution_number(input)?;
    if let Ok((rest2, sfx)) = istr("px")(rest) {
        return Ok((rest2, format!("{}{}", num, sfx)));
    }
    if let Ok((rest2, sfx)) = istr("p")(rest) {
        return Ok((rest2, format!("{}{}", num, sfx)));
    }
    Ok((rest, num.to_string()))
}

fn resolution_token(input: &str) -> IResult<&str, String> {
    if let Ok((rest, alias)) = match_indexed_icase(ix_resolution_aliases(), input) {
        return Ok((rest, alias.to_string()));
    }
    let (rest, first) = resolution_primitive(input)?;
    if let Ok((rest2, _)) = istr("x")(rest) {
        if let Ok((rest3, second)) = resolution_primitive(rest2) {
            return Ok((rest3, format!("{}x{}", first, second)));
        }
    }
    Ok((rest, first))
}

fn version_token(input: &str) -> IResult<&str, String> {
    let (rest, v) = satisfy(|c| c == 'v' || c == 'V')(input)?;
    let (rest, d) = digit_char(rest)?;
    Ok((rest, format!("{}{}", v, d)))
}

fn four_digits(input: &str) -> IResult<&str, &str> {
    take_while_m_n(4, 4, |c: char| c.is_ascii_digit())(input)
}

fn two_digits(input: &str) -> IResult<&str, &str> {
    take_while_m_n(2, 2, |c: char| c.is_ascii_digit())(input)
}

fn date_token(input: &str) -> IResult<&str, String> {
    if let Ok((rest, result)) = recognize(tuple((
        four_digits,
        char('.'),
        two_digits,
        char('.'),
        two_digits,
    )))(input) {
        return Ok((rest, result.to_string()));
    }
    if let Ok((rest, result)) = recognize(tuple((
        two_digits,
        char('.'),
        two_digits,
        char('.'),
        four_digits,
    )))(input) {
        return Ok((rest, result.to_string()));
    }
    let (rest, y) = four_digits(input)?;
    Ok((rest, y.to_string()))
}

fn episode_token(input: &str) -> IResult<&str, String> {
    let (rest, start) = digits1(input)?;
    let (rest, _) = ws0(rest)?;
    let (rest, sep) = alt((char('~'), char('-')))(rest)?;
    let (rest, _) = ws0(rest)?;
    let (rest, end) = digits1(rest)?;
    Ok((rest, format!("{} {} {}", start, sep, end)))
}

fn season_prefix_token(input: &str) -> IResult<&str, String> {
    let (rest, part) = match_indexed_icase(ix_season_part_terms(), input)?;
    let (rest, digits_and_spaces) = take_while1(|c: char| c.is_ascii_digit() || c == ' ')(rest)?;
    if !digits_and_spaces.chars().any(|c| c.is_ascii_digit()) {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
    }
    let trimmed_end = digits_and_spaces.trim_end();
    Ok((rest, format!("{}{}", part, trimmed_end)))
}

/// Words that may follow an English ordinal. Spelled out rather than reusing
/// [`SEASON_PART_TERMS`] because that list carries the bare `S`, and `2nd S` is not a season.
const ORDINAL_SEASON_WORDS: &[&str] = &["Season", "Part", "Cour"];

/// `4th Season` puts the number before the word, so the prefix form cannot reach it. It is the
/// second most common season spelling in the metadata sources, behind `Season 4` and ahead of
/// every non-Latin form put together.
fn season_ordinal_token(input: &str) -> IResult<&str, String> {
    let fail = || nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag));

    let digits: usize = input.chars().take_while(char::is_ascii_digit).count();
    if digits == 0 {
        return Err(fail());
    }
    let rest = &input[digits..];
    let suffix = rest.get(..2).ok_or_else(fail)?;
    if !matches!(suffix.to_ascii_lowercase().as_str(), "st" | "nd" | "rd" | "th") {
        return Err(fail());
    }
    let rest = &rest[2..];
    let spaces = rest.chars().take_while(|c| *c == ' ').count();
    let rest = &rest[spaces..];

    let word = ORDINAL_SEASON_WORDS
        .iter()
        .find(|word| {
            rest.get(..word.len())
                .is_some_and(|head| head.eq_ignore_ascii_case(word))
        })
        .ok_or_else(fail)?;
    let rest = &rest[word.len()..];
    // `2nd Seasonal` is not a season, so the word has to end here
    if rest.chars().next().is_some_and(char::is_alphanumeric) {
        return Err(fail());
    }

    let taken = input.len() - rest.len();
    Ok((rest, input[..taken].to_string()))
}

fn cjk_digit(c: char) -> Option<u32> {
    match c {
        '〇' | '零' => Some(0),
        '一' => Some(1),
        '二' => Some(2),
        '三' => Some(3),
        '四' => Some(4),
        '五' => Some(5),
        '六' => Some(6),
        '七' => Some(7),
        '八' => Some(8),
        '九' => Some(9),
        _ => None,
    }
}

/// Units that turn a number into a season: 期 and 季 in Japanese and Chinese, 기 in Korean.
/// 話 and 集 are deliberately absent, those count episodes.
const CJK_SEASON_UNITS: &[char] = &['期', '季', '기'];

/// `第二季` and `4기` name a season the way `Season 2` does, but the number sits between the
/// marker and the unit, which no prefix or suffix form can express. Chinese writes the number
/// itself in CJK numerals, so 第四季 has no ascii digit to key off at all.
fn season_cjk_token(input: &str) -> IResult<&str, String> {
    let fail = || nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag));

    let rest = input.strip_prefix('第').unwrap_or(input);

    let ascii: usize = rest.chars().take_while(char::is_ascii_digit).count();
    let number: usize = if ascii > 0 {
        ascii
    } else {
        rest.chars()
            .take_while(|c| cjk_digit(*c).is_some() || *c == '十')
            .map(char::len_utf8)
            .sum()
    };
    if number == 0 {
        return Err(fail());
    }
    let rest = &rest[number..];

    let unit = rest.chars().next().ok_or_else(fail)?;
    if !CJK_SEASON_UNITS.contains(&unit) {
        return Err(fail());
    }
    let rest = &rest[unit.len_utf8()..];

    let taken = input.len() - rest.len();
    Ok((rest, input[..taken].to_string()))
}

fn season_token(input: &str) -> IResult<&str, String> {
    alt((season_cjk_token, season_ordinal_token, season_prefix_token))(input)
}

fn non_delimited_group(input: &str) -> IResult<&str, String> {
    let (rest, _) = char('-')(input)?;
    let (rest, name) = take_while1(|c: char| c != '.' && !is_whitespace(c) && !is_closing_delim(c))(rest)?;
    if name.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
    }
    // a group tag is the tail of the name: this keeps the honorific in `Kobayashi-san Chi no Maid Dragon` attached to the title
    let tail = rest.trim_start_matches(|c: char| is_whitespace(c));
    let ends_the_name = tail.is_empty()
        || tail.starts_with(|c: char| is_opening_delim(c) || is_closing_delim(c))
        || parse_version(tail).is_ok()
        || try_metadata_token(tail).is_ok();
    if !ends_the_name {
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

fn parse_version(input: &str) -> IResult<&str, MetadataToken> {
    let (rest, v) = version_token(input)?;
    Ok((rest, MetadataToken { kind: MetadataKind::VersionTerm, value: v }))
}

fn parse_type_term(input: &str) -> IResult<&str, MetadataToken> {
    let (rest, v) = match_indexed_icase(ix_type_terms(), input)?;
    Ok((rest, MetadataToken { kind: MetadataKind::TypeTerm, value: v.to_string() }))
}

fn parse_audio_codec(input: &str) -> IResult<&str, MetadataToken> {
    let (rest, v) = match_indexed_icase(ix_audio_codecs(), input)?;
    Ok((rest, MetadataToken { kind: MetadataKind::AudioCodec, value: v.to_string() }))
}

fn parse_audio_term(input: &str) -> IResult<&str, MetadataToken> {
    let (rest, v) = match_indexed_icase(ix_audio_terms(), input)?;
    Ok((rest, MetadataToken { kind: MetadataKind::AudioTerm, value: v.to_string() }))
}

fn parse_container_term(input: &str) -> IResult<&str, MetadataToken> {
    let (rest, v) = match_indexed_icase(ix_container_terms(), input)?;
    Ok((rest, MetadataToken { kind: MetadataKind::ContainerTerm, value: v.to_string() }))
}

fn parse_video_codec(input: &str) -> IResult<&str, MetadataToken> {
    let (rest, v) = match_indexed_icase(ix_video_codecs(), input)?;
    Ok((rest, MetadataToken { kind: MetadataKind::VideoCodec, value: v.to_string() }))
}

fn parse_video_term(input: &str) -> IResult<&str, MetadataToken> {
    let (rest, v) = match_indexed_icase(ix_video_terms(), input)?;
    Ok((rest, MetadataToken { kind: MetadataKind::VideoTerm, value: v.to_string() }))
}

fn parse_resolution(input: &str) -> IResult<&str, MetadataToken> {
    let (rest, v) = resolution_token(input)?;
    Ok((rest, MetadataToken { kind: MetadataKind::Resolution, value: v }))
}

fn parse_batch_term(input: &str) -> IResult<&str, MetadataToken> {
    let (rest, v) = match_indexed_icase(ix_batch_terms(), input)?;
    Ok((rest, MetadataToken { kind: MetadataKind::BatchTerm, value: v.to_string() }))
}

fn parse_subtitle_term(input: &str) -> IResult<&str, MetadataToken> {
    let (rest, v) = match_indexed_icase(ix_subtitle_terms(), input)?;
    Ok((rest, MetadataToken { kind: MetadataKind::SubtitleTerm, value: v.to_string() }))
}

fn parse_season_term(input: &str) -> IResult<&str, MetadataToken> {
    let (rest, v) = season_token(input)?;
    Ok((rest, MetadataToken { kind: MetadataKind::SeasonTerm, value: v }))
}

fn parse_subtitle_language(input: &str) -> IResult<&str, MetadataToken> {
    let (rest, v) = match_indexed_icase(ix_subtitle_language_codes(), input)?;
    Ok((rest, MetadataToken { kind: MetadataKind::SubtitleLanguage, value: v.to_string() }))
}

fn parse_audio_language(input: &str) -> IResult<&str, MetadataToken> {
    if let Ok((rest, v)) = match_indexed_icase(ix_audio_language_terms(), input) {
        return Ok((rest, MetadataToken { kind: MetadataKind::AudioLanguage, value: v.to_string() }));
    }
    if let Ok((rest, v)) = match_indexed_exact(ix_audio_language_exact(), input) {
        return Ok((rest, MetadataToken { kind: MetadataKind::AudioLanguage, value: v.to_string() }));
    }
    // language codes are caps by convention: matching loosely turns the `Chi` of `Kobayashi-san Chi no Maid Dragon` into Chinese
    let (rest, v) = match_indexed_exact(ix_language_codes(), input)?;
    Ok((rest, MetadataToken { kind: MetadataKind::AudioLanguage, value: v.to_string() }))
}

fn parse_source_term(input: &str) -> IResult<&str, MetadataToken> {
    // likewise caps by convention: `CR` is Crunchyroll, `Cr` is part of a word
    let (rest, v) = match_indexed_exact(ix_source_term_keys(), input)?;
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

/// The ordered list of metadata parsers tried inside delimited blocks.
/// Order matters: longer/more-specific patterns must come before shorter ones.
fn try_metadata_token(input: &str) -> IResult<&str, MetadataToken> {
    // same order as the TypeScript implementation's `metadataTokenValue` array; keep the two in step
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

    let close_pos = rest.find(close).ok_or_else(|| {
        nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag))
    })?;

    let inner = &rest[..close_pos];
    let after_close = &rest[close_pos + close.len_utf8()..];

    let tokens = parse_inner_metadata(inner);

    Ok((after_close, RootToken::Metadata(tokens)))
}

/// Characters that join two metadata terms inside one bracket, as in
/// `HEVC-10bit` or `CHS/CHT`.
fn is_term_joiner(c: char) -> bool {
    matches!(c, '-' | '/' | '&' | ',' | '+' | '_' | '|') || is_whitespace(c)
}

/// Reads `text` as a run of metadata terms separated by punctuation, returning
/// the tokens only when the whole of it is accounted for. A single leftover
/// word makes it a title instead, so this never eats a name.
fn all_metadata(text: &str) -> Option<Vec<MetadataToken>> {
    let mut rest = text.trim();
    if rest.is_empty() {
        return None;
    }
    let mut tokens = Vec::new();
    while !rest.is_empty() {
        rest = rest.trim_start_matches(is_term_joiner);
        if rest.is_empty() {
            break;
        }
        let (next, token) = try_metadata_token(rest).ok()?;
        // the term has to end where a joiner or the text does, so `WEB` cannot claim the front of `WEBHOOK`
        if !next.is_empty() && !next.starts_with(is_term_joiner) {
            return None;
        }
        tokens.push(token);
        rest = next;
    }
    (!tokens.is_empty()).then_some(tokens)
}

/// Text left over inside a delimited block once the metadata is taken out is
/// worth keeping only when it could be a name. Episode counts, language tags
/// and CRC32 stamps are not.
/// A checksum such as `91ABD6D7` or `40DD2396`: hex digits only, and long
/// enough that a short word like `dead` cannot be mistaken for one.
fn is_hex_stamp(text: &str) -> bool {
    text.len() >= 6
        && text.chars().all(|c| c.is_ascii_hexdigit())
        && text.chars().any(|c| c.is_ascii_digit())
}

fn is_title_like(text: &str) -> bool {
    if !text.chars().any(char::is_alphabetic) {
        return false;
    }
    if is_hex_stamp(text) {
        return false;
    }
    if NOTE_TERMS.iter().any(|n| text.eq_ignore_ascii_case(n)) {
        return false;
    }
    true
}

/// Parse the content inside delimiters, extracting metadata tokens and keeping
/// the text between them.
///
/// Bracketed text that is not metadata is usually a title or an alias of one:
/// `[名探偵プリキュア！/Star Detective Precure！]` is the only title that name
/// carries, and `(Shingeki no Kyojin)` is the romaji alias of a Cyrillic title.
/// Both used to be discarded a character at a time.
fn parse_inner_metadata(mut input: &str) -> Vec<MetadataToken> {
    // a CRC32 stamp has to be rejected whole: tokenizing it lets the date parser claim the `2396` of `40DD2396`
    if is_hex_stamp(input.trim()) {
        return Vec::new();
    }

    let mut tokens = Vec::new();
    let mut pending = String::new();

    fn flush(pending: &mut String, tokens: &mut Vec<MetadataToken>) {
        let text = trim_non_word(pending);
        pending.clear();
        if let Some(mut inner) = all_metadata(&text) {
            tokens.append(&mut inner);
        } else if is_title_like(&text) {
            tokens.push(MetadataToken { kind: MetadataKind::Title, value: text });
        }
    }

    while !input.is_empty() {
        if let Ok((rest, ws_text)) = ws(input) {
            pending.push_str(ws_text);
            input = rest;
            if input.is_empty() {
                break;
            }
        }

        if let Ok((rest, token)) = try_metadata_token(input) {
            if rest.is_empty()
                || rest.starts_with(|c: char| is_whitespace(c))
                || rest.starts_with(|c: char| is_closing_delim(c))
            {
                flush(&mut pending, &mut tokens);
                tokens.push(token);
                input = rest;
                continue;
            }
        }

        let mut chars = input.chars();
        pending.push(chars.next().unwrap());
        input = chars.as_str();
    }

    flush(&mut pending, &mut tokens);
    tokens
}

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
    if c == '-' {
        return true;
    }
    if parse_version(rest).is_ok() || try_metadata_token(rest).is_ok() {
        return true;
    }
    false
}

fn parse_non_delimited_metadata(input: &str) -> IResult<&str, RootToken> {
    if let Ok((rest, tok)) = parse_group_from_hyphen(input) {
        return Ok((rest, RootToken::Metadata(vec![tok])));
    }

    if let Ok((rest_ws, _)) = ws(input) {
        if let Ok((rest, tok)) = try_metadata_token(rest_ws) {
            if is_token_boundary(rest) {
                return Ok((rest, RootToken::Metadata(vec![tok])));
            }
        }
        if let Ok((rest, tok)) = parse_episode(rest_ws) {
            if is_token_boundary(rest) {
                return Ok((rest, RootToken::Metadata(vec![tok])));
            }
        }
        if let Ok((rest, tok)) = parse_group_from_hyphen(rest_ws) {
            return Ok((rest, RootToken::Metadata(vec![tok])));
        }
    }

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

fn parse_data_token(input: &str) -> IResult<&str, RootToken> {
    if input.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Eof)));
    }

    let mut pos = 0;

    while pos < input.len() {
        let remaining = &input[pos..];

        if parse_delimited_block(remaining).is_ok() || parse_non_delimited_metadata(remaining).is_ok() {
            break;
        }

        let c = remaining.chars().next().unwrap();
        pos += c.len_utf8();
    }

    if pos == 0 {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
    }

    Ok((&input[pos..], RootToken::Data(input[..pos].to_string())))
}

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

fn is_non_word(c: char) -> bool {
    c.is_ascii_punctuation() || c.is_whitespace() || c.is_control() || c.is_ascii_control()
        || matches!(c, '\u{00A0}'..='\u{00BF}') // symbols/punctuation block
}

fn trim_non_word(s: &str) -> String {
    let trimmed = s.trim();
    let start = trimmed.find(|c: char| !is_non_word(c)).unwrap_or(0);
    // `i + c.len_utf8()`, not `i + 1`: a multi-byte trailing char would leave `end` inside it
    let end = trimmed
        .char_indices()
        .rev()
        .find(|(_, c)| !is_non_word(*c))
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(0);
    if start >= end {
        String::new()
    } else {
        trimmed[start..end].to_string()
    }
}

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

pub fn parse(input: &str) -> Result<ParseResult, String> {
    // separators are normalized first because scene releases delimit words with `.` or `_`; the tokenizer keys metadata off whitespace, so an unnormalized name collapses into a single title
    let normalized = crate::normalize::normalize_separators(input);
    let input = normalized.as_ref();

    let (_, tokens) = parse_all_tokens(input)
        .map_err(|e| format!("Parser errored: {:?}", e))?;

    let mut result = ParseResult::default();

    let mut first = true;
    for token in &tokens {
        match token {
            RootToken::Metadata(items) => {
                if first && items.is_empty() {
                    // deliberate no-op: a first delimited block with no recognized metadata is handled by the first-token logic just below
                }

                if first {
                    first = false;
                    // a leading bracket holding nothing but text is the release group, as in `[SubsPlease]`, so it outranks the title reading
                    if items.len() == 1 && items[0].kind == MetadataKind::Title {
                        result.push(&MetadataKind::Group, items[0].value.clone());
                        continue;
                    }
                    if items.is_empty() {
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
        assert_eq!(
            r.titles.as_deref(),
            Some(&["Cyberpunk: Edgerunners".to_string(), "Cyberpunk Edgerunners".to_string()][..])
        );
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
