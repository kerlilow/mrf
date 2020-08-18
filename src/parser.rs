use std::fmt;
use std::str::FromStr;

use nom::{
    branch::alt,
    bytes::complete::{escaped, is_not},
    character::complete::{char, digit1, one_of},
    combinator::{all_consuming, map, map_res, opt, peek, verify},
    error::{convert_error, ParseError, VerboseError},
    multi::many0,
    sequence::{delimited, preceded},
    Err, IResult,
};

use crate::{elem::Elem, formatter::Formatter, matcher::Matcher, spec::Spec};

#[derive(Debug, Clone)]
pub struct Error {
    msg: String,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

/// Parse elements.
///
/// # Arguments
///
/// * `s` - String slice to parse.
///
/// # Returns
///
/// A `Result` containing a `Vec` of elements parsed from the string.
pub fn parse<'a>(s: &'a str) -> Result<Vec<Elem>, Error> {
    match root::<VerboseError<&'a str>>(s) {
        Ok((_, elems)) => Ok(elems),
        Err(Err::Error(e)) => Err(Error {
            msg: convert_error(s, e),
        }),
        Err(Err::Failure(e)) => Err(Error {
            msg: convert_error(s, e),
        }),
        Err(Err::Incomplete(_)) => Err(Error {
            msg: "incomplete input".to_owned(),
        }),
    }
}

/// Parse root.
///
/// Begin parsing from here.
fn root<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, Vec<Elem>, E> {
    all_consuming(many0(elem))(s)
}

/// Parse an element.
///
/// An element could be a "literal" (`Elem::Lit`) or a "specifier" (`Elem::Spec`).
fn elem<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, Elem, E> {
    alt((elem_lit, elem_spec))(s)
}

/// Parse a literal element.
///
/// A literal is everything outside any specifiers.
///
/// A literal ends when there is an opening curly brace, which denotes a specifier, or at the end of
/// the input.
///
/// A backslash (`\`) may be used to escape any of these characters: `{}\`.
fn elem_lit<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, Elem, E> {
    map(
        verify(escaped(is_not("\\{"), '\\', one_of(r#"{}\"#)), |v: &str| {
            !v.is_empty()
        }),
        |v: &str| Elem::Lit(unescape_lit(v)),
    )(s)
}

/// Unescape literals.
///
/// Remove backslashes from escaped characters.
fn unescape_lit(s: &str) -> String {
    s.replace("\\{", "{")
        .replace("\\}", "}")
        .replace("\\\\", "\\")
}

/// Parse a specifier element.
///
/// A specifier is surrounded by curly braces (`{<specifier>}`).
fn elem_spec<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, Elem, E> {
    map(delimited(char('{'), spec, char('}')), Elem::Spec)(s)
}

/// Parse a specifier.
///
/// A specifier consists of 4 optional parts:
/// 1. A matcher.
/// 2. An index.
/// 3. A replace string, preceded by an equal sign (`=`).
/// 4. A format string, preceded by a colon (`:`).
fn spec<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, Spec, E> {
    let (s, matcher) = spec_matcher(s)?;
    let (s, index) = opt(map_res(digit1, usize::from_str))(s)?;
    let (s, replace) = opt(preceded(char('='), spec_replace))(s)?;
    let (s, formatter) = opt(preceded(char(':'), spec_formatter))(s)?;
    Ok((
        s,
        Spec {
            matcher,
            index,
            replace,
            formatter,
        },
    ))
}

/// Parse a matcher.
///
/// A matcher is specified at the beginning of a specifier, until a digit (which indicates the
/// beginning of the index), an equal sign (which indicates the beginning of the replace string),
/// a colon (which indicates the beginning of the format string), or a closing curly brace (which
/// indicates the end of the specifier) is met.
///
/// One of the following is accepted:
/// * `"n"` - A `Number` matcher.
/// * `""` (Blank) - An `Any` matcher.
fn spec_matcher<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, Matcher, E> {
    map(opt(is_not("0123456789=:}")), |m: Option<&str>| {
        match &m.unwrap_or("").trim()[..] {
            "n" => Matcher::Number,
            _ => Matcher::Any,
        }
    })(s)
}

/// Parse a replace string.
///
/// A replace string ends when a colon (which indicates the beginning of the format string), or a
/// closing curly brace (which denotes the end of the specifier) is met.
///
/// A backslash (`\`) may be used to escape any of these characters: `{}:\`.
fn spec_replace<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, String, E> {
    map(
        opt(escaped(is_not("\\:}"), '\\', one_of(r#"{}:\"#))),
        |v: Option<&str>| unescape_replace(v.unwrap_or("")),
    )(s)
}

/// Unescape replace string.
///
/// Remove backslashes from escaped characters.
fn unescape_replace(s: &str) -> String {
    s.replace("\\{", "{")
        .replace("\\}", "}")
        .replace("\\:", ":")
        .replace("\\\\", "\\")
}

/// Parse a format string.
///
/// A format string ends when a closing curly brace is met.
fn spec_formatter<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, Formatter, E> {
    let (s, fill) = opt(char('0'))(s)?;
    let (s, width) = opt(digit1)(s)?;
    let (s, _) = peek(char('}'))(s)?;
    Ok((
        s,
        Formatter {
            fill: fill.unwrap_or(' '),
            width: width.map(|w| w.parse::<usize>().unwrap()).unwrap_or(0),
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! parse_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (s, expected) = $value;
                    assert_eq!(parse(s).unwrap_or_else(|e| {
                        panic!("{}", e);
                    }), expected);
                }
            )*
        }
    }

    parse_tests!(
        parse_empty: ("", &[]),
        parse_literal: ("abc", &[Elem::Lit("abc".to_owned())]),
        parse_any: ("{}", &[Elem::Spec(Spec::new(Matcher::Any))]),
        parse_number: ("{n}", &[Elem::Spec(Spec::new(Matcher::Number))]),
        parse_ignore_ws: ("{ }", &[Elem::Spec(Spec::new(Matcher::Any))]),
        parse_ignore_ws_number: ("{ n }", &[Elem::Spec(Spec::new(Matcher::Number))]),
        parse_prefix_any: ("abc-{}", &[
            Elem::Lit("abc-".to_owned()),
            Elem::Spec(Spec::new(Matcher::Any)),
        ]),
        parse_any_number: ("{}{n}", &[
            Elem::Spec(Spec::new(Matcher::Any)),
            Elem::Spec(Spec::new(Matcher::Number)),
        ]),
        parse_escaped: (r#"\{{}\}"#, &[
            Elem::Lit("{".to_owned()),
            Elem::Spec(Spec::new(Matcher::Any)),
            Elem::Lit("}".to_owned()),
        ]),

        parse_index: (
            "{1}",
            &[
                Elem::Spec(Spec {
                    matcher: Matcher::Any,
                    index: Some(1),
                    replace: None,
                    formatter: None
                }),
            ],
        ),

        parse_index_ws: (
            "{n 1}",
            &[
                Elem::Spec(Spec {
                    matcher: Matcher::Number,
                    index: Some(1),
                    replace: None,
                    formatter: None
                }),
            ],
        ),

        parse_replace: (
            "{=x}",
            &[
                Elem::Spec(Spec {
                    matcher: Matcher::Any,
                    index: None,
                    replace: Some("x".to_owned()),
                    formatter: None
                }),
            ],
        ),

        parse_empty_replace: (
            "{=}",
            &[
                Elem::Spec(Spec {
                    matcher: Matcher::Any,
                    index: None,
                    replace: Some("".to_owned()),
                    formatter: None
                }),
            ],
        ),

        parse_replace_with_escapes_1: (
            r#"{=\:}"#,
            &[
                Elem::Spec(Spec {
                    matcher: Matcher::Any,
                    index: None,
                    replace: Some(":".to_owned()),
                    formatter: None
                }),
            ],
        ),

        parse_replace_with_escapes_2: (
            r#"{=\:\:}"#,
            &[
                Elem::Spec(Spec {
                    matcher: Matcher::Any,
                    index: None,
                    replace: Some("::".to_owned()),
                    formatter: None
                }),
            ],
        ),

        parse_replace_with_escapes_3: (
            r#"{=\::}"#,
            &[
                Elem::Spec(Spec {
                    matcher: Matcher::Any,
                    index: None,
                    replace: Some(":".to_owned()),
                    formatter: Some(Formatter { fill: ' ', width: 0 }),
                }),
            ],
        ),

        parse_format: (
            "{:4}",
            &[
                Elem::Spec(Spec {
                    matcher: Matcher::Any,
                    index: None,
                    replace: None,
                    formatter: Some(Formatter { fill: ' ', width: 4 }),
                }),
            ],
        ),

        parse_format_zero: (
            "{n:04}",
            &[
                Elem::Spec(Spec {
                    matcher: Matcher::Number,
                    index: None,
                    replace: None,
                    formatter: Some(Formatter { fill: '0', width: 4 }),
                }),
            ],
        ),

        parse_replace_format: (
            "{n=1:04}",
            &[
                Elem::Spec(Spec {
                    matcher: Matcher::Number,
                    index: None,
                    replace: Some("1".to_owned()),
                    formatter: Some(Formatter { fill: '0', width: 4 }),
                }),
            ],
        ),

        parse_index_replace_format: (
            "{n1=1:04}",
            &[
                Elem::Spec(Spec {
                    matcher: Matcher::Number,
                    index: Some(1),
                    replace: Some("1".to_owned()),
                    formatter: Some(Formatter { fill: '0', width: 4 }),
                }),
            ],
        ),
    );

    #[test]
    fn parse_incomplete() {
        assert!(parse("{").is_err());
    }
}
