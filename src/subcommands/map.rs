use std::borrow::Cow;
use std::error::Error;

use clap::{AppSettings, Clap};

use super::utils::items_from_opt;

use crate::replacement::{resolve, ResolveOpts};

/// Map each item according to the replacer.
///
/// Examples:
///
/// 1. Replace hyphen with underscore:
///
///     $ mrf map example-001 '{}{=_}{}'
///     example-001 -> example_001
///
/// 2. Pipe to cp (consider using the "exec" subcommand instead):
///
///     $ mrf map * '{}{=-}{}' | xargs -0 -n2 cp
#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp, verbatim_doc_comment)]
pub struct Opts {
    /// Only output the input string (left-hand side of mapping).
    #[clap(short = "l", long, conflicts_with = "right-only")]
    left_only: bool,
    /// Only output the replaced string (right-hand side of mapping).
    #[clap(short = "r", long, conflicts_with = "left-only")]
    right_only: bool,
    /// Items to replace. Pass "-" to read from stdin.
    #[clap(required = true)]
    item: Vec<String>,
    /// Replacer string.
    replacer: String,
}

/// Run map (`map`) subcommand.
pub fn run(opts: Opts) -> Result<(), Box<dyn Error>> {
    let items = items_from_opt(opts.item)?;
    let print: fn(&(Cow<'_, str>, String)) = if atty::is(atty::Stream::Stdout) {
        if opts.left_only {
            |(left, _)| println!("{}", left)
        } else if opts.right_only {
            |(_, right)| println!("{}", right)
        } else {
            |(left, right)| println!("{} -> {}", left, right)
        }
    } else if opts.left_only {
        |(left, _)| print!("{}\0", left)
    } else if opts.right_only {
        |(_, right)| print!("{}\0", right)
    } else {
        |(left, right)| print!("{}\0{}\0", left, right)
    };
    resolve(
        &items,
        &opts.replacer,
        ResolveOpts {
            highlight: atty::is(atty::Stream::Stdout),
        },
    )?
    .iter()
    .for_each(print);
    Ok(())
}
