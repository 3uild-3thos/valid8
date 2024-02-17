use std::{fs::File, path::Path, io::{Write, Read}};
use anyhow::{Result, Error};
use flate2::read::ZlibDecoder;
// use serde_json::Value;
use solana_program::pubkey::Pubkey;
use anchor_lang::{idl::IdlAccount, AnchorDeserialize};
use super::{Network, AccountSchema};

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
    let mut account: AccountSchema = client.get_account(&pubkey)?.into();
    account.add_network(network)
        .and_then(|_| account.add_pubkey(pubkey))?;

    Ok(account)
}

pub fn fetch_account_data(network: &Network, pubkey: &Pubkey) -> Result<Vec<u8>> {
    let client = network.client();
    Ok(client.get_account_data(&pubkey)?)
}

pub fn clone_program(account: &AccountSchema) -> Result<()> {
    // Get program executable data address
    let program_executable_data_address = get_program_executable_data_address(&account)?;

    // Fetch program executable data
    let program_executable_data = fetch_account_data(&account.get_network(), &program_executable_data_address)?;

    // Save program executable data
    save_program(&account.get_address(), &program_executable_data)
}

pub fn clone_idl(account: &AccountSchema) -> Result<()> {
    // Get program address
    let program_id = account.get_address();
    // Get IDL address
    let idl_address = find_idl_address(&account.get_address())?;

    // Get IDL data
    match fetch_idl_schema(&account.get_network(), &idl_address) {
        Ok(d) => {
            save_idl(&program_id, &d)
        },
        Err(e) => {
            Err(Error::msg(e.to_string()))
        }
    }
}

pub fn get_program_executable_data_address(account: &AccountSchema) -> Result<Pubkey> {
    let mut executable_data_bytes = [0u8;32];
    executable_data_bytes.copy_from_slice(&account.data[4..36]);
    Ok(Pubkey::new_from_array(executable_data_bytes))
}

pub fn save_idl(pubkey: &Pubkey, data: &Vec<u8>) -> Result<()> {
    File::create(Path::new(&format!("./.valid8/{}.idl.json", pubkey)))
        .and_then(|mut file| file.write_all(&data))?;
    Ok(())
}

pub fn save_program(pubkey: &Pubkey, data: &Vec<u8>) -> Result<()> {
    File::create(Path::new(&format!("./.valid8/{}.so", pubkey)))
        .and_then(|mut file| file.write_all(&data))?;
    Ok(())
}