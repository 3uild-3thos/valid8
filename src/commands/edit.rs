use anyhow::Result;
use dialoguer::Select;

use crate::{program, account, context::Valid8Context};

pub fn edit(ctx: &mut Valid8Context) -> Result<()> {
    let items = vec![
        "Clone program",
        "Clone account",
        "Edit Program", 
        "Edit Account"
    ];

    let selection = Select::new()
        .with_prompt("Select an option, or press Esc to exit.")
        .items(&items)
        .interact_opt()?;

    if let Some(n) = selection {
        match n {
            0 => program::clone(ctx)?,
            1 => account::clone(ctx)?,
            2 => program::edit(ctx)?,
            3 => account::edit(ctx)?,
            _ => {}
        }
    }

    Ok(())
}