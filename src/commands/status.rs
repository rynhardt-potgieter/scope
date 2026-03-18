/// `sc status` — show index status and freshness.
///
/// Quick health check: is the index built? How many symbols and files?
/// Are there stale/unindexed files?
use anyhow::Result;
use clap::Args;

/// Arguments for the `sc status` command.
#[derive(Args, Debug)]
pub struct StatusArgs {
    /// Output as JSON instead of human-readable format
    #[arg(long, short = 'j')]
    pub json: bool,
}

/// Run the `sc status` command (stub).
pub fn run(args: &StatusArgs) -> Result<()> {
    let _ = args;
    eprintln!("status command not yet implemented.");
    Ok(())
}
