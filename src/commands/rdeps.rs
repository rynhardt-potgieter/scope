/// `sc rdeps <symbol>` — show what depends on a symbol (reverse dependencies).
///
/// Critical before any refactor or deletion. Shows all symbols and files
/// that depend on the given symbol.
use anyhow::Result;
use clap::Args;

/// Arguments for the `sc rdeps` command.
#[derive(Args, Debug)]
pub struct RdepsArgs {
    /// Symbol name to show reverse dependencies for.
    pub symbol: String,

    /// Transitive reverse dependency depth (default: 1, direct only)
    #[arg(long, default_value = "1")]
    pub depth: usize,

    /// Output as JSON instead of human-readable format
    #[arg(long, short = 'j')]
    pub json: bool,
}

/// Run the `sc rdeps` command (stub).
pub fn run(args: &RdepsArgs) -> Result<()> {
    eprintln!("rdeps command not yet implemented. Symbol: {}", args.symbol);
    Ok(())
}
