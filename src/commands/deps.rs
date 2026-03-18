/// `sc deps <symbol>` — show what a symbol depends on.
///
/// Lists direct imports, calls, and type references. Use `--depth 2`
/// for transitive dependencies.
use anyhow::Result;
use clap::Args;

/// Arguments for the `sc deps` command.
#[derive(Args, Debug)]
pub struct DepsArgs {
    /// Symbol name or file path to show dependencies for.
    pub symbol: String,

    /// Transitive dependency depth (default: 1, direct only)
    #[arg(long, default_value = "1")]
    pub depth: usize,

    /// Output as JSON instead of human-readable format
    #[arg(long, short = 'j')]
    pub json: bool,
}

/// Run the `sc deps` command (stub).
pub fn run(args: &DepsArgs) -> Result<()> {
    eprintln!("deps command not yet implemented. Symbol: {}", args.symbol);
    Ok(())
}
