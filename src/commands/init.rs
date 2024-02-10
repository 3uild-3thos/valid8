use std::path::Path;

use anyhow::Result;

use crate::{APP_NAME, commands::edit, common::helpers::{create_resources_dir, create_new_workspace}, context::Valid8Context};

pub fn command(ctx: &mut Valid8Context) -> Result<()> {
    let filename = format!("{}.json", APP_NAME.to_ascii_lowercase());
    let config_path = Path::new(&filename);
    create_resources_dir()?;
    match config_path.exists() {
        false => {
            create_new_workspace()?;
            println!("Initialized new {} workspace.", APP_NAME);
            edit::command(ctx)?
        },
        true => {
            println!("{} workspace already exists.", APP_NAME)
        }
    }
    Ok(())
}