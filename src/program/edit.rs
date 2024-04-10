use crate::{
    common::{helpers, network},
    context::Valid8Context,
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
        let account = ctx.programs.iter().find_map(|account_schema| {
            if program_id.unwrap() == account_schema.pubkey {
                Some(account_schema)
            } else {
                None
            }
        });

        if let Some(program) = account {
            let program_executable_data_address = program.get_program_executable_data_address()?;
            let program_data_account = ctx
                .accounts
                .iter()
                .find(|account| account.pubkey == program_executable_data_address)
                .ok_or_else(|| anyhow!("Can't find program data account in context"))?;

            let upgrade_authority = if let Ok(UpgradeableLoaderState::ProgramData {
                upgrade_authority_address,
                slot,
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

            let mut selected_string = String::new();

            match selection {
                0 => {
                    let new_owner: String = Input::new()
                        .with_prompt("New owner pubkey")
                        .interact_text()?;
                }
                1 => todo!(),
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

                    // helpers::
                }
                _ => todo!(), // 3 => Err(Error::msg("Exit")),
                              // _ => if items.len() > selection {
                              //     Ok(Network::Custom(items[selection].clone()))
                              // } else {
                              //     Err(Error::msg("Invalid network selection"))
                              // }
            }
        }
    }
    Ok(())
}
