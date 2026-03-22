/// `scope trace <symbol>` — show how requests reach a symbol.
///
/// Traces the call graph backward from the target to find entry points
/// (symbols with no incoming calls). Shows every path from an entry
/// point through intermediate callers to the target.
///
/// Use this to understand how a bug is triggered or how a method is
/// reached from API endpoints, workers, or event handlers.
///
/// Examples:
///   scope trace processPayment              — paths to a function
///   scope trace SubscriptionService.processRenewal — paths to a method
use anyhow::{bail, Result};
use clap::Args;
use std::path::Path;

use crate::core::graph::Graph;
use crate::output::formatter;
use crate::output::json::JsonOutput;

/// Show how requests reach a symbol — entry-point-to-target call paths.
///
/// Traces the call graph backward from the target to find entry points
/// (symbols with no incoming calls). Shows every path from an entry
/// point through intermediate callers to the target.
///
/// Use this to understand how a bug is triggered or how a method is
/// reached from API endpoints, workers, or event handlers.
///
/// Examples:
///   scope trace processPayment
///   scope trace SubscriptionService.processRenewal
#[derive(Args, Debug)]
pub struct TraceArgs {
    /// Symbol name to trace paths to
    pub symbol: String,

    /// Maximum call chain depth to search
    #[arg(long, default_value = "10")]
    pub max_depth: usize,

    /// Maximum number of paths to display (default: 20)
    #[arg(long, default_value = "20")]
    pub limit: usize,

    /// Output as JSON instead of human-readable format
    #[arg(long, short = 'j')]
    pub json: bool,
}

/// Run the `scope trace` command.
pub fn run(args: &TraceArgs, project_root: &Path) -> Result<()> {
    let scope_dir = project_root.join(".scope");

    if !scope_dir.exists() {
        bail!("No .scope/ directory found. Run 'scope init' first.");
    }

    let db_path = scope_dir.join("graph.db");
    if !db_path.exists() {
        bail!("No index found. Run 'scope index' to build one first.");
    }

    let graph = Graph::open(&db_path)?;

    let symbol = graph.find_symbol(&args.symbol)?.ok_or_else(|| {
        anyhow::anyhow!(
            "Symbol '{}' not found in index.\n\
             Tip: Check spelling, or use 'scope find \"{}\"' for semantic search.",
            args.symbol,
            args.symbol
        )
    })?;

    let mut result = graph.find_call_paths(&symbol.id, &symbol.name, args.max_depth)?;
    let total = result.paths.len();
    let truncated = total > args.limit;

    if truncated {
        result.paths.truncate(args.limit);
    }

    if args.json {
        let output = JsonOutput {
            command: "trace",
            symbol: Some(args.symbol.clone()),
            data: &result,
            truncated,
            total,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        formatter::print_trace(&args.symbol, &result);
        if truncated {
            println!("... {} more paths (use --limit to show more)", total - args.limit);
        }
    }

    Ok(())
}
