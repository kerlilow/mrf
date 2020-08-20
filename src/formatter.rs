use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Formatter {
    fill: char,
    width: usize,
}

pub enum InputType {
    /// String.
    String,
    /// Number. Leading zeros are trimmed on format.
    Number,
}

impl Formatter {
    /// Create a `Formatter`.
    ///
    /// # Returns
    ///
    /// A `Formatter`.
    pub fn new() -> Self {
        Self {
            width: 0,
            fill: ' ',
        }
    }

    /// Create a `Formatter` with width.
    ///
    /// # Arguments
    ///
    /// * `width` - Minimum width.
    /// * `fill` - Character to pad with when string length is less than `width`.
    ///
    /// # Returns
    ///
    /// A `Formatter` with the specified width and fill.
    pub fn with_width(width: usize, fill: char) -> Self {
        Self { width, fill }
    }

    /// Format string.
    ///
    /// # Arguments
    ///
    /// * `input_type` - Treat `s` as `input_type`. If `Number`, leading zeros are trimmed.
    /// * `s` - Input string.
    ///
    /// # Returns
    ///
    /// The formatted string.
    pub fn format(&self, input_type: InputType, s: &str) -> String {
        let s = match input_type {
            InputType::String => s,
            InputType::Number => s.trim_start_matches('0'),
        };
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

impl Default for Formatter {
    fn default() -> Self {
        Self::new()
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
                    let (width, fill, input_type, s, expected) = $value;
                    assert_eq!(Formatter::with_width(width, fill).format(input_type, s), expected);
                }
            )*
        }
    }

    format_tests!(
        format_empty: (0, ' ', InputType::String, "", ""),
        format_no_pad: (1, ' ', InputType::String, "a", "a"),
        format_simple: (2, ' ', InputType::String, "a", " a"),
        format_number: (4, '0', InputType::Number, "1", "0001"),
        format_number_string: (4, '0', InputType::String, "1", "0001"),
        format_number_truncate_zeros: (2, '0', InputType::Number, "0001", "01"),
        format_number_string_no_truncate_zeros: (2, '0', InputType::String, "0001", "0001"),
        format_number_no_truncate_non_zeros: (2, '0', InputType::Number, "1234", "1234"),
    );
}
