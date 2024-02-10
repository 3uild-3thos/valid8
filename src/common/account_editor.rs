// use super::{AccountSchema, AccountField};

// // pub fn edit_account(account: &mut AccountSchema) {
// //     account.owner = 
// // }

// // pub fn edit_owner(ctx: &mut AccountSchema) -> {
// //     let new_owner = 
// // }

// // pub enum types {
// //     U8,
// //     U16,
// //     U32,
// //     U64,
// //     U128,
// //     USize,
// //     I8,
// //     I16,
// //     I32,
// //     I64,
// //     I128,
// //     ISize,
// //     Bool,
// //     String,
// //     VecU8,
// //     Pubkey
// // }

// use anyhow::Result;
// use dialoguer::Select;

// use crate::{program, account, context::Valid8Context};

// pub fn command(ctx: &mut Valid8Context) -> Result<()> {
//     let items = ctx.accounts.into_iter().map(|a| {
//         a.0
//     }) vec![
//         "Clone Program",
//         "Edit Program", 
//         "Clone Account", 
//         "Edit Account"
//     ];

//     let selection = Select::new()
//         .with_prompt("Select an option")
//         .items(&items)
//         .interact_opt()?;

//     if let Some(n) = selection {
//         match n {
//             0 => program::clone::command(ctx)?,
//             1 => todo!(), //program::edit::command()?,
//             2 => account::clone::command(ctx)?,
//             3 => todo!(), // account::edit::command()?,
//             _ => todo!()
//         }
//     }

//     Ok(())
// }