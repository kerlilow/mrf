use std::error::Error;

use clap::{AppSettings, Clap};

use super::utils::{items_from_opt, resolve_replacements};

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    #[clap(short = "l", long)]
    output_left: bool,
    #[clap(short = "r", long)]
    output_right: bool,
    #[clap(required = true)]
    item: Vec<String>,
    replacer: String,
}

/// Run map (`map`) subcommand.
pub fn run(opts: Opts) -> Result<(), Box<dyn Error>> {
    let items = items_from_opt(opts.item)?;
    let replacements = resolve_replacements(&items, &opts.replacer)?;
    let print: fn(&str, &str) = if atty::is(atty::Stream::Stdout) {
        if opts.output_left && !opts.output_right {
            |left, _| println!("{}", left)
        } else if !opts.output_left && opts.output_right {
            |_, right| println!("{}", right)
        } else {
            |left, right| println!("{} -> {}", left, right)
        }
    } else if opts.output_left && !opts.output_right {
        |left, _| print!("{}\0", left)
    } else if !opts.output_left && opts.output_right {
        |_, right| print!("{}\0", right)
    } else {
        |left, right| print!("{}\0{}\0", left, right)
    };
    for (left, right) in replacements {
        print(left, &right);
    }
    Ok(())
}
