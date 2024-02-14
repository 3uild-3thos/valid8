use anyhow::{anyhow, Result};
use dialoguer::Input;
use solana_program::pubkey::Pubkey;
use std::str::FromStr;
use crate::{
    common::network, 
    context::Valid8Context
};

pub fn command(ctx: &mut Valid8Context) -> Result<()> {
    let network = network::command()?;
    let mut program_id: Option<Pubkey> = None;
    while program_id.is_none() {
        let program_id_string: String = Input::new()
            .with_prompt("Program address")
            .interact_text()?;

        match Pubkey::from_str(&program_id_string) {
            Ok(p) => program_id = Some(p),
            Err(_) => println!("Invalid address: {}. Please enter a valid base58-encoeded Solana address.", &program_id_string)
        }
    }
    let pubkey = program_id.ok_or(anyhow!("Public key not defined"))?;// .expect("Public key not defined"); // This will never fail    
    ctx.add_program(&network, &pubkey)
}