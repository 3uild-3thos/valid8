use anchor_lang::prelude::borsh::schema::Fields;
use anyhow::{Error, Result};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use solana_program::pubkey::Pubkey;
use solana_sdk::account::Account;
use crate::serialization::{b58, b64};

use super::Network;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccountSchema {
    #[serde(with = "b58")]
    pubkey: Pubkey,
    network: Network,
    pub lamports: u64,
    #[serde(with = "b64")]
    pub data: Vec<u8>,
    #[serde(with = "b58")]
    pub owner: Pubkey,
    executable: bool,
    pub rent_epoch: u64
}

pub struct AccountField {
    type_of: String,
    length: u64,
    value: Value
}

impl From<AccountSchema> for Account {
    fn from(account_schema: AccountSchema) -> Self {
        Self {
            lamports: account_schema.lamports,
            data: account_schema.data,
            owner: account_schema.owner,
            executable: account_schema.executable,
            rent_epoch: account_schema.rent_epoch,
        }
    }
}

impl From<Account> for AccountSchema {
    fn from(account: Account) -> Self {
        Self {
            pubkey: Pubkey::default(),
            network: Network::default(),
            lamports: account.lamports,
            data: account.data,
            owner: account.owner,
            executable: account.executable,
            rent_epoch: account.rent_epoch,
        }
    }
}

impl AccountSchema {

    pub fn get_discriminator(&self) -> Result<[u8; 8]> {
        let mut d = [0u8;8];
        if self.data.len() < 8 {
            return Err(Error::msg("Account discriminator not found"));
        }
        d.copy_from_slice(&self.data[0..8]);
        Ok(d)
    }

    pub fn add_pubkey(&mut self, pubkey: &Pubkey) -> Result<()> {
        self.pubkey = pubkey.clone();
        Ok(())
    }

    pub fn add_network(&mut self, network: &Network) -> Result<()> {
        self.network = network.clone();
        Ok(())
    }

    // pub fn get_idl(&self) -> Result<(String, Vec<AccountField>)> {
        // let idl = load_idl
        // let name
    // }

    // pub fn to_json(&self) -> Value {
    //     let mut data_base64 = String::new();
    //     base64::engine::general_purpose::STANDARD.encode_string(&self.data, &mut data_base64);
    //     json!({
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
    //     )
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

    pub fn get_address(&self) -> Pubkey {
        self.pubkey.clone()
    }

    pub fn get_network(&self) -> Network {
        self.network.clone()
    }
}