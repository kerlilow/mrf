use std::error::Error;

use clap::{AppSettings, Clap};
use dialoguer::Confirm;
use indicatif::{ParallelProgressIterator, ProgressBar};
use rayon::prelude::*;

use super::utils::{items_from_opt, replacement_previews, resolve_replacements, setup_rayon};

/// Move each file according to the replacer.
///
/// Note: Currently this subcommand only supports moving within the same filesystem. To move
/// between filesystems, use `mrf exec mv` instead.
#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    /// Assume yes as answer to all prompts and run non-interactively.
    #[clap(short = "y", long)]
    assume_yes: bool,
    /// Number of threads to use.
    #[clap(short, long)]
    concurrency: Option<usize>,
    /// Files to move.
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
    let replacements = resolve_replacements(&items, &opts.replacer)?;
    if !opts.assume_yes {
        println!(
            "Moving {} out of {} items:",
            replacements.len(),
            items.len()
        );
        println!("{}", replacement_previews(&replacements));
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
        .for_each(|(left, right)| std::fs::rename(left, right).unwrap());
    Ok(())
}
