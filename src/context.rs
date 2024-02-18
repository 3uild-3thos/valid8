use std::{collections::{HashMap, HashSet}, fs::{create_dir_all, File}, io::{Read, Write}, path::Path, str::FromStr};
use anyhow::Result;
use dialoguer::{Input, Select};
use serde::{Serialize, Deserialize};
use anyhow::anyhow;
use serde_json::Value;
use solana_ledger::{
    blockstore::create_new_ledger, 
    blockstore_options::LedgerColumnOptions,
};

use solana_runtime::genesis_utils::create_genesis_config_with_leader_ex;

use solana_sdk::{
    account::AccountSharedData, account_utils::StateMut, bpf_loader_upgradeable::UpgradeableLoaderState, epoch_schedule::EpochSchedule, fee_calculator::FeeRateGovernor, native_token::sol_to_lamports, program_pack::Pack, pubkey::Pubkey, rent::Rent, signature::{write_keypair_file, Keypair}, signer::Signer, system_program
};

use spl_token::state::Account as TokenAccount;

use crate::{common::{
        helpers, idl::{self, unpack_idl_account}, project_name::ProjectName, AccountSchema, Network
    }, config::ConfigJson, serialization::b58
};
//const MAX_GENESIS_ARCHIVE_UNPACKED_SIZE: u64 = 10 * 1024 * 1024; // 10 MiB from testvalidator source is not enough 

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
    pub overrides: Option<Vec<Override>>,
    pub idls: Vec<String>,
    pub compose: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
pub struct Override {
    #[serde(with = "b58")]
    pub pubkey: Pubkey,
    pub edit_fields: Vec<EditField>,
}

impl Override {
    pub fn new(pubkey: Pubkey, edit_field: EditField) -> Self {
        Self { pubkey, edit_fields: vec![edit_field] }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum EditField{
    #[serde(with = "b58")]
    Owner(Pubkey),
    #[serde(with = "b58")]
    UpgradeAuthority(Pubkey),
    Lamports(u64),
    Data(Value),
    UnpackTokenAccount,
    UnpackPDA,
}

impl From<ConfigJson> for Valid8Context {
    fn from(value: ConfigJson) -> Self {

        // Try to read accounts from disc, or return with default empty vector
        let programs = value.programs.iter()
            .map(|(pubkey, _)| helpers::read_account_from_disc(&value.project_name, pubkey))
            .collect::<Result<Vec<AccountSchema>>>()
            .unwrap_or_default();

        let accounts = value.accounts.iter()
            .map(|(pubkey, _)| helpers::read_account_from_disc(&value.project_name, pubkey))
            .collect::<Result<Vec<AccountSchema>>>()
            .unwrap_or_default();
        
        Self { 
            project_name: value.project_name,
            networks: value.networks,
            programs,
            accounts,
            idls: value.idls,
            overrides: value.overrides,
            compose: value.compose,
        }
    }
}

impl Valid8Context {

    pub fn init(name: Option<String>) -> Result<Valid8Context>{
        let mut project_name = ProjectName::default();
        if let Some(name) =  name {
            project_name = ProjectName::from_str(&name)?;
        }
        
        if let Ok((config, installed)) = Self::try_open_config(&project_name) {
            if !installed {
                let items = vec!["Install"];

                let selection = Select::new()
                    .with_prompt("Accounts not yet installed, select install, or press ESC to exit?")
                    .items(&items)
                    .interact_opt()?;

                if let Some(n) = selection {
                    match n {
                        0 => {Ok(
                            config.to_context()?
                        )},
                        _ => Err(anyhow!("Invalid option. Exit.")),
                    }
                } else {
                    Err(anyhow!("Accounts not installed. Exit"))
                }

            } else {
                println!("{} config found, accounts installed: {}", project_name.to_config(), installed);
                Ok(config.into())
            }
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

    pub fn try_init_config(project_name: &ProjectName) -> Result<Self> {
        let ctx = Valid8Context {
            project_name: project_name.clone(),
            .. Valid8Context::default()
        };
        let pretty_string = serde_json::to_string_pretty(&ConfigJson::from(ctx.clone()))?;

        // Create resources dir for this project
        Self::create_resources_dir(project_name)
            // Create config json for project
            .and_then(|_| Self::create_project_config(project_name))
            .map(|mut file| file.write_all(pretty_string.as_bytes()))??;

        Ok(ctx)
    }

    pub fn try_save_config(&self) -> Result<()> {

        let pretty_string = serde_json::to_string_pretty(&ConfigJson::from(self.clone()))?;
        File::create(Path::new(&self.project_name.to_config()))
            .and_then(|mut file|file.write_all(pretty_string.as_bytes()))?;
        Ok(())
    }

    pub fn try_open_config(project_name: &ProjectName) -> Result<(ConfigJson, bool)> {
        let mut buf = vec![];
        File::open(Path::new(&project_name.to_config()))
            .and_then(|mut file| file.read_to_end(&mut buf))?;
        let config: ConfigJson = serde_json::from_slice(&buf)?;
        println!("Config {:?}", &config);
    
        // Convert ConfigJson to Valid8Context, this also tries to read accounts from disc
        let mut installed = true;
        if !&config.is_installed() {
            println!("Accounts not found in local workspace, please run valid8 install to clone them.");
            installed = false;
        }

        Ok((config, installed))
    }

    pub fn try_compose(&self) -> Result<(u8,u32, u32)> {

        let mut this_ctx: ConfigJson = self.clone().into();
        let mut compose_count = 0;
        let mut account_count = 0;
        let mut program_count = 0;
        let mut new_config_path = self.compose.clone();

        while let Some(new_config) = new_config_path.clone() {
            compose_count += 1;

            if compose_count>20{return Err(anyhow!(compose_count))};
            
            let (new_ctx, _) = Valid8Context::try_open_config(&ProjectName::from_str(&new_config.replace(".json", ""))?)?;

            new_ctx.accounts.iter().for_each(|new_acc| {
                if !this_ctx.accounts.contains(new_acc) {this_ctx.accounts.push(new_acc.clone()); account_count+=1}
            });
    
            new_ctx.programs.iter().for_each(|new_prog| {
                if !this_ctx.programs.contains(new_prog) {this_ctx.programs.push(new_prog.clone()); program_count+=1}
            });
    
            new_ctx.idls.iter().for_each(|new_idl| {
                if !this_ctx.idls.contains(new_idl) {this_ctx.idls.push(new_idl.clone())}
            });
    
            new_ctx.networks.iter().for_each(|new_network| {
                this_ctx.networks.insert(new_network.clone());
            });
    
            if let Some(new_overrides) = new_ctx.overrides {
                new_overrides.iter().for_each(|new_over| {
                    if let Some(overrides) = this_ctx.overrides.as_mut(){
                        if !overrides.contains(new_over) {overrides.push(new_over.clone())}
                    }
                });
            }
            new_config_path = new_ctx.compose;
        }
        let new_context = this_ctx.to_context()?;
        new_context.try_save_config()?;

        Ok((compose_count, account_count, program_count))
    }

    pub fn has_account(&self, pubkey: &Pubkey) -> bool {
        self.accounts.iter().any(|acc| acc.pubkey == *pubkey) 
    }

    pub fn has_program(&self, program_id: &Pubkey) -> bool {
        self.programs.iter().any(|acc| acc.pubkey == *program_id)
    }

    pub fn add_program(&mut self, network: &Network, program_id: &Pubkey) -> Result<()> {
        // Check if we have the program in our hashmap already
        if self.has_program(program_id) {
            println!("{} already added", &program_id.to_string());
            return Ok(())
        }
        self.add_program_unchecked(network, program_id)
    }

    pub fn add_program_unchecked(&mut self, network: &Network, program_id: &Pubkey) -> Result<()> {
        // Get program account
        let program_account = helpers::fetch_account(network, program_id)?;

        match program_id.to_string().as_ref() {
            "BPFLoaderUpgradeab1e11111111111111111111111" => {  },
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" => {  },
            "11111111111111111111111111111111" => {  },
            _address => {
                self.programs.push(program_account.clone());

                // Clone program data
                let program_data_account = helpers::clone_program_data(self, &program_account)?;
                self.accounts.push(program_data_account);
            
                // Get IDL address
                if helpers::clone_idl(&program_account).is_ok() {
                    self.add_idl(program_id)?
                }
            }
        }
        // Save program account

        self.try_save_config()
    }

    pub fn add_account(&mut self, network: &Network, pubkey: &Pubkey) -> Result<()> {
        // Check if we have the account in our accounts
        if self.has_account(pubkey) {
            println!("{} already added", &pubkey.to_string());
            return Ok(())
        }
        self.add_account_unchecked(network, pubkey)
    }

    pub fn add_account_unchecked(&mut self, network: &Network, pubkey: &Pubkey) -> Result<()> {
        // Get account
        let account = helpers::fetch_account(network, pubkey)?;

        // Save program account
        self.accounts.push(account.clone());
        self.networks.insert(network.clone());

        match self.has_program(&account.owner) {
            true => self.try_save_config(),
            false => self.add_program_unchecked(network, &account.owner)
        }
        
    }

    pub fn add_idl(&mut self, program_id: &Pubkey) -> Result<()> {
        self.idls.push(program_id.to_string());
        Ok(())
    }

    pub fn get_account(&mut self, pubkey: &Pubkey,) -> Result<AccountSchema> {
        let position = self.accounts
            .iter()
            .position(|acc| acc.pubkey == *pubkey)
            .ok_or(anyhow!("No account found in context; Edit"))?;
        
        Ok(self.accounts.remove(position))
    }

    pub fn add_override(&mut self, over: Override) {
        if let Some(override_list) = self.overrides.as_mut() {
            if !override_list.contains(&over) {
                override_list.push(over)
            }
        } else {
            self.overrides = Some(vec![over])
        }
    }

    pub fn apply_overrides(&mut self) -> Result<()> {
        // iterate through all overrides from the config, and apply them on all accounts and programs
        if let Some (override_list) = self.overrides.clone() {
            override_list.iter().map(|over| {
                if self.accounts.iter().any(|acc| acc.pubkey == over.pubkey) {
                    over.edit_fields
                        .iter()
                        .map(|edit_field| self.edit_account(&over.pubkey, edit_field.clone()))
                        .collect::<Result<Vec<()>>>()
                        
                } else if self.programs.iter().any(|acc| acc.pubkey == over.pubkey) { 
                    over.edit_fields
                        .iter()
                        .map(|edit_field| self.edit_program(None, &over.pubkey, None, edit_field.clone()))
                        .collect::<Result<Vec<()>>>()
                } else {
                    Err(anyhow!("Account not found in context!: {}", over.pubkey))
                }
            }).collect::<Result<Vec<_>>>()?;
        }
        Ok(())
    }

    pub fn edit_account(&mut self, pubkey: &Pubkey, edit_field: EditField) -> Result<()> {
       
       // get the account from the context
        let mut account = self.get_account(pubkey)?;

        match edit_field {
            EditField::Lamports(new_lamports) => {
                account.lamports = new_lamports
            }
            EditField::Owner(new_owner) => {
                account.owner = new_owner
            },
            EditField::UnpackTokenAccount => {
                // deserialize token account data to Account struct for editing
                let mut token_account = TokenAccount::unpack(&account.data)?;

                let fields: Vec<String> = vec![
                    format!("Owner: {}", token_account.owner.to_string()),
                    format!("Token amount: {}", token_account.amount.to_string()),
                    format!("Token delegate: {:?}", token_account.delegate),
                    format!("Token delegate amount: {}", token_account.delegated_amount),
                ];

                let selection = Select::new().with_prompt("Select a field to edit").items(&fields).interact()?;

                match selection {
                    0 => {
                        let new_owner: Pubkey = Input::new().with_prompt("New owner pubkey:").interact_text()?;
                        token_account.owner = new_owner;
                    }
                    1 => {
                        let new_amount: u64 = Input::new().with_prompt("New amount:").interact_text()?;
                        token_account.amount = new_amount;
                    }
                    2 => {
                        let new_delegate: Pubkey = Input::new().with_prompt("New delegate pubkey:").interact_text()?;
                        token_account.delegate = Some(new_delegate).into();
                    }
                    3 => {
                        let new_delegate_amount: u64 = Input::new().with_prompt("New delegate amount:").interact_text()?;
                        token_account.delegated_amount = new_delegate_amount;
                    }
                    4 => {}
                    _ => return Err(anyhow!("Invalid token account edit option"))
                }

                let mut new_data = [0u8;TokenAccount::LEN];
                token_account.pack_into_slice(&mut new_data);
                account.data = new_data.to_vec();
            },
            _ => return Err(anyhow!("Invalid option")),
        }

        helpers::save_account_to_disc(&self.project_name, &account)?;
        self.accounts.push(account);
        self.add_override(Override::new(*pubkey, edit_field));
        self.try_save_config()?;
        
        Ok(())
    }

    pub fn edit_program(&mut self, program_account: Option<&Pubkey>, program_data_account: &Pubkey, program_pda: Option<&Pubkey>, edit_field: EditField) -> Result<()> {

        let mut program_data = self.get_account(program_data_account)?;

        let mut changed = true;

        match &edit_field {
            EditField::Lamports(new_lamports) => {
                // change the amount of lamports the account holds
                program_data.lamports = *new_lamports;
            }
            EditField::Owner(new_owner) => {
                // change the owner of the account
                program_data.owner = *new_owner
            },
            EditField::UpgradeAuthority(new_upgrade_auth) => {
                // serialize new accont data state and add it to the account data field, with new upgrade authority and slot 0
                let new_statue = UpgradeableLoaderState::ProgramData {
                    slot: 0,
                    upgrade_authority_address: Some(*new_upgrade_auth),
                };
                let mut acc = program_data.to_account()?;
                acc.set_state(&new_statue)?;
                program_data = AccountSchema::from_account(&acc, program_data_account, &program_data.network)?;
            },
            EditField::UnpackTokenAccount => { },
            EditField::UnpackPDA => {
                let idl = idl::open_idl(program_account.ok_or(anyhow!("No program key to edit pda"))?)?;
                let map = idl::generate_discriminator_map(&idl)?;

                // Get the account from context to edit
                let mut pda = self.get_account(program_pda.unwrap())?;
                println!("pda found {:?}", pda);
                
                // iterate on the idl map to find the right account discriminator
                let _ = map.iter().map(|(key,idl_type_def)| {
                    if key == &pda.data[..8] {
                        // create a mutable buffer for new account data, adding the discriminator as first part of the buffer
                        let mut new_acc_data = key.to_vec();

                        // unpack the pda account data to a vector of idl account fields
                        if let Ok(mut return_val) = unpack_idl_account(idl_type_def, pda.data[8..].to_vec()).map_err(|e|anyhow!(e))  {
                            
                            // create a vector from the deserialized values for the user to select from
                            let account_fields = return_val.iter().map(|field| {
                                format!("{}: {:?}",field.name, field.value)
                            }).collect::<Vec<String>>();

                                                        let selection = Select::new()
                                .with_prompt("Select PDA field to edit.")
                                .items(&account_fields)
                                .interact_opt()?;

                            let new_value: String = Input::new().with_prompt("New value").interact_text()?;
                
                            
                            if let Some(index_select) = selection {
                                let _ = return_val.iter_mut().enumerate().map(|(index,field)| {

                                    // find the user selected field in the vector, and edit it
                                    if index == index_select {
                                        field.edit(new_value.clone())?;
                                    }
                                    // serialize all idl account fields with borsh and add them to the new account data vector
                                    new_acc_data.extend_from_slice(&field.to_bytes()?);
                                    Ok(())

                                }).collect::<Result<Vec<()>>>()?;
                            }
                        }

                        // copy the newly created account data vector to the pda data field
                        pda.data[..new_acc_data.len()].copy_from_slice(&new_acc_data);
                        println!("changed data: {:?}", &pda);
                    }
                    Ok(())
                     
                }).collect::<Result<Vec<()>>>()?;
                helpers::save_account_to_disc(&self.project_name, &pda)?;
                self.accounts.push(pda);
                changed = false;
            },
            _ => return Err(anyhow!("Invalid option")),

        }
        // save edited account to disc for persistence
        helpers::save_account_to_disc(&self.project_name, &program_data)?;
        // add edited account back to context 
        self.accounts.push(program_data);
        if changed {
            // if an override happened, add it to the overrides list
            self.add_override(Override::new(*program_data_account, edit_field));
        }
        self.try_save_config()?;

        Ok(())
    }

    pub fn create_ledger(&self) -> Result<()> {

        // create a solana-test-validator compatible ledger directory with account and programs added
        let mint_address = Keypair::new();
        let validator_identity = Keypair::new();
        let validator_vote_account = Keypair::new();
        let validator_stake_account = Keypair::new();
        let faucet_keypair = Keypair::new();
        let validator_identity_lamports = sol_to_lamports(500.);
        let validator_stake_lamports = sol_to_lamports(1_000_000.);
        let mint_lamports = sol_to_lamports(500_000_000.);
        let faucet_lamports = sol_to_lamports(10_000_000.);


        let mut accounts: HashMap<Pubkey, AccountSharedData> = HashMap::new();

        for program in &self.programs {
            accounts.insert(program.pubkey, AccountSharedData::from(program.to_account()?));
        }
        
        for account in &self.accounts {
            accounts.insert(account.pubkey, AccountSharedData::from(account.to_account()?));
        }

        accounts.insert(
            faucet_keypair.pubkey(), 
            AccountSharedData::new(faucet_lamports, 0, &system_program::id())
        );

        let mut genesis_config = create_genesis_config_with_leader_ex(
            mint_lamports,
            &mint_address.pubkey(),
            &validator_identity.pubkey(),
            &validator_vote_account.pubkey(),
            &validator_stake_account.pubkey(),
            validator_stake_lamports,
            validator_identity_lamports,
            FeeRateGovernor::default(),
            Rent::default(),
            solana_sdk::genesis_config::ClusterType::Development,
            accounts.into_iter().collect(),
        );

        genesis_config.epoch_schedule = EpochSchedule::without_warmup();

        let test_ledger_path = Path::new("test-ledger");

        let _last_hash = create_new_ledger(
            test_ledger_path,
            &genesis_config,
            15485760,
            LedgerColumnOptions::default(),
        )
        .map_err(|err| {
            anyhow!(
                "Failed to create ledger at {}: {}",
                test_ledger_path.display(),
                err
            )
        })?;

        write_keypair_file(
            &validator_identity,
            test_ledger_path.join("validator-keypair.json").to_str().unwrap(),
        ).map_err(|e| anyhow!(e.to_string()))?;

        write_keypair_file(
            &validator_stake_account,
            test_ledger_path
                .join("stake-account-keypair.json")
                .to_str()
                .unwrap(),
        ).map_err(|e| anyhow!(e.to_string()))?;

        write_keypair_file(
            &validator_vote_account,
            test_ledger_path
                .join("vote-account-keypair.json")
                .to_str()
                .unwrap(),
        ).map_err(|e| anyhow!(e.to_string()))?;

        write_keypair_file(
            &faucet_keypair,
            test_ledger_path.join("faucet-keypair.json").to_str().unwrap(),
        ).map_err(|e| anyhow!(e.to_string()))?;
        println!("ledger directory created: test-ledger");

        Ok(())
    }
}