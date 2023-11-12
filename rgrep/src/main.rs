use anyhow::Result;
use clap::*;
use rgrep::*;

fn main() -> Result<()> {
    let config = GrepConfig::parse();
    config.match_with_default_strategy()?;
    Ok(())
}
