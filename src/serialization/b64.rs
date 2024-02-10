use base64::Engine;
use serde::Deserialize;
use serde::{Deserializer, Serializer};


pub fn serialize<S: Serializer>(v: &Vec<u8>, s: S) -> Result<S::Ok, S::Error> {
    let mut buf = String::new();
    base64::engine::general_purpose::STANDARD.encode_string(v, &mut buf);
    s.collect_str(&buf)
}

pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
    let s = String::deserialize(d)?;
    base64::engine::general_purpose::STANDARD.decode(&s.as_bytes())
        .map(|v| v)
        .map_err(|e| serde::de::Error::custom(e))
}