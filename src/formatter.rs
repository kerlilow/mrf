use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Formatter {
    pub fill: char,
    pub width: usize,
}

pub enum InputType {
    String,
    Number,
}

impl Formatter {
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

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! format_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (fill, width, input_type, s, expected) = $value;
                    assert_eq!(Formatter { fill, width }.format(input_type, s), expected);
                }
            )*
        }
    }

    format_tests!(
        format_empty: (' ', 0, InputType::String, "", ""),
        format_no_pad: (' ', 1, InputType::String, "a", "a"),
        format_simple: (' ', 2, InputType::String, "a", " a"),
        format_number: ('0', 4, InputType::Number, "1", "0001"),
        format_number_string: ('0', 4, InputType::String, "1", "0001"),
        format_number_truncate_zeros: ('0', 2, InputType::Number, "0001", "01"),
        format_number_string_no_truncate_zeros: ('0', 2, InputType::String, "0001", "0001"),
        format_number_no_truncate_non_zeros: ('0', 2, InputType::Number, "1234", "1234"),
    );
}
