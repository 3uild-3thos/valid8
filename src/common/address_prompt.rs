use std::str::FromStr;
use anyhow::Result;
use dialoguer::Input;
use solana_sdk::pubkey::Pubkey;

pub fn prompt_address() -> Result<Pubkey> {
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
    Ok(program_id.unwrap())
}