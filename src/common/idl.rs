use std::{collections::HashMap, fs::File, io::Read, mem, path::Path, str::FromStr};
use anyhow::{anyhow, Ok, Result};
use borsh::{BorshDeserialize, BorshSerialize};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use solana_sdk::{pubkey::Pubkey, hash::hash};
use convert_case::{Case, Casing};

use anchor_lang::anchor_syn::idl::types::{Idl, IdlField, IdlType, IdlTypeDefinition, IdlTypeDefinitionTy::Struct};


pub type Discriminator = [u8;8];
pub type DiscriminatorMap = HashMap<[u8;8], IdlTypeDefinition>;

#[derive(Debug)]
pub struct IdlAccountField {
    pub name: String,
    pub value: Option<FieldValue>,
    pub field_len: usize,
    pub orig_idl_field: IdlField
}

impl IdlAccountField {
    pub fn edit(&mut self, new_value: String) -> Result<()> {
        self.value = match &self.orig_idl_field.ty {
            IdlType::Bool | IdlType::U8 | IdlType::I8 | IdlType::U16 | IdlType::I16 | IdlType::U32 | IdlType::I32 | IdlType::U64 | IdlType::I64 | 
            IdlType::U128 | IdlType::I128 | IdlType::U256 | IdlType::I256 => Some(FieldValue::Number(new_value.parse::<usize>()?)),
            IdlType::F32 => unimplemented!(),
            IdlType::F64 => unimplemented!(),
            IdlType::Bytes => unimplemented!(),
            IdlType::String => Some(FieldValue::String(new_value)),
            IdlType::PublicKey => Some(FieldValue::Pubkey(Pubkey::from_str(&new_value)?)),
            IdlType::Defined(_) => unimplemented!(),
            IdlType::Option(_) => unimplemented!(),
            IdlType::Vec(_) => unimplemented!(),
            IdlType::Array(_, _) => unimplemented!(),
            IdlType::GenericLenArray(_, _) => unimplemented!(),
            IdlType::Generic(_) => unimplemented!(),
            IdlType::DefinedWithTypeArgs { name: _name, args: _args } => todo!(),
        };
        Ok(())
    }


    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut buf = vec![];
        match &self.orig_idl_field.ty {
            IdlType::Bool | IdlType::U8 => {
                if let Some(FieldValue::Number(num)) = &self.value {
                    u8::serialize(&u8::try_from(*num)?, &mut buf)?;
                }
            },
            IdlType::I8 => {              
                if let Some(FieldValue::Number(num)) = &self.value {
                    i8::serialize(&i8::try_from(*num)?, &mut buf)?;
                }
            }
            IdlType::U16 => {              
                if let Some(FieldValue::Number(num)) = &self.value {
                    u16::serialize(&u16::try_from(*num)?, &mut buf)?;
                }
            },
            IdlType::I16 => {              
                if let Some(FieldValue::Number(num)) = &self.value {
                    i16::serialize(&i16::try_from(*num)?, &mut buf)?;
                }
            },
            IdlType::U32 => {              
                if let Some(FieldValue::Number(num)) = &self.value {
                    u32::serialize(&u32::try_from(*num)?, &mut buf)?;
                }
            },
            IdlType::I32 => {              
                if let Some(FieldValue::Number(num)) = &self.value {
                    i32::serialize(&i32::try_from(*num)?, &mut buf)?;
                }
            },
            IdlType::F32 => unimplemented!(),
            IdlType::U64 => {              
                if let Some(FieldValue::Number(num)) = &self.value {
                    u64::serialize(&u64::try_from(*num)?, &mut buf)?;
                }
            },
            IdlType::I64 => {              
                if let Some(FieldValue::Number(num)) = &self.value {
                    i64::serialize(&i64::try_from(*num)?, &mut buf)?;
                }
            },
            IdlType::F64 => unimplemented!(),
            IdlType::U128 => {              
                if let Some(FieldValue::Number(num)) = &self.value {
                    u128::serialize(&u128::try_from(*num)?, &mut buf)?;
                }
            },
            IdlType::I128 => {              
                if let Some(FieldValue::Number(num)) = &self.value {
                    i128::serialize(&i128::try_from(*num)?, &mut buf)?;
                }
            },
            IdlType::U256 => unimplemented!(),
            IdlType::I256 => unimplemented!(),
            IdlType::Bytes => {              
                if let Some(FieldValue::Bytes(bytes)) = &self.value {
                    buf = bytes.clone();
                }
            },
            IdlType::String => {              
                if let Some(FieldValue::String(value)) = &self.value {
                    String::serialize(value, &mut buf)?;
                }
            },
            IdlType::PublicKey => {              
                if let Some(FieldValue::Pubkey(pubkey)) = &self.value {
                    Pubkey::serialize(pubkey, &mut buf)?;
                }
            },
            IdlType::Defined(_) => unimplemented!(),
            IdlType::Option(_) => unimplemented!(),
            IdlType::Vec(_) => unimplemented!(),
            IdlType::Array(idl_type, _length) => {
                if let Some(FieldValue::Array(field_values)) = &self.value {
                    let _ = field_values.iter().map(|fv| {
                        let gen_field = IdlAccountField {
                            name: "".into(),
                            value: Some(fv.clone()),
                            field_len: 0,
                            orig_idl_field: IdlField { name: "".into(), docs: None, ty: *idl_type.clone() },
                        };
                        buf.extend_from_slice(&gen_field.to_bytes()?);
                        Ok(())
                    }).collect::<Result<Vec<()>>>()?;
                }
            },
            IdlType::GenericLenArray(_, _) => unimplemented!(),
            IdlType::Generic(_) => unimplemented!(),
            IdlType::DefinedWithTypeArgs { name: _, args: _ } => unimplemented!(),
        }
        Ok(buf)

    }
}

pub fn unpack_data_idl_type(idl_type: &IdlType, data: &[u8]) -> Result<FieldValue> {
    match idl_type {
        IdlType::Bool => Ok(FieldValue::Number(u8::try_from_slice(data)? as usize)),
        IdlType::U8 => Ok(FieldValue::Number(u8::try_from_slice(data)?as usize)),
        IdlType::I8 => Ok(FieldValue::Number(i8::try_from_slice(data)? as usize)),
        IdlType::U16 => Ok(FieldValue::Number(u16::try_from_slice(data)? as usize)),
        IdlType::I16 => Ok(FieldValue::Number(i16::try_from_slice(data)? as usize)),
        IdlType::U32 => Ok(FieldValue::Number(u32::try_from_slice(data)? as usize)),
        IdlType::I32 => Ok(FieldValue::Number(i32::try_from_slice(data)? as usize)),
        IdlType::F32 => Ok(FieldValue::Number(f32::try_from_slice(data)? as usize)),
        IdlType::U64 => Ok(FieldValue::Number(u64::try_from_slice(data)? as usize)),
        IdlType::I64 => Ok(FieldValue::Number(i64::try_from_slice(data)? as usize)),
        IdlType::F64 => Ok(FieldValue::Number(f64::try_from_slice(data)? as usize)),
        IdlType::U128 => Ok(FieldValue::Number(u128::try_from_slice(data)? as usize)),
        IdlType::I128 => Ok(FieldValue::Number(i128::try_from_slice(data)? as usize)),
        IdlType::U256 => todo!(),
        IdlType::I256 => todo!(),
        IdlType::Bytes => Ok(FieldValue::Bytes(data.to_vec())),
        IdlType::String => Ok(FieldValue::String(String::from_utf8(data.to_vec())?)),
        IdlType::PublicKey => Ok(FieldValue::Pubkey(Pubkey::try_from_slice(data)?)),
        IdlType::Defined(_) => todo!(),
        IdlType::Option(_) => todo!(),
        IdlType::Vec(_) => todo!(),
        IdlType::Array(item_type, length) => {
            let idl_type_len = idl_type_len(item_type);
            let mut offset = 0usize;
            let collection = (0..*length).map(|_| {
                let item = unpack_data_idl_type(item_type, &data[offset..offset+idl_type_len])?;
                offset += idl_type_len;
                Ok(item)
            }).collect::<Result<Vec<FieldValue>>>()?;

            Ok(FieldValue::Array(collection))
        },
        IdlType::GenericLenArray(_, _) => todo!(),
        IdlType::Generic(_) => todo!(),
        IdlType::DefinedWithTypeArgs { name: _, args: _ } => todo!(),
    }
}

#[derive(Debug, Clone)]
pub enum FieldValue {
    String(String),
    Number(usize),
    Bytes(Vec<u8>),
    Pubkey(Pubkey),
    Array(Vec<FieldValue>)
}

pub fn open_idl(pubkey: &Pubkey) -> Result<Idl> {
    let mut b: Vec<u8> = vec![];
    let mut f = File::open(Path::new(&format!("./.valid8/{}.idl.json", pubkey.to_string())))?;
    f.read_to_end(&mut b)?;
    let schema: Idl = serde_json::from_slice(&b)?;
    Ok(schema)
}

pub fn generate_discriminator_map(idl: &Idl) -> Result<DiscriminatorMap> {
    let map: DiscriminatorMap = idl.accounts.par_iter().map(|a| {
        let mut discriminator: Discriminator = [0u8;8];
        discriminator[0..8].copy_from_slice(&hash(format!("account:{}", a.name).as_bytes()).to_bytes()[0..8]);
        (discriminator, a.clone())
    }).collect();

    Ok(map)
}

pub fn unpack_idl_account(idl_type_def: &IdlTypeDefinition, data: Vec<u8>) -> Result<Vec<IdlAccountField>> {
    let mut account_map: Vec<IdlAccountField> = vec!();
    match idl_type_def.ty.clone() {
        Struct { fields } => {
            let mut offset = 0usize;
            for field in fields {
                let mut idl_field = unpack_idl_field(field)?;
                if let Some(data_slice) = &data.get(offset..offset + idl_field.field_len) {
                    idl_field.value = Some(unpack_data_idl_type(&idl_field.orig_idl_field.ty, data_slice)?);
                    offset += &idl_field.field_len;
                    account_map.push(idl_field);
                }
            };
            Ok(account_map)
        },
        _ => Err(anyhow!("Unsupported IDL type: {:?}", idl_type_def.ty)),
    }
}

pub fn unpack_idl_field(idl_field: IdlField) -> Result<IdlAccountField> {

    let unpacked = IdlAccountField {
        name: idl_field.name.clone(),
        value: None,
        field_len: idl_type_len(&idl_field.ty),
        orig_idl_field: idl_field
    };
    Ok(unpacked)
}

pub fn idl_type_len(idl_type: &IdlType) -> usize {
    match idl_type {
        IdlType::Bool => mem::size_of::<u8>(),
        IdlType::U8 => mem::size_of::<u8>(),
        IdlType::I8 => mem::size_of::<i8>(),
        IdlType::U16 => mem::size_of::<u16>(),
        IdlType::I16 => mem::size_of::<i16>(),
        IdlType::U32 => mem::size_of::<u32>(),
        IdlType::I32 => mem::size_of::<i32>(),
        IdlType::F32 => mem::size_of::<f32>(),
        IdlType::U64 => mem::size_of::<u64>(),
        IdlType::I64 => mem::size_of::<i64>(),
        IdlType::F64 => unimplemented!(),
        IdlType::U128 => mem::size_of::<u128>(),
        IdlType::I128 => mem::size_of::<i128>(),
        IdlType::U256 => unimplemented!(),
        IdlType::I256 => unimplemented!(),
        IdlType::Bytes => unimplemented!(),
        IdlType::String => unimplemented!(),
        IdlType::PublicKey => mem::size_of::<Pubkey>(),
        IdlType::Defined(_defined) => unimplemented!(),
        IdlType::Option(_) => unimplemented!(),
        IdlType::Vec(idl_type) => {
            idl_type_len(idl_type)
        },
        IdlType::Array(idl_type, length) => {
            length*idl_type_len(idl_type)
        },
        IdlType::GenericLenArray(_, _) => unimplemented!(),
        IdlType::Generic(_) => unimplemented!(),
        IdlType::DefinedWithTypeArgs { name: _, args: _ } => unimplemented!(),
    }
}
