use std::fmt;

use crate::tokens::{tokenize, TokenType};

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Matcher {
    /// Match any token.
    Any,
    /// Match numbers only.
    Number,
}

#[derive(Debug, Clone)]
pub enum Error {
    MatchError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::MatchError => write!(f, "unable to match specifiers with input"),
        }
    }
}

impl std::error::Error for Error {}

/// Match string with matchers.
///
/// # Arguments
///
/// * `s` - String slice to match.
/// * `matchers` - Slice of matchers to match with.
///
/// # Returns
///
/// A `Result` containing a `Vec` of indices pointing to the start of each match.
pub fn match_all(s: &str, matchers: &[Matcher]) -> Result<Vec<usize>> {
    let (token_indices, token_types) = tokenize(s);
    let indices = match_token(s, &token_indices, &token_types, matchers);
    if indices.len() != matchers.len() {
        return Err(Error::MatchError);
    }
    Ok(indices)
}

/// Check if token type matches matcher.
fn is_match(token_type: TokenType, matcher: &Matcher) -> bool {
    match matcher {
        Matcher::Any => true,
        Matcher::Number => token_type == TokenType::Number,
    }
}

/// Match token.
fn match_token(
    s: &str,
    token_indices: &[usize],
    token_types: &[TokenType],
    matchers: &[Matcher],
) -> Vec<usize> {
    if token_types.is_empty()
        || matchers.is_empty()
        || !is_match(token_types[0], &matchers[0])
        || (matchers.len() == 1 && token_indices.len() != 1 && matchers[0] != Matcher::Any)
    {
        return vec![];
    }
    let mut indices = vec![token_indices[0]];
    if matchers.len() > 1 {
        let next = match matchers[0] {
            Matcher::Any => match_any(s, &token_indices[1..], &token_types[1..], &matchers[1..]),
            _ => match_token(s, &token_indices[1..], &token_types[1..], &matchers[1..]),
        };
        if next.is_empty() {
            return vec![];
        }
        indices.extend(next);
    }
    indices
}

/// Match any matcher.
fn match_any(
    s: &str,
    token_indices: &[usize],
    token_types: &[TokenType],
    matchers: &[Matcher],
) -> Vec<usize> {
    if token_indices.is_empty() || matchers.is_empty() {
        return vec![];
    }
    let mut next = vec![];
    let mut next_token_idx = 0;
    while next.is_empty() {
        if next_token_idx == token_indices.len() {
            return vec![];
        }
        next = match_token(
            s,
            &token_indices[next_token_idx..],
            &token_types[next_token_idx..],
            &matchers,
        );
        next_token_idx += 1;
    }
    next
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! match_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (s, matchers, expected) = $value;
                    assert_eq!(match_all(s, matchers).unwrap(), expected);
                }
            )*
        }
    }

    match_tests!(
        match_1: ("abc", &[Matcher::Any], &[0]),
        match_2: ("abc123", &[Matcher::Any, Matcher::Any], &[0, 3]),
        match_2_to_1: ("abc123", &[Matcher::Any], &[0]),
        match_4_to_2: (
            "abc def456",
            &[Matcher::Any, Matcher::Any],
            &[0, 3],
        ),
        match_number: (
            "abc def456",
            &[Matcher::Any, Matcher::Number],
            &[0, 7],
        ),
        match_skip: (
            "abc123def456",
            &[Matcher::Any, Matcher::Number],
            &[0, 9],
        ),
    );

    macro_rules! match_fail_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (s, specs) = $value;
                    assert!(match_all(s, specs).is_err());
                }
            )*
        }
    }

    match_fail_tests!(
        match_no_match: (
            "abc123def456",
            &[Matcher::Any, Matcher::Number, Matcher::Number],
        ),
    );
}
