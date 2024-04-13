use crate::{
    common::{helpers, network},
    context::{EditField, Valid8Context},
};
use anchor_lang::accounts::program;
use anyhow::{anyhow, Result};
use dialoguer::{Input, Select};
use solana_sdk::{
    account_utils::StateMut,
    bpf_loader_upgradeable::{self, UpgradeableLoaderState},
    pubkey::Pubkey,
};
use std::{fs::File, io::Read, path::Path, str::FromStr};

pub fn edit(ctx: &mut Valid8Context) -> Result<()> {
    // to change upgrade authority, we need to do the following steps

    // look for program to edit in programs of ctx
    // if found, look for program data account in ctx
    // if found, deserialize data to a mutable variable with UpgradeableLoaderState::ProgramData that returns upgrade auth and last deploy slot
    // serialize data and save account to ctx/disc

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

        let program = ctx.programs
            .iter()
            .find(|acc| acc.pubkey == pubkey)
            .ok_or(anyhow!("No account found in context"))?;

        let program_executable_data_address = program.get_program_executable_data_address()?;
        println!("program executable data address {}", program_executable_data_address);

        let program_data_account = ctx
            .accounts
            .iter()
            .find(|account| account.pubkey == program_executable_data_address)
            .ok_or(anyhow!("Can't find program data account in context"))?;

        let upgrade_authority = if let Ok(UpgradeableLoaderState::ProgramData {
            upgrade_authority_address,
            slot: _,
        }) = program_data_account.to_account()?.state()
        {
            upgrade_authority_address
        } else {
            None
        };

        let fields: Vec<String> = vec![
            format!("owner: {}", program_data_account.owner.to_string()),
            format!("lamports: {}", program_data_account.lamports.to_string()),
            {
                if let Some(pubkey) = upgrade_authority {
                    format!("upgrade authority: {}", pubkey)
                } else {
                    format!("upgrade authority: ")
                }
            },
        ];

        let selection = Select::new()
            .with_prompt("Select a field to edit")
            .items(&fields)
            .interact()?;


        match selection {
            0 => {
                let new_owner: String = Input::new().with_prompt("New owner pubkey").interact_text()?;
                ctx.edit_account(&pubkey, EditField::Owner(Pubkey::from_str(&new_owner)?))?;
            },
            1 => {
                let new_lamports: String = Input::new().with_prompt("New lamports").interact_text()?;
                ctx.edit_account(&pubkey, EditField::Lamports(new_lamports.parse()?))?;
            },
            2 => {
                let new_upgrade_auth: String = Input::new()
                    .with_prompt("New upgrade authority pubkey")
                    .interact_text()?;

                let mut program_data =
                    bincode::serialize(&UpgradeableLoaderState::ProgramData {
                        slot: 0,
                        upgrade_authority_address: Some(Pubkey::from_str(&new_upgrade_auth)?),
                    })?;

                let mut so_bytes = vec![];

                File::open(Path::new(&format!(
                    "{}{}.so",
                    ctx.project_name.to_resources(),
                    program.pubkey
                )))
                .and_then(|mut file| file.read_to_end(&mut so_bytes))?;

                program_data.extend_from_slice(&so_bytes);

                let edited_acc = ctx.programs.iter_mut().find_map(|account_schema| {
                    if account_schema.pubkey == program_executable_data_address {
                        account_schema.data = program_data.clone();
                        Some(account_schema)
                    } else {
                        None
                    }
                });
                if let Some(acc) = edited_acc {
                    helpers::save_account_to_disc(&ctx.project_name, acc)?;
                } else {
                    return Err(anyhow!("Couldn't edit and save program account"))
                }

                println!("Program edited");

            }
            _ => todo!(), 
        }
    }

    Ok(())
}
