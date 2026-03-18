/// `sc refs <symbol>` — find all references to a symbol.
///
/// Returns all call sites, imports, type annotations, and other references
/// across the codebase. Use before changing a function signature.
use anyhow::Result;
use clap::Args;

/// Arguments for the `sc refs` command.
#[derive(Args, Debug)]
pub struct RefsArgs {
    /// Symbol name to find references for.
    ///
    /// Examples: processPayment, PaymentService
    pub symbol: String,

    /// Filter by edge kind: calls, imports, extends, implements, instantiates, references
    #[arg(long)]
    pub kind: Option<String>,

    /// Maximum number of references to show
    #[arg(long, default_value = "20")]
    pub limit: usize,

    /// Output as JSON instead of human-readable format
    #[arg(long, short = 'j')]
    pub json: bool,
}

/// Run the `sc refs` command (stub).
pub fn run(args: &RefsArgs) -> Result<()> {
    eprintln!("refs command not yet implemented. Symbol: {}", args.symbol);
    Ok(())
}
