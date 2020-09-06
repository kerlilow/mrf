pub trait SplitAtIndices {
    /// Extract subsequences by splitting original sequence at indices.
    ///
    /// Each subsequence begins from the given index and ends before the next index.
    /// The last subsequence ends at the end of the original sequence.
    fn split_at_indices(self, indices: &[usize]) -> Vec<Self>
    where
        Self: std::marker::Sized;
}

impl<'a> SplitAtIndices for &'a str {
    fn split_at_indices(self, indices: &[usize]) -> Vec<Self> {
        [indices, &[self.len()]]
            .concat()
            .windows(2)
            .map(|w| &self[w[0]..w[1]])
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! split_at_indices_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (s, indices, expected): (&str, &[usize], &[&str]) = $value;
                    assert_eq!(s.split_at_indices(indices), expected);
                }
            )*
        }
    }

    split_at_indices_tests!(
        split_at_indices_2: ("abc", &[0], &["abc"]),
        split_at_indices_1: ("abc", &[0, 1, 2], &["a", "b", "c"]),
        split_at_indices_3: ("abc", &[0, 2], &["ab", "c"]),
        split_at_indices_empty_str: ("", &[0], &[""]),
        split_at_indices_empty_indices: ("", &[], &[]),
    );
}
