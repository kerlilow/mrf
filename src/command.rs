use std::fmt;

use nom::{
    branch::alt,
    bytes::complete::{escaped, is_a, is_not},
    character::complete::{char, one_of},
    combinator::{all_consuming, map, opt},
    multi::{many1, separated_list},
    sequence::delimited,
    IResult,
};

#[derive(Debug, Clone)]
pub struct Error;

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "failed to parse command")
    }
}

/// Parse command.
pub fn parse(s: &str) -> Result<Vec<String>, Error> {
    match all_args(s) {
        Ok((_, a)) => Ok(a),
        Err(_) => Err(Error {}),
    }
}

/// Parse all arguments.
fn all_args(s: &str) -> IResult<&str, Vec<String>> {
    all_consuming(separated_list(is_a(" "), arg))(s.trim())
}

/// Parse one argument.
fn arg(s: &str) -> IResult<&str, String> {
    map(
        many1(alt((is_not(" \"\'"), double_quoted, single_quoted))),
        |a| a.join(""),
    )(s)
}

/// Parse a double quoted string.
fn double_quoted(s: &str) -> IResult<&str, &str> {
    delimited(
        char('"'),
        map(opt(escaped(is_not("\\\""), '\\', one_of(r#""\"#))), |a| {
            a.unwrap_or("")
        }),
        char('"'),
    )(s)
}

/// Parse a single quoted string.
fn single_quoted(s: &str) -> IResult<&str, &str> {
    delimited(
        char('\''),
        map(opt(escaped(is_not("\\'"), '\\', one_of(r#"'\"#))), |a| {
            a.unwrap_or("")
        }),
        char('\''),
    )(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! all_args_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (s, expected) = $value;
                    assert_eq!(
                        all_args(s).unwrap(),
                        ("", expected.iter().map(|a| a.to_string()).collect())
                    );
                }
            )*
        }
    }

    all_args_tests!(
        all_args_empty: ("", Vec::<&str>::new()),
        all_args_simple: ("echo", vec!["echo"]),
        all_args_multi: (r#"echo "Hello, World!""#, vec!["echo", "Hello, World!"]),
        all_args_spaces: (r#"echo    "Hello, World!""#, vec!["echo", "Hello, World!"]),
        all_args_start_spaces: (r#" echo "Hello, World!" "#, vec!["echo", "Hello, World!"]),
        all_args_end_spaces: (r#"echo "Hello, World!" "#, vec!["echo", "Hello, World!"]),
    );

    macro_rules! arg_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (s, expected) = $value;
                    assert_eq!(arg(s).unwrap(), (expected.0, expected.1.to_owned()));
                }
            )*
        }
    }

    arg_tests!(
        arg_simple: ("a", ("", "a")),
        arg_take_one: ("a b", (" b", "a")),
        arg_double_quoted: (r#""a b""#, ("", r"a b")),
        arg_single_quoted: ("'a b'", ("", r"a b")),
        arg_joined_quoted: (r#"a"b c""#, ("", r"ab c")),
    );

    macro_rules! double_quoted_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (s, expected) = $value;
                    assert_eq!(double_quoted(s).unwrap(), expected);
                }
            )*
        }
    }

    double_quoted_tests!(
        double_quoted_empty: (r#""""#, ("", "")),
        double_quoted_simple: (r#""abc""#, ("", "abc")),
        double_quoted_escaped: (r#""abc \"def\"""#, ("", r#"abc \"def\""#)),
        double_quoted_not_escaped: (r#""abc "def""#, (r#"def""#, "abc ")),
        double_quoted_escaped_backslash: (r#""abc \\"def\""#, (r#"def\""#, r#"abc \\"#)),
        double_quoted_single_quotes: (r#""a'b'c""#, ("", "a'b'c")),
    );

    macro_rules! single_quoted_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (s, expected) = $value;
                    assert_eq!(single_quoted(s).unwrap(), expected);
                }
            )*
        }
    }

    single_quoted_tests!(
        single_quoted_empty: ("''", ("", "")),
        single_quoted_simple: ("'abc'", ("", "abc")),
        single_quoted_escaped: (r#"'abc \'def\''"#, ("", r#"abc \'def\'"#)),
        single_quoted_not_escaped: (r#"'abc 'def'"#, ("def'", "abc ")),
        single_quoted_escaped_backslash: (r#"'abc \\'def\'"#, (r#"def\'"#, r#"abc \\"#)),
        single_quoted_double_quotes: (r#"'a"b"c'"#, ("", r#"a"b"c"#)),
    );
}
