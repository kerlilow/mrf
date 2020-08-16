use std::cmp;
use std::error::Error;
use std::fmt;
use std::io::BufRead;
use std::process::Command;

use clap::{AppSettings, Clap};
use crossbeam_channel::{bounded, Receiver};
use crossbeam_utils::thread;
use dialoguer::Confirm;

use mrf::{command, parser::parse, replacer::Replacer};

const MAX_PREVIEWS: usize = 5;
const CHANNEL_CAP_MULTIPLIER: usize = 10;

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
    #[clap(short, long)]
    concurrency: Option<usize>,
    command: String,
    #[clap(required = true)]
    item: Vec<String>,
    replacer: String,
}

#[derive(Clone)]
struct ExecOutputOpts {
    output_left: bool,
    output_right: bool,
}

impl From<ExecOpts> for ExecOutputOpts {
    fn from(opts: ExecOpts) -> Self {
        ExecOutputOpts {
            output_left: opts.output_left,
            output_right: opts.output_right,
        }
    }
}

fn main() {
    std::process::exit(match run_app() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("An error occurred:\n{}", err);
            1
        }
    });
}

fn run_app() -> Result<(), Box<dyn Error>> {
    let opts = Opts::parse();
    match opts.subcmd {
        Subcommand::Map(sub_opts) => handle_map(sub_opts),
        Subcommand::Exec(sub_opts) => handle_exec(sub_opts),
    }
}

/// Handle map (`map`) subcommand.
fn handle_map(opts: MapOpts) -> Result<(), Box<dyn Error>> {
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

/// Handle exec subcommand.
fn handle_exec(opts: ExecOpts) -> Result<(), Box<dyn Error>> {
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
    let output_opts = ExecOutputOpts {
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
    opts: &ExecOutputOpts,
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

/// If items contain a single string "-", read items from stdin, otherwise return as-is.
fn items_from_opt(items: Vec<String>) -> Result<Vec<String>, std::io::Error> {
    Ok(if items.len() == 1 && items[0] == "-" {
        read_items_from_stdin()?
    } else {
        items
    })
}

/// Read items from stdin, one item per line.
fn read_items_from_stdin() -> Result<Vec<String>, std::io::Error> {
    let mut items: Vec<String> = vec![];
    for line in std::io::stdin().lock().lines() {
        items.push(line?);
    }
    Ok(items)
}

/// Resolve replacements by parsing elements from `replacement` and applying replacer to each item.
fn resolve_replacements<'a, T: AsRef<str> + cmp::PartialEq>(
    items: &'a [T],
    replacer_str: &str,
) -> Result<Vec<(&'a T, String)>, Box<dyn Error>> {
    let elems = parse(replacer_str)?;
    let replacer = Replacer::new(&elems);
    Ok(replace_items(&replacer, items))
}

/// Apply replacer to each item.
fn replace_items<'a, T: AsRef<str>>(replacer: &Replacer, items: &'a [T]) -> Vec<(&'a T, String)> {
    items
        .iter()
        .filter_map(|left| {
            replacer
                .replace(left.as_ref())
                .map(|right| (left, right))
                .ok()
        })
        .collect()
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
