use std::{collections::{HashMap, HashSet}, fs::{create_dir_all, File}, io::{Read, Write}, path::Path, str::FromStr};
use anchor_lang::accounts::program;
use anyhow::{anyhow, Result};
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use serde::{Serialize, Deserialize};
use solana_program::pubkey::Pubkey;

use crate::common::{
    helpers, project_name::ProjectName, AccountSchema, Network
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

#[derive(Serialize, Deserialize, Default)]
pub struct Valid8Context {
    pub project_name: ProjectName,
    pub networks: HashMap<String, Network>,
    pub programs: HashMap<String, AccountSchema>,
    pub accounts: HashMap<String, AccountSchema>,
    pub idls: HashSet<String>,
}

impl Valid8Context {

    pub fn init(name: Option<String>) -> Result<Valid8Context>{
        let mut project_name = ProjectName::default();
        if let Some(name) =  name {
            project_name = ProjectName::from_str(&name)?;
        }
        
        if let Ok(config) = Self::try_open_config(&project_name) {
            Ok(config)
        } else {
            Self::try_init_config(&project_name)
        }

    }

    pub fn create_resources_dir(project_name: &ProjectName) -> Result<()> {
        create_dir_all(Path::new(&project_name.to_resources()))?;
        Ok(())
    }

    pub fn create_project_config(project_name: &ProjectName) -> Result<File> {
        let file = File::create(Path::new(&project_name.to_config()))?;
        Ok(file)
    }

    pub fn install(&self) -> Result<()> {

        // Check if project already installed on local workspace



        let _ = self.programs.values().collect::<Vec<&AccountSchema>>().into_par_iter().map(|p| {
            helpers::clone_program(&self, &p)?;
            if self.idls.contains(&p.get_pubkey().to_string()) {
                helpers::clone_idl(&self, &p)?;
            }
            Ok(())
        }).collect::<Vec<Result<()>>>();
        Ok(())
    }

    pub fn try_init_config(project_name: &ProjectName) -> Result<Self> {
        let mut ctx = Valid8Context::default();
        ctx.project_name = project_name.clone();
        let pretty_string = serde_json::to_string_pretty(&ctx)?;

        // Create resources dir for this project
        Self::create_resources_dir(&project_name)
            // Create config for project
            .and_then(|_| Self::create_project_config(&project_name))
            // Write config to file
            .and_then(|mut file| Ok(file.write_all(pretty_string.as_bytes())))??;

        Ok(ctx)
    }

    pub fn try_save_config(&self) -> Result<()> {
        let pretty_string = serde_json::to_string_pretty(&self)?;
        File::create(Path::new(&self.project_name.to_config()))
            .and_then(|mut file|file.write_all(pretty_string.as_bytes()))?;
        Ok(())
    }

    pub fn try_open_config(project_name: &ProjectName) -> Result<Self> {
        let mut buf = vec![];
        File::open(Path::new(&project_name.to_config()))
            .and_then(|mut file| file.read_to_end(&mut buf))?;
        let ctx: Valid8Context = serde_json::from_slice(&buf)?;
        Ok(ctx)
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
        let account = helpers::fetch_account(&self, &network, &program_id)?;

        match program_id.to_string().as_ref() {
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" => {

            },
            address => {
                self.programs.insert(address.to_string(), account.clone());

                // Clone program data
                helpers::clone_program(&self, &account)?;
            
                // Get IDL address
                if let Ok(_) = helpers::clone_idl(&self, &account) {
                    self.add_idl(&program_id)?
                }
            }
        }
        // Save program account

        self.try_save_config()
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
        let mut account = helpers::fetch_account(&self, &network, &pubkey)?;
        // let account_bin = helpers::save_account(&self.project_name, pubkey, account.data)?;

        // Save program account
        self.accounts.insert(pubkey.to_string(), account.clone());

        match self.has_program(&account.owner) {
            true => self.try_save_config(),
            false => self.add_program_unchecked(&network, &account.owner)
        }
    }

    pub fn add_idl(&mut self, program_id: &Pubkey) -> Result<()> {
        self.idls.insert(program_id.to_string());
        Ok(())
    }
}