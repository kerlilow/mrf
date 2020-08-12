use std::error::Error;
use std::fmt;
use std::process::Command;

use clap::{AppSettings, Clap};
use dialoguer::Confirm;

use mrf::{parser::parse, replacer::Replacer};

const MAX_PREVIEWS: usize = 5;

#[derive(Clap)]
#[clap(
    version = "0.1",
    author = "Kerli Low <kerlilow@gmail.com>",
    setting = AppSettings::ColoredHelp,
)]
struct Opts {
    #[clap(subcommand)]
    subcmd: Subcommand,
}

#[derive(Clap)]
enum Subcommand {
    Map(MapOpts),
    Exec(ExecOpts),
}

#[derive(Clap)]
struct MapOpts {
    #[clap(short = "l", long)]
    output_left: bool,
    #[clap(short = "r", long)]
    output_right: bool,
    #[clap(required = true)]
    item: Vec<String>,
    replacer: String,
}

#[derive(Clap)]
struct ExecOpts {
    #[clap(short, long)]
    yes: bool,
    #[clap(short = "l", long)]
    output_left: bool,
    #[clap(short = "r", long)]
    output_right: bool,
    command: String,
    #[clap(required = true)]
    item: Vec<String>,
    replacer: String,
}

fn main() {
    let opts = Opts::parse();
    match opts.subcmd {
        Subcommand::Map(sub_opts) => handle_map(sub_opts),
        Subcommand::Exec(sub_opts) => handle_exec(sub_opts),
    }
    .unwrap_or_else(|e| println!("An error occurred:\n{}", e));
}

/// Handle map (`map`) subcommand.
fn handle_map(opts: MapOpts) -> Result<(), Box<dyn Error>> {
    let replacements = resolve_replacements(&opts.item, &opts.replacer)?;
    let print = if atty::is(atty::Stream::Stdout) {
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
        print(left, right);
    }
    Ok(())
}

/// Handle exec subcommand.
fn handle_exec(opts: ExecOpts) -> Result<(), Box<dyn Error>> {
    let replacements = resolve_replacements(&opts.item, &opts.replacer)?;
    if !opts.yes {
        println!(
            "Matched {} out of {} items:",
            replacements.len(),
            opts.item.len()
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
    let args = opts.command.split(' ').collect::<Vec<&str>>();
    for (left, right) in &replacements {
        let mut cmd = Command::new(args[0]);
        cmd.args(&args[1..]);
        if opts.output_left && !opts.output_right {
            cmd.arg(left);
        } else if !opts.output_left && opts.output_right {
            cmd.arg(&right);
        } else {
            cmd.args(&[left, &right]);
        }
        cmd.spawn()?;
    }
    Ok(())
}

/// Resolve replacements by parsing elements from `replacement` and applying replacer to each
/// string.
fn resolve_replacements<'a, T: AsRef<str>>(
    items: &'a [T],
    replacer_str: &str,
) -> Result<Vec<(&'a T, String)>, Box<dyn Error>> {
    let elems = parse(replacer_str)?;
    let replacer = Replacer::new(&elems);
    Ok(items
        .iter()
        .filter_map(|left| {
            replacer
                .replace(left.as_ref())
                .map(|right| (left, right))
                .ok()
        })
        .collect())
}

/// Return a formatted preview of replacements, useful for confirmation with user.
fn replacement_previews<T, U>(replacements: &[(T, U)]) -> String
where
    T: AsRef<str> + fmt::Display,
    U: AsRef<str> + fmt::Display,
{
    let mut lines = vec![];
    if replacements.len() > MAX_PREVIEWS {
        let head_count = (MAX_PREVIEWS - 1) / 2;
        let tail_count = MAX_PREVIEWS - 1 - head_count;
        for (left, right) in replacements.iter().take(head_count) {
            lines.push(format!("    {} -> {}", left, right));
        }
        lines.push("    ...".to_owned());
        for (left, right) in replacements.iter().rev().take(tail_count).rev() {
            lines.push(format!("    {} -> {}", left, right));
        }
    } else {
        for (left, right) in replacements.iter() {
            lines.push(format!("    {} -> {}", left, right));
        }
    }
    lines.join("\n")
}
