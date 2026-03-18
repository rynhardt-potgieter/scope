/// `sc impact <symbol>` — analyse blast radius if a symbol changes.
///
/// Performs transitive reverse dependency traversal, showing direct callers,
/// second-degree dependents, and affected test files.
use anyhow::Result;
use clap::Args;

/// Arguments for the `sc impact` command.
#[derive(Args, Debug)]
pub struct ImpactArgs {
    /// Symbol name to analyse impact for.
    pub symbol: String,

    /// Maximum traversal depth (default: 3)
    #[arg(long, default_value = "3")]
    pub depth: usize,

    /// Output as JSON instead of human-readable format
    #[arg(long, short = 'j')]
    pub json: bool,
}

/// Run the `sc impact` command (stub).
pub fn run(args: &ImpactArgs) -> Result<()> {
    eprintln!(
        "impact command not yet implemented. Symbol: {}",
        args.symbol
    );
    Ok(())
}
