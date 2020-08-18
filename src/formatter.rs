use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Formatter {
    pub fill: char,
    pub width: usize,
}

impl Formatter {
    pub fn format(&self, s: &str) -> String {
        if s.len() >= self.width {
            return s.to_owned();
        }
        [
            self.fill.to_string().repeat(self.width - s.len()),
            s.to_owned(),
        ]
        .concat()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! format_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (fill, width, s, expected) = $value;
                    assert_eq!(Formatter { fill, width }.format(s), expected);
                }
            )*
        }
    }

    format_tests!(
        format_empty: (' ', 0, "", ""),
        format_no_pad: (' ', 1, "a", "a"),
        format_simple: (' ', 2, "a", " a"),
        format_number: ('0', 4, "1", "0001"),
    );
}
