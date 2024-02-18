use anyhow::{anyhow, Result};
use dialoguer::{Input, Select};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

use crate::context::{EditField, Valid8Context};

pub fn edit(ctx: &mut Valid8Context) -> Result<()> {

    let mut address: Option<Pubkey> = None;
    while address.is_none() {
        let address_string: String = Input::new()
            .with_prompt("Account address to edit")
            .interact_text()?;

        match Pubkey::from_str(&address_string) {
            Ok(p) => address = Some(p),
            Err(_) => println!("Invalid address: {}. Please enter a valid base58-encoeded Solana address.", &address_string)
        }
    }

    let pubkey = address.ok_or(anyhow!("Public key not defined"))?;
    if ctx.has_account(&pubkey) {
        let account = ctx.accounts
            .iter()
            .find(|acc| acc.pubkey == pubkey)
            .ok_or(anyhow!("No account found in context"))?;
        // let account = ctx.accounts.get(position).ok_or(anyhow!("No account at that position"))?;

        let fields: Vec<String> = vec![
            format!("Owner: {}", account.owner.to_string()),
            format!("Lamports: {}", account.lamports.to_string()),
            format!("Unpack TokenAccount"),
            format!("Unpack PDA"),
        ];

        let selection = Select::new()
            .with_prompt("Select a field to edit")
            .items(&fields)
            .interact()?;

        match selection {
            0 => {
                let new_owner: Pubkey = Input::new().with_prompt("New owner pubkey").interact_text()?;
                ctx.edit_account(&pubkey, EditField::Owner(new_owner))?;
            },
            1 => {
                let new_lamports: u64 = Input::new().with_prompt("New lamports").interact_text()?;
                ctx.edit_account(&pubkey, EditField::Lamports(new_lamports))?;
            },
            2 => {
                ctx.edit_account(&pubkey, EditField::UnpackTokenAccount)?;
            },
            3 => {
                todo!();
                // ctx.edit_account(&pubkey, EditField::UnpackPDA(new_lamports))?;
            },
            _ => {}
        }

    } else {
        return Err(anyhow!("Account not found in context"));
    }

    Ok(())
}