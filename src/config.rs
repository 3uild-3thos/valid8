use anyhow::Result;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use solana_sdk::pubkey::Pubkey;
use std::{collections::HashSet, path::Path, str::FromStr};

use crate::{
    common::{helpers, project_name::ProjectName, AccountSchema, Network},
    context::{Override, Valid8Context},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ConfigJson {
    pub project_name: ProjectName,
    pub networks: HashSet<Network>,
    pub programs: Vec<(String, Network)>,
    pub accounts: Vec<(String, Network)>,
    pub overrides: Option<Vec<Override>>,
    pub idls: Vec<String>,
    pub compose: Option<String>
}


#[allow(unused_assignments)]
impl ConfigJson {
    pub fn to_context(&self) -> Result<Valid8Context> {
        let mut account_counter = 0;
        let mut new_context = Valid8Context {
            project_name: self.project_name.clone(),
            networks: self.networks.clone(),
            programs: vec![],
            accounts: vec![],
            overrides: self.overrides.clone(),
            idls: self.idls.clone(),
            compose: self.compose.clone(),
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
                let program_account =
                    helpers::fetch_account(&network, &Pubkey::from_str(&pubkey_string)?)?;
                let program_data = helpers::clone_program_data(&new_context, &program_account)?;
                helpers::save_account_to_disc(&self.project_name, &program_account)?;
                helpers::save_account_to_disc(&self.project_name, &program_data)?;
                let program_idl = helpers::clone_idl(&program_account);
                Ok((program_account, program_data, program_idl))
            })
            .collect::<Result<Vec<(AccountSchema, AccountSchema, Result<_>)>>>()?;

        programs.into_iter().for_each(|(program, program_data, program_idl)| {
            new_context.programs.push(program.clone());
            new_context.accounts.push(program_data);
            if program_idl.is_ok() {
                new_context.idls.push(program.pubkey.to_string())
            }
            account_counter+=2;
        });
        new_context.apply_overrides()?;

        Ok(new_context)
    }

    pub fn is_installed(&self) -> bool {
        let mut ret = false;
        if Path::new(&self.project_name.to_resources()).exists() && self
                .accounts
                .clone()
                .into_par_iter()
                .all(|(pubkey, _network)| {
                    helpers::read_account_from_disc(&self.project_name, &pubkey).is_ok()
                }) {
            ret = true;
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
                let _ = helpers::save_account_to_disc(&value.project_name, a_s);
                (a_s.pubkey.to_string(), a_s.network.clone())
            })
            .collect();

        let accounts: Vec<(String, Network)> = value
            .accounts
            .iter()
            .map(|a_s| {
                let _ = helpers::save_account_to_disc(&value.project_name, a_s);
                (a_s.pubkey.to_string(), a_s.network.clone())
            })
            .collect();

        Self {
            project_name: value.project_name,
            networks: value.networks,
            programs,
            accounts,
            overrides: value.overrides,
            idls: value.idls,
            compose: value.compose,
        }
    }
}
