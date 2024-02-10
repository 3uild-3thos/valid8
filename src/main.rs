use anyhow::Result;
use clap::{Parser, Subcommand};
use context::Valid8Context;

mod account;
mod commands;
mod context;
mod program;
mod common;
mod serialization;

const APP_NAME: &str = "Valid8";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Install,
    Run,
}

fn main() {
    let mut ctx = match Valid8Context::init() {
        Ok(c) => c,
        Err(e) => return eprintln!("{}", e)
    };

    match router(&mut ctx) {
        Ok(_) => (),
        Err(e) => eprintln!("{}", e)
    }
}

fn router(ctx: &mut Valid8Context) -> Result<()> {
    let cli = Cli::parse();

    if let Some(c) = &cli.command {
        match c {
            Commands::Init => commands::init::command(ctx)?,
            Commands::Install => commands::install::command(ctx)?,
            Commands::Run => todo!()
        }
    } else {
        commands::edit::command(ctx)?
    }

    Ok(())
}