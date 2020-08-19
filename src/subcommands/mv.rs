use std::error::Error;

use clap::{AppSettings, Clap};
use dialoguer::Confirm;
use indicatif::{ParallelProgressIterator, ProgressBar};
use rayon::prelude::*;

use super::utils::{items_from_opt, replacement_previews, resolve_replacements, setup_rayon};

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    #[clap(short, long)]
    yes: bool,
    #[clap(short, long)]
    concurrency: Option<usize>,
    #[clap(required = true)]
    item: Vec<String>,
    replacer: String,
}

/// Run move subcommand.
pub fn run(opts: Opts) -> Result<(), Box<dyn Error>> {
    let concurrency = opts.concurrency.unwrap_or(0);
    setup_rayon(concurrency)?;
    let items = items_from_opt(opts.item)?;
    let replacements = resolve_replacements(&items, &opts.replacer)?;
    if !opts.yes {
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
