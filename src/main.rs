use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use common::ProjectName;
use context::Valid8Context;

mod account;
mod commands;
mod context;
mod program;
mod common;
mod serialization;

const APP_NAME: &str = "Valid8";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    // #[command(subcommand)]
    command: Option<Commands>,
    // ProjectName
    name: Option<String>
}

#[derive(Debug, Clone, ValueEnum)]
enum Commands {
    Install,
    Run,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut ctx = Valid8Context::init(cli.name.clone())?;

    router(&cli, &mut ctx)

}

fn router(cli: &Cli, ctx: &mut Valid8Context) -> Result<()> {

    if let Some(c) = &cli.command {
        match c {
            Commands::Install => commands::install(ctx)?,
            Commands::Run => todo!()
        }
    } else {
        commands::edit(ctx)?
    }

    Ok(())
}