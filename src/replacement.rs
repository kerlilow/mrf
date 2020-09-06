use colored::*;
use std::borrow::Cow;
use std::cmp;
use std::error::Error;
use std::fmt;

use crate::{
    indices::SplitAtIndices,
    parser::parse,
    replacer::{ReplaceSource, Replacer},
};

const DEFAULT_MAX_PREVIEWS: usize = 5;

const COLOR_MAP: [&str; 5] = ["cyan", "green", "yellow", "red", "magenta"];

pub type Replacement<'a> = (Cow<'a, str>, String);

#[derive(Default)]
pub struct ResolveOpts {
    pub highlight: bool,
}

impl ResolveOpts {
    pub fn new() -> Self {
        Self { highlight: false }
    }

    pub fn with_highlight() -> Self {
        Self { highlight: true }
    }
}

/// Resolve replacements by parsing elements from `replacement` and applying replacer to each item.
///
/// # Arguments
///
/// * `items` - Items.
/// * `replacer_str` - Replacer string.
/// * `opts` - Options.
///
/// # Returns
///
/// A `Result` containing a `Vec` of replacements.
pub fn resolve<'a, T>(
    items: &'a [T],
    replacer_str: &str,
    opts: ResolveOpts,
) -> Result<Vec<Replacement<'a>>, Box<dyn Error>>
where
    T: AsRef<str> + cmp::PartialEq + std::clone::Clone,
{
    let elems = parse(replacer_str)?;
    let replacer = Replacer::new(&elems);
    Ok(if opts.highlight {
        replace_items_highlight(&replacer, items)
    } else {
        replace_items(&replacer, items)
    })
}

/// Apply replacer to each item.
fn replace_items<'a, T>(replacer: &Replacer, items: &'a [T]) -> Vec<Replacement<'a>>
where
    T: AsRef<str> + cmp::PartialEq + std::clone::Clone,
{
    items
        .iter()
        .filter_map(|left| {
            replacer
                .replace(left.as_ref())
                .map(|(right, _)| (Cow::Borrowed(left.as_ref()), right))
                .ok()
        })
        .collect()
}

/// Apply replacer to each item with match highlighting.
fn replace_items_highlight<'a, T>(replacer: &Replacer, items: &'a [T]) -> Vec<Replacement<'a>>
where
    T: AsRef<str> + cmp::PartialEq + std::clone::Clone,
{
    items
        .iter()
        .filter_map(|left| {
            replacer
                .replace(left.as_ref())
                .map(|(right, indices)| {
                    (
                        Cow::Owned(apply_color_map(left.as_ref(), &indices.matches)),
                        apply_replaced_color_map(
                            right.as_ref(),
                            &indices.replaced,
                            &indices.sources,
                        ),
                    )
                })
                .ok()
        })
        .collect()
}

/// Apply color map to string.
fn apply_color_map(s: &str, indices: &[usize]) -> String {
    s.split_at_indices(indices)
        .iter()
        .enumerate()
        .map(|(i, p)| p.color(COLOR_MAP[i % COLOR_MAP.len()]).to_string())
        .collect::<Vec<String>>()
        .join("")
}

/// Apply color map to replaced string.
fn apply_replaced_color_map(s: &str, indices: &[usize], sources: &[ReplaceSource]) -> String {
    s.split_at_indices(indices)
        .iter()
        .enumerate()
        .map(|(i, p)| match sources[i] {
            ReplaceSource::Index(matcher_i) => {
                p.color(COLOR_MAP[matcher_i % COLOR_MAP.len()]).to_string()
            }
            _ => p.normal().to_string(),
        })
        .collect::<Vec<String>>()
        .join("")
}

#[derive(Default)]
pub struct PreviewOpts {
    pub max_previews: usize,
    pub highlight: bool,
}

impl PreviewOpts {
    pub fn new() -> Self {
        Self {
            max_previews: DEFAULT_MAX_PREVIEWS,
            highlight: true,
        }
    }
}

/// Return a formatted preview of replacements, useful for confirmation with user.
///
/// # Arguments
///
/// * `items` - Items.
/// * `replacer_str` - Replacer string.
/// * `opts` - Options.
///
/// # Returns
///
/// A `Result` containing the preview string.
pub fn previews<T>(
    items: &[T],
    replacer_str: &str,
    opts: PreviewOpts,
) -> Result<String, Box<dyn Error>>
where
    T: AsRef<str> + cmp::PartialEq + fmt::Display,
{
    let (head, tail) = if items.len() > opts.max_previews {
        (opts.max_previews / 2, ((opts.max_previews - 1) / 2))
    } else {
        (items.len(), 0)
    };
    let preview_items = take_ends(items, head, tail);
    let replacements = resolve(
        &preview_items,
        replacer_str,
        ResolveOpts {
            highlight: opts.highlight,
        },
    )?;
    let mut lines = vec![];
    for (left, right) in replacements.iter().take(head) {
        lines.push(format!("    {} -> {}", left, right));
    }
    if tail != 0 {
        lines.push("    ...".to_owned());
        for (left, right) in replacements.iter().rev().take(tail).rev() {
            lines.push(format!("    {} -> {}", left, right));
        }
    }
    Ok(lines.join("\n"))
}

/// Take some items from each end.
///
/// # Arguments
///
/// * `items` - Items.
/// * `head` - Number of items to take from the beginning.
/// * `tail` - Number of items to take from the end.
///
/// # Returns
///
/// Items.
fn take_ends<'a, T>(items: &'a [T], head: usize, tail: usize) -> Vec<&'a T> {
    items
        .iter()
        .take(head)
        .chain(items.iter().rev().take(tail).rev())
        .collect()
}
