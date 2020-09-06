use std::error::Error;
use std::process::Command;

use clap::{AppSettings, Clap};
use dialoguer::Confirm;
use indicatif::{ParallelProgressIterator, ProgressBar};
use rayon::prelude::*;

use super::utils::{items_from_opt, setup_rayon};

use crate::command;
use crate::replacement::{previews, resolve, PreviewOpts, ResolveOpts};

/// Execute the given command with each replaced item.
///
/// Examples:
///
/// 1. Make directory:
///
///     $ mrf exec -r 'mkdir -p' * '{3}{=}'
///     Matched 1 out of 1 items:
///         image-2020-01-01.jpg -> 2020
///
/// 2. Copy files:
///
///     $ mrf exec cp * '{}{=_}{}'
///     Matched 1 out of 1 items:
///         image-001.jpg -> image_001.jpg
#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp, verbatim_doc_comment)]
pub struct Opts {
    /// Assume yes as answer to all prompts and run non-interactively.
    #[clap(short = "y", long)]
    assume_yes: bool,
    /// Only pass the input string (left-hand side of mapping) to the command.
    #[clap(short = "l", long, conflicts_with = "right-only")]
    left_only: bool,
    /// Only pass the replaced string (right-hand side of mapping) to the command.
    #[clap(short = "r", long, conflicts_with = "left-only")]
    right_only: bool,
    /// Number of threads to use.
    #[clap(short, long)]
    concurrency: Option<usize>,
    /// Command to run. To pass arguments to the command, quote the command (e.g. "mkdir -p").
    command: String,
    /// Items to replace. Pass "-" to read from stdin.
    #[clap(required = true)]
    item: Vec<String>,
    /// Replacer string.
    replacer: String,
}

#[derive(Clone)]
struct OutputOpts {
    left_only: bool,
    right_only: bool,
}

/// Run exec subcommand.
pub fn run(opts: Opts) -> Result<(), Box<dyn Error>> {
    let concurrency = opts.concurrency.unwrap_or(0);
    setup_rayon(concurrency)?;
    let items = items_from_opt(opts.item)?;
    let replacements = resolve(&items, &opts.replacer, ResolveOpts::new())?;
    if !opts.assume_yes {
        println!(
            "Matched {} out of {} items:",
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
    let args = command::parse(&opts.command)?;
    let output_opts = OutputOpts {
        left_only: opts.left_only,
        right_only: opts.right_only,
    };
    replacements
        .par_iter()
        .progress_with(ProgressBar::new(replacements.len() as u64))
        .for_each(|(left, right)| {
            do_exec(&output_opts, &args, left, right).unwrap_or_else(|e| {
                eprintln!("{}", e);
            })
        });
    Ok(())
}

/// Execute command with args and replacement.
fn do_exec(
    opts: &OutputOpts,
    args: &[String],
    left: &str,
    right: &str,
) -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::new(&args[0]);
    cmd.args(&args[1..]);
    if opts.left_only {
        cmd.arg(left);
    } else if opts.right_only {
        cmd.arg(right);
    } else {
        cmd.args(&[left, right]);
    }
    cmd.spawn()?.wait()?;
    Ok(())
}
