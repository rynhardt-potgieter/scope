/// `scope impact <symbol>` — analyse blast radius if a symbol changes.
///
/// **Deprecated**: Use `scope callers <symbol> --depth N` instead.
///
/// Performs transitive reverse dependency traversal, showing direct callers,
/// second-degree dependents, and affected test files.
///
/// Examples:
///   scope impact processPayment             — who breaks if this changes
///   scope impact PaymentConfig              — blast radius of config change
///   scope impact src/types/payment.ts       — impact of changing a types file
use anyhow::Result;
use clap::Args;
use std::path::Path;

use super::refs::{run_callers_transitive, CallersArgs};

/// Arguments for the `scope impact` command.
#[derive(Args, Debug)]
pub struct ImpactArgs {
    /// Symbol name or file path to analyse impact for.
    ///
    /// Pass a symbol name to see what breaks if it changes.
    /// Pass a file path to see the combined impact of all symbols in that file.
    ///
    /// Examples: processPayment, PaymentConfig, src/types/payment.ts
    pub symbol: String,

    /// Maximum traversal depth (default: 3).
    ///
    /// Depth 1 = direct callers only. Depth 2 = callers of callers. etc.
    #[arg(long, default_value = "3")]
    pub depth: usize,

    /// Output as JSON instead of human-readable format
    #[arg(long, short = 'j')]
    pub json: bool,
}

/// Run the `scope impact` command.
///
/// Prints a deprecation notice to stderr and delegates to `scope callers --depth N`.
pub fn run(args: &ImpactArgs, project_root: &Path) -> Result<()> {
    eprintln!(
        "Note: 'scope impact' is deprecated. Use 'scope callers {} --depth {}' instead.",
        args.symbol, args.depth
    );

    let callers_args = CallersArgs {
        symbol: args.symbol.clone(),
        depth: args.depth,
        limit: 20,
        context: 0,
        json: args.json,
    };

    run_callers_transitive(&callers_args, project_root, "impact")
}
