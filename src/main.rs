use std::error::Error;

use clap::{AppSettings, Clap};

use mrf::subcommands;

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
    Map(subcommands::map::Opts),
    Exec(subcommands::exec::Opts),
    Mv(subcommands::mv::Opts),
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
    ctrlc::set_handler(move || {
        let term = console::Term::stdout();
        let _ = term.show_cursor();
    })?;

    let opts = Opts::parse();
    match opts.subcmd {
        Subcommand::Map(sub_opts) => subcommands::map::run(sub_opts),
        Subcommand::Exec(sub_opts) => subcommands::exec::run(sub_opts),
        Subcommand::Mv(sub_opts) => subcommands::mv::run(sub_opts),
    }
}
