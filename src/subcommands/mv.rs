use std::error::Error;

use clap::{AppSettings, Clap};
use crossbeam_channel::{bounded, Receiver};
use crossbeam_utils::thread;
use dialoguer::Confirm;

use super::utils::{items_from_opt, replacement_previews, resolve_replacements};

const CHANNEL_CAP_MULTIPLIER: usize = 10;

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
    let concurrency = opts.concurrency.unwrap_or_else(num_cpus::get);
    let (s, r) = bounded::<(&str, String)>(concurrency * CHANNEL_CAP_MULTIPLIER);
    thread::scope(|scope| {
        let mut ts = vec![];
        for _ in 0..concurrency {
            let r = r.clone();
            ts.push(scope.spawn(|_| {
                run_move_thread(r).unwrap();
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

/// Run move thread.
fn run_move_thread(r: Receiver<(&str, String)>) -> Result<(), Box<dyn Error>> {
    while let Ok((left, right)) = r.recv() {
        std::fs::rename(left, right)?;
    }
    Ok(())
}
