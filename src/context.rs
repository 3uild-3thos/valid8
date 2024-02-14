use std::{collections::{HashMap, HashSet}, path::Path, io::{Read, Write}, fs::File};
use anchor_lang::accounts::program;
use anyhow::Result;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Serialize, Deserialize};
use solana_program::pubkey::Pubkey;

use crate::common::{
    helpers::create_resources_dir, 
    AccountSchema, 
    Network, 
    fetch_account,
    clone_program, clone_idl
};

/*

    Valid8Context is responsible for managing configuration, dependencies, overrides and atomically saving these changes to our config file.

    Upon startup it will:
    - Check for a valid valid8.json config and try to load it into memory with serde, or
    - If a config file doesn't exist, initialize a new one for us.
    - Check for a local .valid8 directory and ensure it is writable, or
    - Creates the .valid8 directory if it doesn't exist.
    - Loads all IDLs and programs for accounts/programs in our config file

*/

const CONFIG_PATH: &str = "./valid8.json";

#[derive(Serialize, Deserialize, Default)]
pub struct Valid8Context {
    pub networks: HashMap<String, Network>,
    pub programs: HashMap<String, AccountSchema>,
    pub accounts: HashMap<String, AccountSchema>,
    pub idls: HashSet<String>,
}

// impl Default for Valid8Context {
//     fn default() -> Self {
//         Self {
//             networks: HashMap::new(),
//             programs: HashMap::new(),
//             idls: HashSet::new(),
//             accounts: HashMap::new()
//         }
//     }
// }

impl Valid8Context {
    pub fn init() -> Result<Valid8Context>{
        create_resources_dir()?;
        Valid8Context::try_open()
    }

    pub fn install(&self) -> Result<()> {
        self.programs.values().collect::<Vec<&AccountSchema>>().into_par_iter().for_each(|p| {
            let _ = clone_program(&p);
            if self.idls.contains(&p.get_address().to_string()) {
                let _ = clone_idl(&p);
            }
        });
        Ok(())
    }

    pub fn try_save(&self) -> Result<()> {
        // let path = Path::new(CONFIG_PATH);
        // let mut f = File::create(path)?;
        let pretty_string = serde_json::to_string_pretty(&self)?;
        // f.write_all(pretty_string.as_bytes())?;
        File::create(Path::new(CONFIG_PATH))
            .and_then(|mut file|file.write_all(pretty_string.as_bytes()))?;
        Ok(())
    }

    pub fn try_open() -> Result<Self> {
        // let path = Path::new(CONFIG_PATH);

        if let Ok(mut file) = File::open(Path::new(CONFIG_PATH)) {
            let mut buf = vec![];
            file.read_to_end(&mut buf)?;
            let ctx: Valid8Context = serde_json::from_slice(&buf)?;
            Ok(ctx)
        } else {
            let ctx = Valid8Context::default();
            ctx.try_save()?;
            Ok(ctx)
        }
        // match File::open(path) {
        //     Ok(mut f) => {
        //         let mut buf = vec![];
        //         f.read_to_end(&mut buf)?;
        //         let ctx: Valid8Context = serde_json::from_slice(&buf)?;
        //         Ok(ctx)
        //     },
        //     Err(_) => {
        //         let ctx = Valid8Context::default();
        //         ctx.try_save()?;
        //         Ok(ctx)
        //     }
        // }
    }

    pub fn has_account(&self, pubkey: &Pubkey) -> bool {
        self.accounts.contains_key(&pubkey.to_string())
    }

    pub fn has_program(&self, program_id: &Pubkey) -> bool {
        self.programs.contains_key(&program_id.to_string())
    }

    pub fn add_program(&mut self, network: &Network, program_id: &Pubkey) -> Result<()> {
        // Check if we have the program in our hashmap already
        if self.has_program(&program_id) {
            println!("{} already added", &program_id.to_string());
            return Ok(())
        }
        self.add_program_unchecked(network, program_id)
    }

    pub fn add_program_unchecked(&mut self, network: &Network, program_id: &Pubkey) -> Result<()> {
        // Get program account
        let account = fetch_account(&network, &program_id)?;

        match program_id.to_string().as_ref() {
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" => {

            },
            address => {
                self.programs.insert(address.to_string(), account.clone());

                // Clone program data
                clone_program(&account)?;
            
                // Get IDL address
                if let Ok(_) = clone_idl(&account) {
                    self.add_idl(&program_id)?
                }
                // match clone_idl(&account) {
                //     Ok(_) => {
                //         self.add_idl(&program_id)?
                //     },
                //     Err(_) => ()
                // }
            }
        }
        // Save program account

        self.try_save()
    }

    pub fn add_account(&mut self, network: &Network, pubkey: &Pubkey) -> Result<()> {
        // Check if we have the program in our hashmap already
        if self.has_account(&pubkey) {
            println!("{} already added", &pubkey.to_string());
            return Ok(())
        }
        self.add_account_unchecked(network, pubkey)
    }

    pub fn add_account_unchecked(&mut self, network: &Network, pubkey: &Pubkey) -> Result<()> {
        // Get account
        let account = fetch_account(&network, &pubkey)?;

        // Save program account
        self.accounts.insert(pubkey.to_string(), account.clone());

        match self.has_program(&account.owner) {
            true => self.try_save(),
            false => self.add_program_unchecked(&network, &account.owner)
        }
    }

    pub fn add_idl(&mut self, program_id: &Pubkey) -> Result<()> {
        self.idls.insert(program_id.to_string());
        Ok(())
    }
}