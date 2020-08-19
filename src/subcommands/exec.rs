use std::error::Error;
use std::process::Command;

use clap::{AppSettings, Clap};
use dialoguer::Confirm;
use rayon::prelude::*;

use super::utils::{items_from_opt, replacement_previews, resolve_replacements, setup_rayon};

use crate::command;

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    #[clap(short, long)]
    yes: bool,
    #[clap(short = "l", long)]
    output_left: bool,
    #[clap(short = "r", long)]
    output_right: bool,
    #[clap(short, long)]
    concurrency: Option<usize>,
    command: String,
    #[clap(required = true)]
    item: Vec<String>,
    replacer: String,
}

#[derive(Clone)]
struct OutputOpts {
    output_left: bool,
    output_right: bool,
}

/// Run exec subcommand.
pub fn run(opts: Opts) -> Result<(), Box<dyn Error>> {
    let concurrency = opts.concurrency.unwrap_or(0);
    setup_rayon(concurrency)?;
    let items = items_from_opt(opts.item)?;
    let replacements = resolve_replacements(&items, &opts.replacer)?;
    if !opts.yes {
        println!(
            "Matched {} out of {} items:",
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
    let args = command::parse(&opts.command)?;
    let output_opts = OutputOpts {
        output_left: opts.output_left,
        output_right: opts.output_right,
    };
    replacements
        .par_iter()
        .for_each(|(left, right)| do_exec(&output_opts, &args, left, right).unwrap());
    Ok(())
}

fn do_exec(
    opts: &OutputOpts,
    args: &[String],
    left: &str,
    right: &str,
) -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::new(&args[0]);
    cmd.args(&args[1..]);
    if opts.output_left && !opts.output_right {
        cmd.arg(left);
    } else if !opts.output_left && opts.output_right {
        cmd.arg(right);
    } else {
        cmd.args(&[left, right]);
    }
    cmd.spawn()?.wait()?;
    Ok(())
}
