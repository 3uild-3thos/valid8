use std::{fmt::Display, str::FromStr};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectName {
    name: String
}

impl ProjectName {
    pub const DEFAULT: &'static str = "valid8";

    pub fn to_config(&self) -> String {
        String::from(format!("{}.json", self.name))
    }
    pub fn to_resources(&self) -> String {
        String::from(format!(".{}/", self.name))
    }
}

impl Default for ProjectName {
    // Default project name is valid8
    fn default() -> Self {
        Self { name: Self::DEFAULT.into() }
    }
}

impl FromStr for ProjectName {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self{ name: s.to_string() })
    }
    
}

impl Display for ProjectName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
