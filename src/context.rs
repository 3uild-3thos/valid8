use std::{collections::HashSet, fs::{create_dir_all, File}, io::{Read, Write}, path::Path, str::FromStr};
use anchor_lang::accounts::program;
use anyhow::Result;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
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

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Valid8Context {
    pub project_name: ProjectName,
    pub networks: HashSet<Network>,
    pub programs: Vec<AccountSchema>,
    pub accounts: Vec<AccountSchema>,
    pub idls: Vec<String>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ConfigJson {
    pub project_name: ProjectName,
    pub networks: HashSet<Network>,
    pub programs: Vec<(String, Network)>,
    pub accounts: Vec<(String, Network)>,
    pub idls: Vec<String>,
}

impl From<Valid8Context> for ConfigJson {
    fn from(value: Valid8Context) -> Self {

        let programs: Vec<(String, Network)> = value.programs.iter().map(|a_s| {
            let _ = helpers::save_account_to_disc(&value.project_name, &a_s);
            (a_s.pubkey.to_string(), a_s.network.clone())
        }).collect();

        let accounts: Vec<(String, Network)> = value.accounts.iter().map(|a_s| {
            let _ = helpers::save_account_to_disc(&value.project_name, &a_s);
            (a_s.pubkey.to_string(), a_s.network.clone())
        }).collect();

        Self {
            project_name: value.project_name,
            networks: value.networks,
            programs,
            accounts,
            idls: value.idls,
        }
    }
}

impl From<ConfigJson> for Valid8Context {
    fn from(value: ConfigJson) -> Self {

        // Try to read accounts from disc, or return with default empty vector
        let programs = value.programs.iter().map(|(pubkey, _)| helpers::read_account_from_disc(&value.project_name, pubkey)).collect::<Result<Vec<AccountSchema>>>().unwrap_or_default();
        let accounts = value.accounts.iter().map(|(pubkey, _)| helpers::read_account_from_disc(&value.project_name, pubkey)).collect::<Result<Vec<AccountSchema>>>().unwrap_or_default();
        
        Self { 
            project_name: value.project_name,
            networks: value.networks,
            programs: programs,
            accounts: accounts,
            idls: value.idls,
        }
    }
}

impl Valid8Context {

    pub fn init(name: Option<String>) -> Result<Valid8Context>{
        let mut project_name = ProjectName::default();
        if let Some(name) =  name {
            project_name = ProjectName::from_str(&name)?;
        }
        
        if let Ok(config) = Self::try_open_config(&project_name) {
            println!("{} config found", project_name.to_config());
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



        let _ = self.programs.iter().collect::<Vec<&AccountSchema>>().into_par_iter().map(|p| {
            helpers::clone_program(&self, &p)?;
            if self.idls.contains(&p.pubkey.to_string()) {
                helpers::clone_idl(&self, &p)?;
            }
            Ok(())
        }).collect::<Vec<Result<()>>>();
        Ok(())
    }

    pub fn try_init_config(project_name: &ProjectName) -> Result<Self> {
        let mut ctx = Valid8Context::default();
        ctx.project_name = project_name.clone();
        let pretty_string = serde_json::to_string_pretty(&ConfigJson::from(ctx.clone()))?;

        // Create resources dir for this project
        Self::create_resources_dir(&project_name)
            // Create config json for project
            .and_then(|_| Self::create_project_config(&project_name))
            // Write config to file
            .and_then(|mut file| Ok(file.write_all(pretty_string.as_bytes())))??;

        Ok(ctx)
    }

    pub fn try_save_config(&self) -> Result<()> {

        let pretty_string = serde_json::to_string_pretty(&ConfigJson::from(self.clone()))?;
        File::create(Path::new(&self.project_name.to_config()))
            .and_then(|mut file|file.write_all(pretty_string.as_bytes()))?;
        Ok(())
    }

    pub fn try_open_config(project_name: &ProjectName) -> Result<Self> {
        let mut buf = vec![];
        File::open(Path::new(&project_name.to_config()))
            .and_then(|mut file| file.read_to_end(&mut buf))?;
        let config: ConfigJson = serde_json::from_slice(&buf)?;
        println!("try open config {:?}", &config);

        // Convert ConfigJson to Valid8Context, this also tries to read accounts from disc
        Ok(config.into())
    }

    pub fn has_account(&self, pubkey: &Pubkey) -> bool {
        self.accounts.iter().find(|acc| acc.pubkey == *pubkey).is_some() 
    }

    pub fn has_program(&self, program_id: &Pubkey) -> bool {
        self.programs.iter().find(|acc| acc.pubkey == *program_id).is_some()
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
        let account = helpers::fetch_account(&network, &program_id)?;

        match program_id.to_string().as_ref() {
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" => {  },
            "11111111111111111111111111111111" => {  },
            _address => {
                self.programs.push(account.clone());

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
        let account = helpers::fetch_account(&network, &pubkey)?;

        // Save program account
        self.accounts.push(account.clone());
        self.networks.insert(network.clone());

        match self.has_program(&account.owner) {
            true => self.try_save_config(),
            false => self.add_program_unchecked(&network, &account.owner)
        }
    }

    pub fn add_idl(&mut self, program_id: &Pubkey) -> Result<()> {
        self.idls.push(program_id.to_string());
        Ok(())
    }

}