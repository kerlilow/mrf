use std::error::Error;
use std::process::Command;

use clap::{AppSettings, Clap};
use crossbeam_channel::{bounded, Receiver};
use crossbeam_utils::thread;
use dialoguer::Confirm;

use super::utils::{items_from_opt, replacement_previews, resolve_replacements};

use crate::command;

const CHANNEL_CAP_MULTIPLIER: usize = 10;

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
    let concurrency = opts.concurrency.unwrap_or_else(num_cpus::get);
    let (s, r) = bounded::<(&str, String)>(concurrency * CHANNEL_CAP_MULTIPLIER);
    let output_opts = OutputOpts {
        output_left: opts.output_left,
        output_right: opts.output_right,
    };
    thread::scope(|scope| {
        let mut ts = vec![];
        for _ in 0..concurrency {
            let r = r.clone();
            ts.push(scope.spawn(|_| {
                run_exec_thread(r, &args, &output_opts).unwrap();
            }));
        }
        for (left, right) in replacements {
            s.send((left, right)).unwrap();
        }
        drop(s);
    })
    .unwrap();
    Ok(())
}

/// Run exec thread.
fn run_exec_thread(
    r: Receiver<(&str, String)>,
    args: &[String],
    opts: &OutputOpts,
) -> Result<(), Box<dyn Error>> {
    while let Ok((left, right)) = r.recv() {
        let mut cmd = Command::new(&args[0]);
        cmd.args(&args[1..]);
        if opts.output_left && !opts.output_right {
            cmd.arg(left);
        } else if !opts.output_left && opts.output_right {
            cmd.arg(right);
        } else {
            cmd.args(&[left, &right]);
        }
        cmd.spawn()?.wait()?;
    }
    Ok(())
}
