/// Tokenize string.
///
/// Each contiguous section of a type of character is treated as a token:
///
/// * Number - Characters representing numbers.
/// * Whitespace - Characters representing whitespaces.
/// * Punctuation - Characters representing ASCII punctuations.
/// * Text - Everything else.
///
/// For example, the string "Hi, number 42." is tokenized as "[Hi][,][ ][number][ ][42][.]".
///
/// # Arguments
///
/// * s - String slice to tokenize.
///
/// # Returns
///
/// A `Vec` of indices pointing to the start of each token, and a corresponding `Vec` of the types
/// of each token.
pub fn tokenize(s: &str) -> (Vec<usize>, Vec<TokenType>) {
    let mut current_token_type = TokenType::Init;
    let mut indices = vec![];
    let mut token_types = vec![];
    for (i, c) in s.chars().enumerate() {
        let tt = token_type(c);
        if current_token_type != tt {
            indices.push(i);
            token_types.push(tt);
            current_token_type = tt;
        }
    }
    (indices, token_types)
}

/// Get token type of character.
fn token_type(c: char) -> TokenType {
    if c.is_ascii_digit() {
        return TokenType::Number;
    }
    if c.is_ascii_whitespace() {
        return TokenType::Whitespace;
    }
    if c.is_ascii_punctuation() {
        return TokenType::Punctuation;
    }
    TokenType::Text
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    /// A special zero-value token type.
    Init,
    /// A number token.
    Number,
    /// A whitespace token.
    Whitespace,
    /// A punctuation token.
    Punctuation,
    /// A text token.
    Text,
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! tokenize_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (s, expected) = $value;
                    assert_eq!(tokenize(s), expected);
                }
            )*
        }
    }

    tokenize_tests!(
        tokenize_text: ("a", (vec![0], vec![TokenType::Text])),
        tokenize_number: ("a1", (vec![0, 1], vec![
            TokenType::Text,
            TokenType::Number,
        ])),
        tokenize_numbers: ("a12", (vec![0, 1], vec![
            TokenType::Text,
            TokenType::Number,
        ])),
        tokenize_alternating: ("a12bc", (vec![0, 1, 3], vec![
            TokenType::Text,
            TokenType::Number,
            TokenType::Text,
        ])),
        tokenize_number_first: ("12bc", (vec![0, 2], vec![
            TokenType::Number,
            TokenType::Text,
        ])),
        tokenize_whitespace: ("12b c", (vec![0, 2, 3, 4], vec![
            TokenType::Number,
            TokenType::Text,
            TokenType::Whitespace,
            TokenType::Text,
        ])),
        tokenize_whitespaces: ("12b \tc", (vec![0, 2, 3, 5], vec![
            TokenType::Number,
            TokenType::Text,
            TokenType::Whitespace,
            TokenType::Text,
        ])),
        tokenize_unicode_whitespace: ("12b　c", (vec![0, 2, 3, 4], vec![
            TokenType::Number,
            TokenType::Text,
            TokenType::Whitespace,
            TokenType::Text,
        ])),
        tokenize_punctuation: ("12b.c", (vec![0, 2, 3, 4], vec![
            TokenType::Number,
            TokenType::Text,
            TokenType::Punctuation,
            TokenType::Text,
        ])),
    );
}
