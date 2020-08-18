use std::error::Error;

use crate::{
    elem::Elem,
    matcher::{match_all, Matcher},
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
    /// A `Result` containing the replaced string.
    pub fn replace(&self, s: &str) -> Result<String, Box<dyn Error>> {
        let indices = match_all(s, &self.matchers)?;
        let parts: Vec<&str> = [vec![s], indices_to_strs(s, &indices)].concat();
        let mut cursor = 1;
        Ok(self
            .elems
            .iter()
            .map(|e| match e {
                Elem::Spec(spec) => {
                    let idx = spec.index.unwrap_or_else(|| cursor);
                    cursor = idx + 1;
                    let r: &str = if let Some(replace) = &spec.replace {
                        &replace
                    } else {
                        parts[idx]
                    };
                    if let Some(formatter) = &spec.formatter {
                        formatter.format(r)
                    } else {
                        r.to_owned()
                    }
                }
                Elem::Lit(lit) => lit.to_owned(),
            })
            .collect::<Vec<String>>()
            .join(""))
    }
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

/// Extract string slices from indices.
///
/// Each string slice begins from the given index and ends before the next index.
/// The last string slice ends at the end of the original string slice.
fn indices_to_strs<'a>(s: &'a str, indices: &[usize]) -> Vec<&'a str> {
    [indices, &[s.len()]]
        .concat()
        .windows(2)
        .map(|w| &s[w[0]..w[1]])
        .collect()
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
        replace_simple: ("a", &[Elem::Lit("b".to_owned())], "b"),
        replace_any: ("a", &[Elem::Spec(Spec::new(Matcher::Any))], "a"),

        replace_any_replace: ("a1", &[
            Elem::Spec(Spec {
                matcher: Matcher::Any,
                index: None,
                replace: Some("b".to_owned()),
                formatter: None,
            }),
            Elem::Spec(Spec::new(Matcher::Any)),
        ], "b1"),

        replace_number_replace: ("a1a1", &[
            Elem::Spec(Spec::new(Matcher::Any)),
            Elem::Spec(Spec {
                matcher: Matcher::Number,
                index: None,
                replace: Some("2".to_owned()),
                formatter: None,
            }),
        ], "a1a2"),

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
        ], "1a"),

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
        ], "a1a1"),

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
        ], "aa1"),

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
        ], "a11"),

        replace_last_number: ("a1b2", &[
            Elem::Spec(Spec::new(Matcher::Any)),
            Elem::Lit("_".to_owned()),
            Elem::Spec(Spec {
                matcher: Matcher::Number,
                index: Some(2),
                replace: None,
                formatter: None,
            }),
        ], "a1b_2"),

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
        ], "a_1_b2"),

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
        ], "aa1"),

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
        ], "1-a1"),

        replace_format: ("a1", &[
            Elem::Spec(Spec::new(Matcher::Any)),
            Elem::Spec(Spec {
                matcher: Matcher::Any,
                index: None,
                replace: None,
                formatter: Some(Formatter { fill: '0', width: 2 }),
            }),
        ], "a01"),

        replace_format_replace: ("a1", &[
            Elem::Spec(Spec::new(Matcher::Any)),
            Elem::Spec(Spec {
                matcher: Matcher::Any,
                index: None,
                replace: Some("2".to_owned()),
                formatter: Some(Formatter { fill: '0', width: 2 }),
            }),
        ], "a02"),
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

    macro_rules! indices_to_strs_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (s, indices, expected): (&str, &[usize], &[&str]) = $value;
                    assert_eq!(indices_to_strs(s, indices), expected);
                }
            )*
        }
    }

    indices_to_strs_tests!(
        indices_to_strs_2: ("abc", &[0], &["abc"]),
        indices_to_strs_1: ("abc", &[0, 1, 2], &["a", "b", "c"]),
        indices_to_strs_3: ("abc", &[0, 2], &["ab", "c"]),
        indices_to_strs_empty_str: ("", &[0], &[""]),
        indices_to_strs_empty_indices: ("", &[], &[]),
    );
}
