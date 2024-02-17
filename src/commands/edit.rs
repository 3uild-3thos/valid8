use anyhow::Result;
use dialoguer::Select;

use crate::{program, account, context::Valid8Context};

pub fn edit(ctx: &mut Valid8Context) -> Result<()> {
    let items = vec![
        "Clone Program",
        "Edit Program", 
        "Clone Account", 
        "Edit Account"
    ];

    let selection = Select::new()
        .with_prompt("Select an option, or press Esc to exit.")
        .items(&items)
        .interact_opt()?;

    if let Some(n) = selection {
        match n {
            0 => program::clone(ctx)?,
            1 => todo!(), //program::edit::command()?,
            2 => account::clone(ctx)?,
            3 => todo!(), // account::edit::command()?,
            _ => {}
        }
    }

    Ok(())
}