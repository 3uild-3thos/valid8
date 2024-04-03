use anyhow::{anyhow, Result};
use dialoguer::{Input, Select};
use std::str::FromStr;
use crate::{
    common::network, 
    context::Valid8Context
};
use solana_sdk::{bpf_loader_upgradeable::{self, UpgradeableLoaderState}, pubkey::Pubkey};

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
                println!("Invalid address: {}. Please enter a valid base58-encoeded Solana address.", &program_id_string);
                continue
            }
        }
        let account = ctx.programs.iter().find_map(|account_schema| {
            if program_id.unwrap() == account_schema.pubkey {
                Some(account_schema)
            } else {
                None
            }
        });
        let mut fields: Vec<String> = vec!["owner".into(), "lamports".into(),];
        
        
        if let Some(account) = account {
            let program_executable_data_address = account.get_program_executable_data_address()?;
            // if let Ok(UpgradeableLoaderState::ProgramData {
            //     upgrade_authority_address,
            //     slot,
            // }) = program_data_account.state()
            // {
            
            let selection = Select::new()
                .with_prompt("Select a field to edit")
                .items(&fields)
                .interact()?;
            
            match selection {
                0 => todo!(),
                1 => todo!(),
                2 => todo!(),
                _ => todo!()
                // 3 => Err(Error::msg("Exit")),
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