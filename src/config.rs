use anyhow::Result;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use solana_sdk::pubkey::Pubkey;
use std::{collections::HashSet, path::Path, str::FromStr};

use crate::{
    common::{helpers, AccountSchema, Network, ProjectName},
    context::Valid8Context,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ConfigJson {
    pub project_name: ProjectName,
    pub networks: HashSet<Network>,
    pub programs: Vec<(String, Network)>,
    pub accounts: Vec<(String, Network)>,
    pub idls: Vec<String>,
}

impl ConfigJson {
    pub fn to_context(&self) -> Result<Valid8Context> {
        let mut account_counter = 0;
        let mut new_context = Valid8Context {
            project_name: self.project_name.clone(),
            networks: self.networks.clone(),
            programs: vec![],
            accounts: vec![],
            idls: self.idls.clone(),
        };
        Valid8Context::create_resources_dir(&new_context.project_name)?;

        let accounts = self
            .accounts
            .clone()
            .into_par_iter()
            .map(|(pubkey_string, network)| {
                let account = helpers::fetch_account(&network, &Pubkey::from_str(&pubkey_string)?)?;
                helpers::save_account_to_disc(&self.project_name, &account)?;
                Ok(account)
            })
            .collect::<Result<Vec<AccountSchema>>>()?;
        account_counter = accounts.len();
        new_context.accounts = accounts;

        let programs = self
            .programs
            .clone()
            .into_par_iter()
            .map(|(pubkey_string, network)| {
                let program =
                    helpers::fetch_account(&network, &Pubkey::from_str(&pubkey_string)?)?;
                let program_data = helpers::clone_program_data(&new_context, &program)?;
                helpers::save_account_to_disc(&self.project_name, &program)?;
                helpers::save_account_to_disc(&self.project_name, &program_data)?;
                Ok((program, program_data))
            })
            .collect::<Result<Vec<(AccountSchema, AccountSchema)>>>()?;

        programs.into_iter().for_each(|(program, program_data)| {
            new_context.programs.push(program);
            new_context.accounts.push(program_data);
            account_counter+=2;
        });
        println!("Accounts installed: {}", account_counter);

        Ok(new_context)
    }

    pub fn is_installed(&self) -> bool {
        let mut ret = false;
        if Path::new(&self.project_name.to_resources()).exists() {
            if self
                .accounts
                .clone()
                .into_par_iter()
                .all(|(pubkey, _network)| {
                    helpers::read_account_from_disc(&self.project_name, &pubkey).is_ok()
                })
            {
                ret = true;
            }
        }
        ret
    }
}

impl From<Valid8Context> for ConfigJson {
    fn from(value: Valid8Context) -> Self {
        let programs: Vec<(String, Network)> = value
            .programs
            .iter()
            .map(|a_s| {
                let _ = helpers::save_account_to_disc(&value.project_name, &a_s);
                (a_s.pubkey.to_string(), a_s.network.clone())
            })
            .collect();

        let accounts: Vec<(String, Network)> = value
            .accounts
            .iter()
            .map(|a_s| {
                let _ = helpers::save_account_to_disc(&value.project_name, &a_s);
                (a_s.pubkey.to_string(), a_s.network.clone())
            })
            .collect();

        Self {
            project_name: value.project_name,
            networks: value.networks,
            programs,
            accounts,
            idls: value.idls,
        }
    }
}
