/// `scope similar <symbol>` — find structurally similar symbols.
///
/// Uses embeddings to find symbols with similar structure or semantics.
/// Useful for discovering existing implementations before writing new code.
use anyhow::Result;
use clap::Args;

/// Arguments for the `scope similar` command.
#[derive(Args, Debug)]
pub struct SimilarArgs {
    /// Symbol name to find similar symbols for.
    pub symbol: String,

    /// Filter by symbol kind: function, class, method
    #[arg(long)]
    pub kind: Option<String>,

    /// Maximum number of results to show
    #[arg(long, default_value = "10")]
    pub limit: usize,

    /// Output as JSON instead of human-readable format
    #[arg(long, short = 'j')]
    pub json: bool,
}

/// Run the `scope similar` command (stub).
pub fn run(args: &SimilarArgs) -> Result<()> {
    eprintln!(
        "similar command not yet implemented. Symbol: {}",
        args.symbol
    );
    Ok(())
}
