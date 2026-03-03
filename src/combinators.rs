#![allow(dead_code, mismatched_lifetime_syntaxes)]
/// Minimal composable parser combinator library optimized for small WASM size.
/// No allocations in the core combinators where possible.

#[derive(Debug, Clone, PartialEq)]
pub struct ParseState<'a> {
    pub input: &'a str,
    pub pos: usize,
}

impl<'a> ParseState<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    pub fn remaining(&self) -> &'a str {
        &self.input[self.pos..]
    }

    pub fn is_empty(&self) -> bool {
        self.pos >= self.input.len()
    }

    pub fn advance(&self, n: usize) -> Self {
        Self {
            input: self.input,
            pos: self.pos + n,
        }
    }
}

pub type ParseResult<'a, T> = Result<(T, ParseState<'a>), ParseState<'a>>;

/// Try to match a case-insensitive string literal. Returns the matched slice from input.
pub fn istr<'a>(target: &str, state: ParseState<'a>) -> ParseResult<'a, &'a str> {
    let rem = state.remaining();
    if rem.len() >= target.len() {
        let candidate = &rem[..target.len()];
        if candidate.eq_ignore_ascii_case(target) {
            let matched = &state.input[state.pos..state.pos + target.len()];
            return Ok((matched, state.advance(target.len())));
        }
    }
    Err(state)
}

/// Match an exact string (case-sensitive).
pub fn str_exact<'a>(target: &str, state: ParseState<'a>) -> ParseResult<'a, &'a str> {
    let rem = state.remaining();
    if rem.starts_with(target) {
        let matched = &state.input[state.pos..state.pos + target.len()];
        Ok((matched, state.advance(target.len())))
    } else {
        Err(state)
    }
}

/// Match a single ASCII digit.
pub fn digit(state: ParseState) -> ParseResult<char> {
    let rem = state.remaining();
    if let Some(c) = rem.chars().next() {
        if c.is_ascii_digit() {
            return Ok((c, state.advance(c.len_utf8())));
        }
    }
    Err(state)
}

/// Match one or more ASCII digits, returning the matched slice.
pub fn digits(state: ParseState) -> ParseResult<&str> {
    let rem = state.remaining();
    let count = rem.chars().take_while(|c| c.is_ascii_digit()).count();
    if count > 0 {
        let byte_len: usize = rem[..].chars().take(count).map(|c| c.len_utf8()).sum();
        let matched = &state.input[state.pos..state.pos + byte_len];
        Ok((matched, state.advance(byte_len)))
    } else {
        Err(state)
    }
}

/// Match a single whitespace character.
pub fn whitespace(state: ParseState) -> ParseResult<char> {
    let rem = state.remaining();
    if let Some(c) = rem.chars().next() {
        if c.is_whitespace() {
            return Ok((c, state.advance(c.len_utf8())));
        }
    }
    Err(state)
}

/// Match zero or more whitespace characters.
pub fn skip_whitespace(state: ParseState) -> ParseResult<&str> {
    let rem = state.remaining();
    let byte_len: usize = rem.chars().take_while(|c| c.is_whitespace()).map(|c| c.len_utf8()).sum();
    let matched = &state.input[state.pos..state.pos + byte_len];
    Ok((matched, state.advance(byte_len)))
}

/// Match one or more whitespace characters.
pub fn whitespace1(state: ParseState) -> ParseResult<&str> {
    let rem = state.remaining();
    let byte_len: usize = rem.chars().take_while(|c| c.is_whitespace()).map(|c| c.len_utf8()).sum();
    if byte_len > 0 {
        let matched = &state.input[state.pos..state.pos + byte_len];
        Ok((matched, state.advance(byte_len)))
    } else {
        Err(state)
    }
}

/// Match a specific character (case-insensitive for ASCII).
pub fn ichar(target: char, state: ParseState) -> ParseResult<char> {
    let rem = state.remaining();
    if let Some(c) = rem.chars().next() {
        if c.to_ascii_lowercase() == target.to_ascii_lowercase() {
            return Ok((c, state.advance(c.len_utf8())));
        }
    }
    Err(state)
}

/// Match a specific character (exact).
pub fn char_exact(target: char, state: ParseState) -> ParseResult<char> {
    let rem = state.remaining();
    if let Some(c) = rem.chars().next() {
        if c == target {
            return Ok((c, state.advance(c.len_utf8())));
        }
    }
    Err(state)
}

/// Match any single character.
pub fn any_char(state: ParseState) -> ParseResult<char> {
    let rem = state.remaining();
    if let Some(c) = rem.chars().next() {
        Ok((c, state.advance(c.len_utf8())))
    } else {
        Err(state)
    }
}

/// Match any character except the given one.
pub fn any_char_except(except: char, state: ParseState) -> ParseResult<char> {
    let rem = state.remaining();
    if let Some(c) = rem.chars().next() {
        if c != except {
            return Ok((c, state.advance(c.len_utf8())));
        }
    }
    Err(state)
}

/// Collect all characters until a condition is met (returns the slice before).
/// Does not consume the stopping character.
pub fn take_until<F>(predicate: F, state: ParseState) -> ParseResult<&str>
where
    F: Fn(char) -> bool,
{
    let rem = state.remaining();
    let byte_len: usize = rem.chars().take_while(|c| !predicate(*c)).map(|c| c.len_utf8()).sum();
    let matched = &state.input[state.pos..state.pos + byte_len];
    Ok((matched, state.advance(byte_len)))
}

/// Check if we're at end of input.
pub fn end_of_input(state: ParseState) -> ParseResult<()> {
    if state.is_empty() {
        Ok(((), state))
    } else {
        Err(state)
    }
}

/// Lookahead: test a parser without consuming input.
pub fn lookahead<'a, T, F>(parser: F, state: ParseState<'a>) -> ParseResult<'a, T>
where
    F: Fn(ParseState<'a>) -> ParseResult<'a, T>,
{
    match parser(state.clone()) {
        Ok((val, _)) => Ok((val, state)),
        Err(_) => Err(state),
    }
}

/// Try the first parser in the slice that succeeds.
/// Each entry maps a parser result to a term type string.
pub fn choice_istr<'a>(terms: &[&str], state: ParseState<'a>) -> ParseResult<'a, &'a str> {
    for term in terms {
        if let Ok(result) = istr(term, state.clone()) {
            return Ok(result);
        }
    }
    Err(state)
}

/// Try exact-case string choices.
pub fn choice_str<'a>(terms: &[&str], state: ParseState<'a>) -> ParseResult<'a, &'a str> {
    for term in terms {
        if let Ok(result) = str_exact(term, state.clone()) {
            return Ok(result);
        }
    }
    Err(state)
}
