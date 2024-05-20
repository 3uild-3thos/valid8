use anyhow::Result;
use clap::{Parser, Subcommand};
use context::Valid8Context;

mod account;
mod commands;
mod context;
mod program;
mod common;
mod serialization;
mod config;

// const APP_NAME: &str = "Valid8";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Clone, Subcommand)]
enum Commands {
    /// Run the interactive menu
    Run,
    /// Edit will promt you for an account or program pubkey to edit or clone
    Edit,
    /// Generate a custom ledger with accounts and programs added at genesis
    Ledger {overwrite_if_exists: Option<String>},
    /// Compose multiple valid8 configs
    Compose,
}


fn main() -> Result<()> {
    let cli = Cli::parse();
    let ctx = Valid8Context::init()?;

    router(&cli, ctx)

}

fn router(cli: &Cli, mut ctx: Valid8Context) -> Result<()> {

    if let Some(c) = &cli.command {
        match c {
            Commands::Run => commands::run(ctx)?,
            Commands::Edit => commands::edit(&mut ctx)?,
            Commands::Ledger{overwrite_if_exists} => commands::ledger(ctx, overwrite_if_exists)?,
            Commands::Compose => commands::compose(ctx)?,
        }
    } else {
        commands::run(ctx)?
    }

    Ok(())
}