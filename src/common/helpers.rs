use std::{fs::File, io::{Read, Write}, path::Path, str::FromStr};
use anyhow::{Error, Result};
use flate2::read::ZlibDecoder;
use anchor_lang::{idl::IdlAccount, AnchorDeserialize};
use solana_sdk::pubkey::Pubkey;
use crate::context::Valid8Context;

use super::{AccountSchema, Network, project_name::ProjectName};

pub fn find_idl_address(pubkey: &Pubkey) -> Result<Pubkey> {
    Ok(IdlAccount::address(pubkey))
}

pub fn fetch_idl_schema(network: &Network, pubkey: &Pubkey) -> Result<Vec<u8>> {
    let data = fetch_account_data(network, pubkey)?;
    // Cut off account discriminator.
    let mut d: &[u8] = &data[8..];
    let idl_account: IdlAccount = AnchorDeserialize::deserialize(&mut d)?;

    let compressed_len: usize = idl_account.data_len.try_into()?;
    let compressed_bytes = &data[44..44 + compressed_len];
    let mut z = ZlibDecoder::new(compressed_bytes);
    let mut s = Vec::new();
    z.read_to_end(&mut s)?;
    Ok(s.to_vec())
}

pub fn fetch_account(network: &Network, pubkey: &Pubkey) -> Result<AccountSchema> {
    let client = network.client();
    let account_scema = AccountSchema::from_account( &client.get_account(pubkey)?, pubkey, network)?;
    Ok(account_scema)
}

pub fn fetch_account_data(network: &Network, pubkey: &Pubkey) -> Result<Vec<u8>> {
    let client = network.client();
    Ok(client.get_account_data(pubkey)?)
}

pub fn clone_program_data(ctx: &Valid8Context, account: &AccountSchema) -> Result<AccountSchema> {
    // Get program executable data address
    let program_executable_data_address = account.get_program_executable_data_address()?;
    let program_executable_data_account = fetch_account(&account.network, &program_executable_data_address)?;
    // ctx.add_account(network, &program_executable_data_address)?;

    // Fetch program executable data
    let program_executable_data = fetch_account_data(&account.get_network(), &program_executable_data_address)?;

    // Save program executable data
    save_program(&ctx.project_name, &account.get_pubkey(), &program_executable_data)?;
    Ok(program_executable_data_account)
}

pub fn clone_idl(program_account: &AccountSchema) -> Result<()> {
    // Get IDL address
    let idl_address = find_idl_address(&program_account.pubkey)?;

    // Get IDL data
    match fetch_idl_schema(&program_account.network, &idl_address) {
        Ok(d) => {
            save_idl(&ProjectName::default(), &program_account.pubkey, &d)
        },
        Err(e) => {
            Err(Error::msg(e.to_string()))
        }
    }
}

pub fn save_account_to_disc(project_name: &ProjectName, account_schema: &AccountSchema) -> Result<String> {
    let account_bytes = bincode::serialize(account_schema)?;
    File::create(Path::new(&format!("{}{}.bin", project_name.to_resources(), account_schema.pubkey)))
        .and_then(|mut file| file.write_all(&account_bytes))?;
    Ok(format!("{}{}", project_name.to_resources(), account_schema.pubkey))
}

pub fn read_account_from_disc(project_name: &ProjectName, pubkey_str: &str) -> Result<AccountSchema> {
    let pubkey = Pubkey::from_str(pubkey_str)?;
    let mut account_bytes = vec![];
    File::open(Path::new(&format!("{}{}.bin", project_name.to_resources(), pubkey)))
        .and_then(|mut file| file.read_to_end(&mut account_bytes))?;
    let account = bincode::deserialize(&account_bytes)?;
    Ok(account)
}

pub fn save_idl(project_name: &ProjectName, pubkey: &Pubkey, data: &[u8]) -> Result<()> {
    File::create(Path::new(&format!("{}{}.idl.json", project_name.to_resources(), pubkey)))
        .and_then(|mut file| file.write_all(data))?;
    Ok(())
}

pub fn save_program(project_name: &ProjectName, pubkey: &Pubkey, data: &[u8]) -> Result<()> {
    File::create(Path::new(&format!("{}{}.so", project_name.to_resources(), pubkey)))
        .and_then(|mut file| file.write_all(data))?;
    Ok(())
}