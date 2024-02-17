use anyhow::Result;
use crate::context::Valid8Context;

pub fn install(ctx: &mut Valid8Context) -> Result<()> {
    ctx.install()?;
    println!("✅ Valid8 packages installed!");
    Ok(())
}