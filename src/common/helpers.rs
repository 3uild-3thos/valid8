use std::{fs::File, path::Path, io::{Write, Read}};
use anyhow::{Result, Error};
use flate2::read::ZlibDecoder;
use serde::Serialize;
// use serde_json::Value;
use anchor_lang::{idl::IdlAccount, AnchorDeserialize};
use solana_sdk::{
    pubkey::Pubkey,
    account::Account, 
};
use borsh::{self, BorshSerialize};
use crate::context::Valid8Context;

use super::{AccountSchema, Network, ProjectName};

pub fn find_idl_address(pubkey: &Pubkey) -> Result<Pubkey> {
    Ok(IdlAccount::address(pubkey))
}

pub fn fetch_idl_schema(ctx: &Valid8Context, network: &Network, pubkey: &Pubkey) -> Result<Vec<u8>> {
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

pub fn fetch_account(ctx: &Valid8Context, network: &Network, pubkey: &Pubkey) -> Result<AccountSchema> {
    let client = network.client();
    let mut account = AccountSchema::from_account(&ctx, client.get_account(&pubkey)?, pubkey)?;
    account.add_network(network)?;
    account.add_pubkey(pubkey)?;

    Ok(account)
}

pub fn fetch_account_data(network: &Network, pubkey: &Pubkey) -> Result<Vec<u8>> {
    let client = network.client();
    Ok(client.get_account_data(&pubkey)?)
}

pub fn clone_program(ctx: &Valid8Context, account: &AccountSchema) -> Result<()> {
    println!("clone_program");
    // Get program executable data address
    let program_executable_data_address = get_program_executable_data_address(&account)?;

    // Fetch program executable data
    let program_executable_data = fetch_account_data(&account.get_network(), &program_executable_data_address)?;

    // Save program executable data
    save_program(&ctx.project_name, &account.get_pubkey(), &program_executable_data)
}

pub fn clone_idl(ctx: &Valid8Context, account: &AccountSchema) -> Result<()> {
    println!("clone_idl");
    // Get program address
    let program_id = account.get_pubkey();
    // Get IDL address
    let idl_address = find_idl_address(&account.get_pubkey())?;

    // Get IDL data
    match fetch_idl_schema(ctx, &account.get_network(), &idl_address) {
        Ok(d) => {
            save_idl(&ctx.project_name, &program_id, &d)
        },
        Err(e) => {
            Err(Error::msg(e.to_string()))
        }
    }
}

pub fn get_program_executable_data_address(account: &AccountSchema) -> Result<Pubkey> {
    println!("get_program_executable_data");
    let account_data = account.get_data()?;
    let mut executable_data_bytes = [0u8;32];
    executable_data_bytes.copy_from_slice(&account_data[4..36]);
    Ok(Pubkey::new_from_array(executable_data_bytes))
}

pub fn save_account(project_name: &ProjectName, pubkey: &Pubkey, data: &Vec<u8>) -> Result<String> {
    println!("save_account");
    File::create(Path::new(&format!("{}{}", project_name.to_resources(), pubkey)))
        .and_then(|mut file| file.write_all(data))?;
    Ok(format!("{}{}", project_name.to_resources(), pubkey))
}

pub fn read_account(project_name: &ProjectName, pubkey: &Pubkey) -> Result<Vec<u8>> {
    println!("read_account");
    let mut data = vec![];
    File::open(Path::new(&format!("{}{}", project_name.to_resources(), pubkey)))
        .and_then(|mut file| file.read_to_end(&mut data))?;
    Ok(data)
}

pub fn save_idl(project_name: &ProjectName, pubkey: &Pubkey, data: &Vec<u8>) -> Result<()> {
    File::create(Path::new(&format!("{}{}.idl.json", project_name.to_resources(), pubkey)))
        .and_then(|mut file| file.write_all(&data))?;
    Ok(())
}

pub fn save_program(project_name: &ProjectName, pubkey: &Pubkey, data: &Vec<u8>) -> Result<()> {
    println!("save_program");
    File::create(Path::new(&format!("{}{}.so", project_name.to_resources(), pubkey)))
        .and_then(|mut file| file.write_all(&data))?;
    Ok(())
}