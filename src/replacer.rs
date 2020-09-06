use std::error::Error;

use crate::{
    elem::Elem,
    formatter::InputType,
    indices::SplitAtIndices,
    matcher::{match_all, Matcher},
    spec::Spec,
};

pub struct Replacer {
    elems: Vec<Elem>,
    matchers: Vec<Matcher>,
}

impl Replacer {
    /// Create a `Replacer` with the given elements.
    ///
    /// # Arguments
    ///
    /// * `elems` - Elements.
    ///
    /// # Returns
    ///
    /// A `Replacer`.
    pub fn new(elems: &[Elem]) -> Self {
        Self {
            elems: elems.to_vec(),
            matchers: matchers_from_elems(elems),
        }
    }

    /// Replace string according to elements.
    ///
    /// # Arguments
    ///
    /// * `s` - String slice to replace.
    ///
    /// # Returns
    ///
    /// A `Result` containing the replaced string and the indices.
    pub fn replace(&self, s: &str) -> Result<(String, ReplaceIndices), Box<dyn Error>> {
        let indices = match_all(s, &self.matchers)?;
        let parts: Vec<&str> = [vec![s], s.split_at_indices(&indices)].concat();
        let mut cursor = 1;
        let mut pos = 0;
        let mut replaced_parts = vec![];
        let mut replaced_indices = vec![];
        let mut sources = vec![];
        for e in &self.elems {
            let (r, src) = match e {
                Elem::Spec(spec) => {
                    let (idx, r) = replace_spec(&spec, cursor, &parts);
                    cursor = idx + 1;
                    let src = if idx == 0 {
                        ReplaceSource::Entire
                    } else {
                        ReplaceSource::Index(idx - 1)
                    };
                    (r, src)
                }
                Elem::Lit(lit) => (lit.to_owned(), ReplaceSource::Literal),
            };
            replaced_indices.push(pos);
            sources.push(src);
            pos += r.len();
            replaced_parts.push(r);
        }
        Ok((
            replaced_parts.join(""),
            ReplaceIndices {
                matches: indices,
                replaced: replaced_indices,
                sources,
            },
        ))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReplaceIndices {
    /// Match indices in source string.
    pub matches: Vec<usize>,
    /// Part indices in replaced string.
    pub replaced: Vec<usize>,
    /// Replacement sources.
    pub sources: Vec<ReplaceSource>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReplaceSource {
    /// Replaces with match at index.
    Index(usize),
    /// Replaces with entire input string.
    Entire,
    /// Replaces with literal.
    Literal,
}

/// Replace specifier given current cursor and parts.
fn replace_spec(spec: &Spec, cursor: usize, parts: &[&str]) -> (usize, String) {
    let idx = spec.index.unwrap_or_else(|| cursor);
    let r: &str = if let Some(replace) = &spec.replace {
        &replace
    } else {
        parts[idx]
    };
    let r = if let Some(formatter) = &spec.formatter {
        formatter.format(spec_input_type(&spec), r)
    } else {
        r.to_owned()
    };
    (idx, r)
}

/// Extract matchers from elements.
fn matchers_from_elems(elems: &[Elem]) -> Vec<Matcher> {
    let mut matchers = vec![];
    for e in elems {
        if let Elem::Spec(s) = e {
            match s.index {
                Some(i) => {
                    if i == 0 {
                        continue;
                    }
                    if i > matchers.len() {
                        matchers.resize(i, Matcher::Any);
                    }
                    if matchers[i - 1] == Matcher::Any {
                        matchers[i - 1] = s.matcher.clone();
                    }
                }
                None => {
                    matchers.push(s.matcher.clone());
                }
            };
        }
    }
    matchers
}

/// Get input type of spec.
fn spec_input_type(spec: &Spec) -> InputType {
    match spec.matcher {
        Matcher::Number => InputType::Number,
        _ => InputType::String,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{formatter::Formatter, spec::Spec};

    macro_rules! replace_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (s, elems, expected) = $value;
                    assert_eq!(Replacer::new(elems).replace(s).unwrap(), expected);
                }
            )*
        }
    }

    replace_tests!(
        replace_simple: ("a", &[Elem::Lit("b".to_owned())], ("b".to_owned(), ReplaceIndices {
            matches: vec![],
            replaced: vec![0],
            sources: vec![ReplaceSource::Literal],
        })),
        replace_any: ("a", &[Elem::Spec(Spec::new(Matcher::Any))], ("a".to_owned(), ReplaceIndices {
            matches: vec![0],
            replaced: vec![0],
            sources: vec![ReplaceSource::Index(0)],
        })),

        replace_any_replace: ("a1", &[
            Elem::Spec(Spec {
                matcher: Matcher::Any,
                index: None,
                replace: Some("b".to_owned()),
                formatter: None,
            }),
            Elem::Spec(Spec::new(Matcher::Any)),
        ], ("b1".to_owned(), ReplaceIndices {
            matches: vec![0, 1],
            replaced: vec![0, 1],
            sources: vec![ReplaceSource::Index(0), ReplaceSource::Index(1)],
        })),

        replace_number_replace: ("a1a1", &[
            Elem::Spec(Spec::new(Matcher::Any)),
            Elem::Spec(Spec {
                matcher: Matcher::Number,
                index: None,
                replace: Some("2".to_owned()),
                formatter: None,
            }),
        ], ("a1a2".to_owned(), ReplaceIndices {
            matches: vec![0, 3],
            replaced: vec![0, 3],
            sources: vec![ReplaceSource::Index(0), ReplaceSource::Index(1)],
        })),

        replace_swap: ("a1", &[
            Elem::Spec(Spec {
                matcher: Matcher::Any,
                index: Some(2),
                replace: None,
                formatter: None,
            }),
            Elem::Spec(Spec {
                matcher: Matcher::Any,
                index: Some(1),
                replace: None,
                formatter: None,
            }),
        ], ("1a".to_owned(), ReplaceIndices {
            matches: vec![0, 1],
            replaced: vec![0, 1],
            sources: vec![ReplaceSource::Index(1), ReplaceSource::Index(0)],
        })),

        replace_duplicate_entire: ("a1", &[
            Elem::Spec(Spec {
                matcher: Matcher::Any,
                index: Some(1),
                replace: None,
                formatter: None,
            }),
            Elem::Spec(Spec {
                matcher: Matcher::Any,
                index: Some(1),
                replace: None,
                formatter: None,
            }),
        ], ("a1a1".to_owned(), ReplaceIndices {
            matches: vec![0],
            replaced: vec![0, 2],
            sources: vec![ReplaceSource::Index(0), ReplaceSource::Index(0)],
        })),

        replace_duplicate_first: ("a1", &[
            Elem::Spec(Spec {
                matcher: Matcher::Any,
                index: Some(1),
                replace: None,
                formatter: None,
            }),
            Elem::Spec(Spec {
                matcher: Matcher::Any,
                index: Some(1),
                replace: None,
                formatter: None,
            }),
            Elem::Spec(Spec {
                matcher: Matcher::Any,
                index: Some(2),
                replace: None,
                formatter: None,
            }),
        ], ("aa1".to_owned(), ReplaceIndices {
            matches: vec![0, 1],
            replaced: vec![0, 1, 2],
            sources: vec![
                ReplaceSource::Index(0),
                ReplaceSource::Index(0),
                ReplaceSource::Index(1),
            ],
        })),

        replace_duplicate_second: ("a1", &[
            Elem::Spec(Spec {
                matcher: Matcher::Any,
                index: Some(1),
                replace: None,
                formatter: None,
            }),
            Elem::Spec(Spec {
                matcher: Matcher::Any,
                index: Some(2),
                replace: None,
                formatter: None,
            }),
            Elem::Spec(Spec {
                matcher: Matcher::Any,
                index: Some(2),
                replace: None,
                formatter: None,
            }),
        ], ("a11".to_owned(), ReplaceIndices {
            matches: vec![0, 1],
            replaced: vec![0, 1, 2],
            sources: vec![
                ReplaceSource::Index(0),
                ReplaceSource::Index(1),
                ReplaceSource::Index(1),
            ],
        })),

        replace_last_number: ("a1b2", &[
            Elem::Spec(Spec::new(Matcher::Any)),
            Elem::Lit("_".to_owned()),
            Elem::Spec(Spec {
                matcher: Matcher::Number,
                index: Some(2),
                replace: None,
                formatter: None,
            }),
        ], ("a1b_2".to_owned(), ReplaceIndices {
            matches: vec![0, 3],
            replaced: vec![0, 3, 4],
            sources: vec![
                ReplaceSource::Index(0),
                ReplaceSource::Literal,
                ReplaceSource::Index(1),
            ],
        })),

        replace_first_number: ("a1b2", &[
            Elem::Spec(Spec::new(Matcher::Any)),
            Elem::Lit("_".to_owned()),
            Elem::Spec(Spec {
                matcher: Matcher::Number,
                index: Some(2),
                replace: None,
                formatter: None,
            }),
            Elem::Lit("_".to_owned()),
            Elem::Spec(Spec::new(Matcher::Any)),
        ], ("a_1_b2".to_owned(), ReplaceIndices {
            matches: vec![0, 1, 2],
            replaced: vec![0, 1, 2, 3, 4],
            sources: vec![
                ReplaceSource::Index(0),
                ReplaceSource::Literal,
                ReplaceSource::Index(1),
                ReplaceSource::Literal,
                ReplaceSource::Index(2),
            ],
        })),

        replace_after_indexed: ("a1", &[
            Elem::Spec(Spec {
                matcher: Matcher::Any,
                index: Some(1),
                replace: None,
                formatter: None,
            }),
            Elem::Spec(Spec {
                matcher: Matcher::Any,
                index: Some(1),
                replace: None,
                formatter: None,
            }),
            Elem::Spec(Spec::new(Matcher::Any)),
        ], ("aa1".to_owned(), ReplaceIndices {
            matches: vec![0, 1],
            replaced: vec![0, 1, 2],
            sources: vec![
                ReplaceSource::Index(0),
                ReplaceSource::Index(0),
                ReplaceSource::Index(1),
            ],
        })),

        replace_prefix_entire: ("a1", &[
            Elem::Spec(Spec {
                matcher: Matcher::Any,
                index: Some(2),
                replace: None,
                formatter: None,
            }),
            Elem::Lit("-".to_owned()),
            Elem::Spec(Spec {
                matcher: Matcher::Any,
                index: Some(0),
                replace: None,
                formatter: None,
            }),
        ], ("1-a1".to_owned(), ReplaceIndices {
            matches: vec![0, 1],
            replaced: vec![0, 1, 2],
            sources: vec![
                ReplaceSource::Index(1),
                ReplaceSource::Literal,
                ReplaceSource::Entire,
            ],
        })),

        replace_format: ("a1", &[
            Elem::Spec(Spec::new(Matcher::Any)),
            Elem::Spec(Spec {
                matcher: Matcher::Any,
                index: None,
                replace: None,
                formatter: Some(Formatter::with_width(2, '0')),
            }),
        ], ("a01".to_owned(), ReplaceIndices {
            matches: vec![0, 1],
            replaced: vec![0, 1],
            sources: vec![
                ReplaceSource::Index(0),
                ReplaceSource::Index(1),
            ],
        })),

        replace_format_replace: ("a1", &[
            Elem::Spec(Spec::new(Matcher::Any)),
            Elem::Spec(Spec {
                matcher: Matcher::Any,
                index: None,
                replace: Some("2".to_owned()),
                formatter: Some(Formatter::with_width(2, '0')),
            }),
        ], ("a02".to_owned(), ReplaceIndices {
            matches: vec![0, 1],
            replaced: vec![0, 1],
            sources: vec![
                ReplaceSource::Index(0),
                ReplaceSource::Index(1),
            ],
        })),
    );

    macro_rules! matchers_from_elems_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (elems, expected) = $value;
                    assert_eq!(matchers_from_elems(elems), expected);
                }
            )*
        }
    }

    matchers_from_elems_tests!(
        matchers_from_elems_empty: (&[], &[]),
        matchers_from_elems_lit: (&[Elem::Lit("".to_owned())], &[]),
        matchers_from_elems_any: (
            &[Elem::Spec(Spec::new(Matcher::Any))],
            &[Matcher::Any],
        ),

        matchers_from_elems_any_lit_any: (
            &[
                Elem::Spec(Spec::new(Matcher::Any)),
                Elem::Lit("".to_owned()),
                Elem::Spec(Spec::new(Matcher::Any)),
            ],
            &[Matcher::Any, Matcher::Any],
        ),

        matchers_from_elems_indexed: (
            &[
                Elem::Spec(Spec {
                    matcher: Matcher::Any,
                    index: Some(1),
                    replace: None,
                    formatter: None,
                }),
                Elem::Spec(Spec {
                    matcher: Matcher::Any,
                    index: Some(1),
                    replace: None,
                    formatter: None,
                }),
            ],
            &[Matcher::Any],
        ),

        matchers_from_elems_fill: (
            &[
                Elem::Spec(Spec {
                    matcher: Matcher::Any,
                    index: Some(3),
                    replace: None,
                    formatter: None,
                }),
            ],
            &[Matcher::Any, Matcher::Any, Matcher::Any],
        ),

        matchers_from_elems_after_indexed: (
            &[
                Elem::Spec(Spec {
                    matcher: Matcher::Any,
                    index: Some(1),
                    replace: None,
                    formatter: None,
                }),
                Elem::Spec(Spec::new(Matcher::Any)),
            ],
            &[Matcher::Any, Matcher::Any],
        ),

        matchers_from_elems_matcher_preserve: (
            &[
                Elem::Spec(Spec::new(Matcher::Number)),
                Elem::Spec(Spec {
                    matcher: Matcher::Any,
                    index: Some(1),
                    replace: None,
                    formatter: None,
                }),
            ],
            &[Matcher::Number],
        ),

        matchers_from_elems_matcher_override: (
            &[
                Elem::Spec(Spec::new(Matcher::Any)),
                Elem::Spec(Spec {
                    matcher: Matcher::Number,
                    index: Some(1),
                    replace: None,
                    formatter: None,
                }),
            ],
            &[Matcher::Number],
        ),

        matchers_from_elems_entire: (
            &[
                Elem::Spec(Spec {
                    matcher: Matcher::Any,
                    index: Some(0),
                    replace: None,
                    formatter: None,
                }),
                Elem::Spec(Spec::new(Matcher::Any)),
            ],
            &[Matcher::Any],
        ),
    );
}
