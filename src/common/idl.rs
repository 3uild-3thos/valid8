use std::{collections::HashMap, path::Path, fs::File, io::Read, fmt::format};
use anyhow::Result;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
// use serde_json::Value;
use solana_program::{pubkey::Pubkey, hash::hash};
use anchor_lang::anchor_syn::idl::types::{Idl, IdlTypeDefinition};
pub type Discriminator = [u8;8];
pub type DiscriminatorMap = HashMap<[u8;8], IdlTypeDefinition>;

pub fn open_idl(pubkey: &Pubkey) -> Result<Idl> {
    let mut b: Vec<u8> = vec![];
    let mut f = File::open(Path::new(&format!("./.valid8/{}.idl.json", pubkey.to_string())))?;
    f.read_to_end(&mut b)?;
    let schema: Idl = serde_json::from_slice(&b)?;
    Ok(schema)
}
// pub fn generate_discriminator_map(idl: &Idl) -> Result<DiscriminatorMap> {
//     let mut map: DiscriminatorMap = HashMap::new();
//     idl.accounts.par_iter().for_each(|a| {
//         let mut discriminator: Discriminator = [0u8;8];
//         discriminator[0..8].copy_from_slice(&hash(format!("global:{}", a.name).as_bytes()).to_bytes()[0..8]);
//         map.insert(discriminator, a.clone());
//     });
//     Ok(map)
// }

pub fn get_account_schema(idl: DiscriminatorMap) -> Result<IdlTypeDefinition> {
    idl.get_key_value(k)
}