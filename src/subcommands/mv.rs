use std::error::Error;

use clap::{AppSettings, Clap};
use dialoguer::Confirm;
use indicatif::{ParallelProgressIterator, ProgressBar};
use rayon::prelude::*;

use super::utils::{items_from_opt, setup_rayon};

use crate::replacement::{previews, resolve, PreviewOpts, ResolveOpts};

/// Move each file according to the replacer.
///
/// Note: Currently this subcommand only supports moving within the same filesystem. To move
/// between filesystems, use `mrf exec mv` instead.
///
/// Examples:
///
/// 1. Replace hyphen with underscore:
///
///     $ mrf mv * '{}{=_}{}'
///     Moving 1 out of 1 items:
///         image-001.jpg -> image_001.jpg
///
/// 2. Rename while keeping numbering:
///
///     $ mrf mv * '{=photo}{}'
///     Moving 1 out of 1 items:
///         image-001.jpg -> photo-001.jpg
///
/// 3. Add zero padding:
///
///     $ mrf mv * '{}{n:03}{}'
///     Moving 1 out of 1 items:
///         image-1.jpg -> image-001.jpg
#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp, verbatim_doc_comment)]
pub struct Opts {
    /// Assume yes as answer to all prompts and run non-interactively.
    #[clap(short = "y", long)]
    assume_yes: bool,
    /// Number of threads to use.
    #[clap(short, long)]
    concurrency: Option<usize>,
    /// Files to move. Pass "-" to read from stdin.
    #[clap(required = true)]
    item: Vec<String>,
    /// Replacer string.
    replacer: String,
}

/// Run move subcommand.
pub fn run(opts: Opts) -> Result<(), Box<dyn Error>> {
    let concurrency = opts.concurrency.unwrap_or(0);
    setup_rayon(concurrency)?;
    let items = items_from_opt(opts.item)?;
    let replacements = resolve(&items, &opts.replacer, ResolveOpts::new())?;
    if !opts.assume_yes {
        println!(
            "Moving {} out of {} items:",
            replacements.len(),
            items.len()
        );
        println!("{}", previews(&items, &opts.replacer, PreviewOpts::new())?);
        if !Confirm::new()
            .with_prompt("Do you want to continue?")
            .default(false)
            .interact()?
        {
            return Ok(());
        }
    }
    replacements
        .par_iter()
        .progress_with(ProgressBar::new(replacements.len() as u64))
        .for_each(|(left, right)| {
            std::fs::rename(left.as_ref(), right).unwrap_or_else(|e| {
                eprintln!("{}", e);
            })
        });
    Ok(())
}
