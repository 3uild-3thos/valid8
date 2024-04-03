use serde::{Deserialize, Deserializer, Serializer};
use solana_sdk::pubkey::Pubkey;
pub fn serialize<S: Serializer>(v: &Pubkey, s: S) -> Result<S::Ok, S::Error> {
    s.collect_str(&v.to_string())
}

pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Pubkey, D::Error> {
    let s = String::deserialize(d)?;
    let x: Vec<u8> = match bs58::decode(&s.as_bytes()).into_vec() {
        Ok(v) => v,
        Err(e) => return Err(serde::de::Error::custom(e))
    };
    if x.len() != 32 {
        return Err(serde::de::Error::custom("Invalid public key length"))
    }
    let mut b: [u8;32] = [0u8; 32];
    b.clone_from_slice(&x[0..32]);
    Ok(Pubkey::new_from_array(b))
}