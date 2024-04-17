use anyhow::Result;
use crate::context::Valid8Context;

pub fn compose(ctx: &mut Valid8Context) -> Result<()> {
    let account_count = ctx.try_compose()?;

    println!("✅ Valid8 configs composed! accounts added: {}", account_count);
    Ok(())
}