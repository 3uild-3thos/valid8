use std::{fs, path::Path};
use anyhow::{anyhow, Result};
use dialoguer::Input;

use crate::context::Valid8Context;

pub fn ledger(ctx: Valid8Context, overwrite: &Option<String>) -> Result<()> {

    let ledger_path = Path::new("test-ledger");

    let mut user_choice = false;
    if ledger_path.exists() {
        if let Some(overwrite) = overwrite {
            if overwrite.to_ascii_lowercase() == *"-y" {
                user_choice = true;
            } 
        } else {
            let overwrite_choice: String = Input::new().with_prompt("Ledger path already exists, do you want to overwrite?(y/n)").interact_text()?;
            
            match overwrite_choice.to_ascii_lowercase().as_ref() {
                "y" => user_choice = true,
                "n" => return Err(anyhow!("No new ledger created")),
                _ => return Err(anyhow!("Incorrect option")),
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