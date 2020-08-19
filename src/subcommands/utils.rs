use std::cmp;
use std::error::Error;
use std::fmt;
use std::io::BufRead;

use crate::{parser::parse, replacer::Replacer};

const MAX_PREVIEWS: usize = 5;

/// Setup rayon (initialize threadpools according to concurrency).
pub fn setup_rayon(concurrency: usize) -> Result<(), Box<dyn Error>> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(concurrency)
        .build_global()?;
    Ok(())
}

/// If items contain a single string "-", read items from stdin, otherwise return as-is.
pub fn items_from_opt(items: Vec<String>) -> Result<Vec<String>, std::io::Error> {
    Ok(if items.len() == 1 && items[0] == "-" {
        read_items_from_stdin()?
    } else {
        items
    })
}

/// Read items from stdin, one item per line.
pub fn read_items_from_stdin() -> Result<Vec<String>, std::io::Error> {
    let mut items: Vec<String> = vec![];
    for line in std::io::stdin().lock().lines() {
        items.push(line?);
    }
    Ok(items)
}

/// Resolve replacements by parsing elements from `replacement` and applying replacer to each item.
pub fn resolve_replacements<'a, T: AsRef<str> + cmp::PartialEq>(
    items: &'a [T],
    replacer_str: &str,
) -> Result<Vec<(&'a T, String)>, Box<dyn Error>> {
    let elems = parse(replacer_str)?;
    let replacer = Replacer::new(&elems);
    Ok(replace_items(&replacer, items))
}

/// Apply replacer to each item.
pub fn replace_items<'a, T: AsRef<str>>(
    replacer: &Replacer,
    items: &'a [T],
) -> Vec<(&'a T, String)> {
    items
        .iter()
        .filter_map(|left| {
            replacer
                .replace(left.as_ref())
                .map(|right| (left, right))
                .ok()
        })
        .collect()
}

/// Return a formatted preview of replacements, useful for confirmation with user.
pub fn replacement_previews<T, U>(replacements: &[(T, U)]) -> String
where
    T: AsRef<str> + fmt::Display,
    U: AsRef<str> + fmt::Display,
{
    let mut lines = vec![];
    if replacements.len() > MAX_PREVIEWS {
        let head_count = (MAX_PREVIEWS - 1) / 2;
        let tail_count = MAX_PREVIEWS - 1 - head_count;
        for (left, right) in replacements.iter().take(head_count) {
            lines.push(format!("    {} -> {}", left, right));
        }
        lines.push("    ...".to_owned());
        for (left, right) in replacements.iter().rev().take(tail_count).rev() {
            lines.push(format!("    {} -> {}", left, right));
        }
    } else {
        for (left, right) in replacements.iter() {
            lines.push(format!("    {} -> {}", left, right));
        }
    }
    lines.join("\n")
}
