use anyhow::Result;
use dialoguer::Select;

use crate::{program, account, context::Valid8Context};

pub fn command(ctx: &mut Valid8Context) -> Result<()> {
    let items = vec![
        "Clone Program",
        "Edit Program", 
        "Clone Account", 
        "Edit Account"
    ];

    let selection = Select::new()
        .with_prompt("Select an option")
        .items(&items)
        .interact_opt()?;

    if let Some(n) = selection {
        match n {
            0 => program::clone::command(ctx)?,
            1 => todo!(), //program::edit::command()?,
            2 => account::clone::command(ctx)?,
            3 => todo!(), // account::edit::command()?,
            _ => todo!()
        }
    }

    Ok(())
}