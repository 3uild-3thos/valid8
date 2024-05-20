use anyhow::{anyhow, Result};
use dialoguer::{Input, Select};
use serde_json::Value;
use solana_sdk::{
    account_utils::StateMut,
    bpf_loader_upgradeable:: UpgradeableLoaderState,
    pubkey::Pubkey,
};
use std::str::FromStr;

use crate::context::{EditField, Valid8Context};

pub fn edit(ctx: &mut Valid8Context) -> Result<()> {
    let mut program_id = None;
    while program_id.is_none() {
        let program_id_string: String = Input::new()
            .with_prompt("Program address to edit")
            .interact_text()?;

        match Pubkey::from_str(&program_id_string) {
            Ok(p) => program_id = Some(p),
            Err(_) => {
                println!(
                    "Invalid address: {}. Please enter a valid base58-encoeded Solana address.",
                    &program_id_string
                );
                continue;
            }
        }
        
        let pubkey = program_id.ok_or(anyhow!("Public key not defined"))?;

        let program = ctx
            .programs
            .iter()
            .find(|acc| acc.pubkey == pubkey)
            .ok_or(anyhow!("No account found in context"))?;

        let program_executable_data_address = &program.get_program_executable_data_address()?;
        println!("program executable data address {}", program_executable_data_address);

        let program_data_account = ctx
            .accounts
            .iter()
            .find(|account| account.pubkey == *program_executable_data_address)
            .ok_or(anyhow!("No program data account in context"))?;

        let upgrade_authority = if let Ok(UpgradeableLoaderState::ProgramData {
            upgrade_authority_address,
            slot: _,
        }) = program_data_account.clone().to_account()?.state()
        {
            upgrade_authority_address
        } else {
            None
        };

        let fields: Vec<String> = vec![
            format!("owner: {}", program_data_account.owner.to_string()),
            format!("lamports: {}", program_data_account.lamports.to_string()),
            format!("data: {{Value}}"),
            {
                if let Some(pubkey) = upgrade_authority {
                    format!("upgrade authority: {}", pubkey)
                } else {
                    "upgrade authority: ".to_string()
                }
            },
            format!("Unpack PDA"),
        ];

        let selection = Select::new()
            .with_prompt("Select a field to edit")
            .items(&fields)
            .interact()?;


        match selection {
            0 => {
                let new_owner: Pubkey = Input::new().with_prompt("New owner pubkey").interact_text()?;
                ctx.edit_account(program_executable_data_address, EditField::Owner(new_owner))?;
            },
            1 => {
                let new_lamports: u64 = Input::new().with_prompt("New lamports").interact_text()?;
                ctx.edit_account(program_executable_data_address, EditField::Lamports(new_lamports))?;
            },
            2 => {
                let new_data: Value = Input::new().with_prompt("New data as Json Value").interact_text()?;
                ctx.edit_program(Some(&pubkey), program_executable_data_address, None, EditField::Data(new_data))?;
            },
            3 => {
                let new_upgrade_auth: Pubkey = Input::new().with_prompt("New upgrade authority pubkey").interact_text()?;
                ctx.edit_program(None, program_executable_data_address, None, EditField::UpgradeAuthority(new_upgrade_auth))?;
            },
            4 => {
                let pda_pubkey: Pubkey = Input::new().with_prompt("PDA pubkey").interact_text()?;
                ctx.edit_program(Some(&pubkey), program_executable_data_address, Some(&pda_pubkey), EditField::UnpackPDA)?;
            }
            _ => return Err(anyhow!("Wrong edit program option")), 
        }
    }

    Ok(())
}
