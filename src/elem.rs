use crate::spec::Spec;

/// Element, either a literal or a specifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Elem {
    /// Literal.
    Lit(String),
    /// Specifier.
    Spec(Spec),
}
