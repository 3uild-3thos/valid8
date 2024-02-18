use anyhow::Result;
use dialoguer::Select;
use anyhow::anyhow;

use crate::{program, account, context::Valid8Context, commands};


pub fn run(ctx: &mut Valid8Context) -> Result<()> {
    let items = vec![
        "Clone Program",
        "Clone Account",
        "Edit Program", 
        "Edit Account",
        "Compose Configs",
        "Generate Custom Ledger"
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
            4 => commands::compose(ctx)?,
            5 => commands::ledger(ctx, &None)?,
            _ => return Err(anyhow!("Invalid option."))
        }
    }

    Ok(())
}