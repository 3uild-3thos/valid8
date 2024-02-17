use std::path::Path;

use anyhow::Result;

use crate::{APP_NAME, commands, context::Valid8Context};

pub fn init(ctx: &mut Valid8Context) -> Result<()> {
    let config_path = Path::new(&ctx.project_name.to_string());
    // create_resources_dir()?;
    match config_path.exists() {
        false => {
            // create_new_workspace()?;
            println!("Initialized new {} workspace.", APP_NAME);
            commands::edit(ctx)?
        },
        true => {
            println!("{} workspace already exists.", APP_NAME)
        }
    }
    Ok(())
}