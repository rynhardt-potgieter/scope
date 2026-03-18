/// `sc sketch <symbol>` — show structural overview of a symbol.
///
/// Returns the class/function signature, dependencies, methods with caller counts,
/// and type information. Use this before `sc source` to understand structure first.
use anyhow::Result;
use clap::Args;

/// Arguments for the `sc sketch` command.
#[derive(Args, Debug)]
pub struct SketchArgs {
    /// Symbol name or file path to sketch.
    ///
    /// Examples: PaymentService, PaymentService.processPayment, src/payments/service.ts
    pub symbol: String,

    /// Output as JSON instead of human-readable format
    #[arg(long, short = 'j')]
    pub json: bool,

    /// Maximum number of methods to show (default: all)
    #[arg(long, default_value = "50")]
    pub limit: usize,
}

/// Run the `sc sketch` command (stub).
pub fn run(args: &SketchArgs) -> Result<()> {
    eprintln!(
        "sketch command not yet implemented. Symbol: {}",
        args.symbol
    );
    Ok(())
}
