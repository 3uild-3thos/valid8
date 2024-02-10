use std::{str::FromStr, fmt::Display};
use serde::{ser::Serialize, de::Deserialize, Serializer, Deserializer};
use anyhow::{Result, Error};
use dialoguer::{Input, Select};
use solana_client::rpc_client::RpcClient;

#[derive(Debug, Clone)]
pub enum Network {
    Mainnet,
    Devnet,
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
        let client = RpcClient::new(url.to_string());   
        client 
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
        let v = String::deserialize(d)?;
        Network::from_str(&v)
            .map_err(serde::de::Error::custom)
    }
}

pub fn command() -> Result<Network> {
    let mut items = vec!["Mainnet Beta", "Devnet"];
    // TODO: Push any custom networks defined in our JSON file
    items.push("Custom");
    items.push("Exit");
    let selection = Select::new()
        .with_prompt("Select a network")
        .items(&items)
        .interact()?;
    
    match selection {
        0 => Ok(Network::Mainnet),
        1 => Ok(Network::Devnet),
        2 => custom_network(),
        _ => return Err(Error::msg("Invalid network selection"))
    }
}

pub fn custom_network() -> Result<Network> {
    let address = Input::new()
        .with_prompt("Network address")
        .with_initial_text("https://")
        .interact()?;
    Ok(Network::Custom(address))
}