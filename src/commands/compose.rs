use anyhow::Result;
use crate::context::Valid8Context;

pub fn compose(ctx: Valid8Context) -> Result<()> {
    let (compose_count, account_count, program_count) = ctx.try_compose()?;

    println!("✅ Valid8 configs composed! {} accounts and {} programs added from {} config(s)", account_count, program_count, compose_count);
    Ok(())
}