use anyhow::Result;
use serde::{Serialize, Deserialize};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::account::Account;
// use crate::{context::Valid8Context, serialization::{b58, b64}};

use super::Network;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccountSchema {
    pub pubkey: Pubkey,
    pub network: Network,
    pub lamports: u64,
    pub data: Vec<u8>,
    pub owner: Pubkey,
    pub executable: bool,
    pub rent_epoch: u64
}

// pub struct AccountField {
//     type_of: String,
//     length: u64,
//     value: Value
// }

impl AccountSchema {

    pub fn from_account(account: &Account, pubkey: &Pubkey, network: &Network) -> Result<Self> {
        Ok(Self {
            pubkey: pubkey.clone(),
            network: network.clone(),
            lamports: account.lamports,
            data: account.data.clone(),
            owner: account.owner,
            executable: account.executable,
            rent_epoch: account.rent_epoch,
        })
    }

    pub fn to_account(&self) -> Result<Account> {
        Ok(Account {
            lamports: self.lamports,
            data: self.data.clone(),
            owner: self.owner,
            executable: self.executable,
            rent_epoch: self.rent_epoch,
        })
    }

    pub fn get_program_executable_data_address(&self) -> Result<Pubkey> {
        let mut executable_data_bytes = [0u8;32];
        executable_data_bytes.copy_from_slice(&self.data[4..36]);
        Ok(Pubkey::new_from_array(executable_data_bytes))
    }

    // pub fn get_discriminator(&self) -> Result<[u8; 8]> {
    //     let mut d = [0u8;8];
    //     if self.data.len() < 8 {
    //         return Err(Error::msg("Account discriminator not found"));
    //     }
    //     d.copy_from_slice(&self.data[0..8]);
    //     Ok(d)
    // }

    // pub fn get_idl(&self) -> Result<(String, Vec<AccountField>)> {
        // let idl = load_idl
        // let name
    // }

    // pub fn export(&self) -> Result<Vec<u8>> {
    //     let mut data_base64 = String::new();
    //     base64::engine::general_purpose::STANDARD.encode_string(&self.data, &mut data_base64);
    //     Ok(json!({
    //         "account": {
    //             "data":[data_base64,"base64"],
    //                 "executable":self.executable,
    //                 "lamports":self.lamports,
    //                 "owner": self.owner.to_string(),
    //                 "rentEpoch": self.rent_epoch,
    //                 "space": self.data.len()
    //             },
    //             "pubkey": self.pubkey.to_string(),
    //             "network": self.network.to_string()
    //         }
    //     ).to_string().as_bytes().to_vec())
    // }
    // pub fn get_data(&self) -> Result<Vec<u8>> {
    //     let mut buf = vec![];
    //     File::open(Path::new(&self.data_path))
    //         .and_then(|mut file | file.read_to_end(&mut buf))?;
    //     Ok(buf)
    // }

    pub fn get_pubkey(&self) -> Pubkey {
        self.pubkey.clone()
    }

    pub fn get_network(&self) -> Network {
        self.network.clone()
    }
}