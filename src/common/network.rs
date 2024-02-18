use std::{fmt::Display, str::FromStr};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use anyhow::{Result, Error};
use dialoguer::{Input, Select};
use solana_client::rpc_client::RpcClient;

use crate::context::Valid8Context;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Network {
    Mainnet,
    Devnet,
    #[default]
    Local,
    Custom(String)
}

impl Network {
    pub fn client(&self) -> RpcClient {
        let url = match self {
            Network::Mainnet => "https://api.mainnet-beta.solana.com",
            Network::Devnet => "https://api.devnet.solana.com",
            Network::Local => "http://localhost:8899",
            Network::Custom(u) => u,
        };
        RpcClient::new(url.to_string()) 
    }
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Network::Mainnet => "mainnet",
            Network::Devnet => "devnet",
            Network::Local => "local",
            Network::Custom(n) => n
        })?;
        Ok(())
    }
}

impl FromStr for Network {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, anyhow::Error> {
        Ok(match s {
            "mainnet" => Network::Mainnet,
            "devnet" => Network::Devnet,
            "local" => Network::Local,
            s => Network::Custom(s.to_string())
        })
    }
}

impl Serialize for Network {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(&self.to_string())
    }
} 

impl<'de> Deserialize<'de> for Network {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d)
            .and_then(|v| Network::from_str(&v)
            .map_err(serde::de::Error::custom))
    }
}

pub fn get(ctx: &Valid8Context) -> Result<Network> {
    let mut items: Vec<String> = vec!["mainnet".into(), "devnet".into(), "custom".into(), "exit".into()];
    for network in ctx.networks.iter() {
        if !items.contains(&network.to_string()) {
            items.push(network.to_string())
        }
    }
    let selection = Select::new()
        .with_prompt("Select a network")
        .items(&items)
        .interact()?;
    
    match selection {
        0 => Ok(Network::Mainnet),
        1 => Ok(Network::Devnet),
        2 => custom_network(),
        3 => Err(Error::msg("Exit")),
        _ => if items.len() > selection {
            Ok(Network::Custom(items[selection].clone()))
        } else {
            Err(Error::msg("Invalid network selection"))
        }
    }   
}

pub fn custom_network() -> Result<Network> {
    let address = Input::new()
        .with_prompt("Network address")
        .with_initial_text("https://")
        .interact()?;
    Ok(Network::Custom(address))
}