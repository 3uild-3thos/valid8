use std::{fs, path::Path};

use anyhow::{anyhow, Result};
use dialoguer::Select;

use crate::{program, account, context::Valid8Context};

pub fn ledger(ctx: &mut Valid8Context, overwrite: bool) -> Result<()> {

    println!("overwrite {}", overwrite);
    let ledger_path = Path::new("test-ledger");

    let mut user_choice = false;
    if ledger_path.exists() {
        if overwrite {
            user_choice = true;
        } else {
            let items = vec![
                "Yes",
                "Exit", 
            ];
            let selection = Select::new()
                .with_prompt("Ledger path already exists, do you want to overwrite?")
                .items(&items)
                .interact_opt()?;

            if let Some(n) = selection {
                match n {
                    0 => user_choice = true,
                    1 => return Err(anyhow!("No new ledger created")),
                    _ => {}
                }
            }
        }
    }

    if user_choice {
        println!("Overwiting test-ledger directory");
        fs::remove_dir_all(ledger_path)?;

    }
    

    ctx.create_ledger()?;
    

    Ok(())
}