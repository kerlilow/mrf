use std::fmt::Debug;

use crate::{formatter::Formatter, matcher::Matcher};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spec {
    pub matcher: Matcher,
    pub index: Option<usize>,
    pub replace: Option<String>,
    pub formatter: Option<Formatter>,
}

impl Spec {
    pub fn new(matcher: Matcher) -> Self {
        Self {
            matcher,
            index: None,
            replace: None,
            formatter: None,
        }
    }
}
