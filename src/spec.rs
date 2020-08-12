use crate::matcher::Matcher;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spec {
    pub matcher: Matcher,
    pub index: Option<usize>,
    pub replace: Option<String>,
    pub format: Option<String>,
}

impl Spec {
    pub fn new(matcher: Matcher) -> Self {
        Self {
            matcher,
            index: None,
            replace: None,
            format: None,
        }
    }
}
