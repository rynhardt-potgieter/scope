/// `sc source <symbol>` — fetch full source of a specific symbol.
///
/// Returns the exact source code of the symbol, including its full definition.
/// Only call this when ready to read or edit the implementation.
use anyhow::Result;
use clap::Args;

/// Arguments for the `sc source` command.
#[derive(Args, Debug)]
pub struct SourceArgs {
    /// Symbol name to fetch source for.
    ///
    /// Examples: processPayment, PaymentService.validateCard
    pub symbol: String,

    /// Output as JSON instead of human-readable format
    #[arg(long, short = 'j')]
    pub json: bool,
}

/// Run the `sc source` command (stub).
pub fn run(args: &SourceArgs) -> Result<()> {
    eprintln!(
        "source command not yet implemented. Symbol: {}",
        args.symbol
    );
    Ok(())
}
