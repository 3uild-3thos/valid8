use anyhow::Result;
use clap::{ArgAction, Parser, Subcommand, ValueEnum};
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
    // #[command(subcommand)]  // ./valid8 json1.json json2.json 
    command: Option<Commands>,
    // commmand argument
    #[arg(long, short, action=ArgAction::SetTrue)]
    yes: bool,
}

#[derive(Debug, Clone, ValueEnum)]
enum Commands {
    Install,
    Edit,
    Run,
    Ledger,
    Compose,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut ctx = Valid8Context::init(None)?;

    router(&cli, &mut ctx)

}

fn router(cli: &Cli, ctx: &mut Valid8Context) -> Result<()> {

    if let Some(c) = &cli.command {
        match c {
            Commands::Install => commands::install(ctx)?,
            Commands::Run => todo!(),
            Commands::Edit => commands::edit(ctx)?,
            Commands::Ledger => commands::ledger(ctx, cli.yes.clone())?,
            Commands::Compose => todo!(),
        }
    } else {
        commands::edit(ctx)?
    }

    Ok(())
}