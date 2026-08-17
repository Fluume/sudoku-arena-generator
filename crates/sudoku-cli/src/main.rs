mod cli;
mod commands;
mod output;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Command};

fn main() -> Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Command::Generate(args) => commands::generate::run(args),
        Command::Rate(args) => commands::rate::run(args),
    }
}
