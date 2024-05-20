use anyhow::{anyhow, Result};
use dialoguer::Input;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

use crate::{ common::network, context::Valid8Context };

pub fn clone(ctx: &mut Valid8Context) -> Result<()> {
    let network = network::get(ctx)?;
    
    let mut address: Option<Pubkey> = None;
    while address.is_none() {
        let address_string: String = Input::new()
            .with_prompt("Account address")
            .interact_text()?;

        match Pubkey::from_str(&address_string) {
            Ok(p) => address = Some(p),
            Err(_) => println!("Invalid address: {}. Please enter a valid base58-encoeded Solana address.", &address_string)
        }
    }

    let pubkey = address.ok_or(anyhow!("Public key not defined"))?;
    
    ctx.add_account(&network, &pubkey)
}