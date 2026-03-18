/// `sc impact <symbol>` — analyse blast radius if a symbol changes.
///
/// Performs transitive reverse dependency traversal, showing direct callers,
/// second-degree dependents, and affected test files.
///
/// Examples:
///   sc impact processPayment             — who breaks if this changes
///   sc impact PaymentConfig              — blast radius of config change
///   sc impact src/types/payment.ts       — impact of changing a types file
use anyhow::{bail, Result};
use clap::Args;
use std::path::Path;

use crate::core::graph::Graph;
use crate::output::formatter;
use crate::output::json::JsonOutput;

/// Arguments for the `sc impact` command.
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

use super::looks_like_file_path;

/// Run the `sc impact` command.
pub fn run(args: &ImpactArgs, project_root: &Path) -> Result<()> {
    let scope_dir = project_root.join(".scope");

    if !scope_dir.exists() {
        bail!("No .scope/ directory found. Run 'sc init' first.");
    }

    let db_path = scope_dir.join("graph.db");
    if !db_path.exists() {
        bail!("No index found. Run 'sc index' to build one first.");
    }

    let graph = Graph::open(&db_path)?;

    if looks_like_file_path(&args.symbol) {
        return run_file_impact(args, &graph);
    }

    run_symbol_impact(args, &graph)
}

/// Show impact for a single symbol.
fn run_symbol_impact(args: &ImpactArgs, graph: &Graph) -> Result<()> {
    let result = graph.find_impact(&args.symbol, args.depth)?;

    if args.json {
        let output = JsonOutput {
            command: "impact",
            symbol: Some(args.symbol.clone()),
            data: &result,
            truncated: false,
            total: result.total_affected + result.test_files.len(),
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        formatter::print_impact(&args.symbol, &result);
    }

    Ok(())
}

/// Show impact for all symbols in a file.
fn run_file_impact(args: &ImpactArgs, graph: &Graph) -> Result<()> {
    let file_path = formatter::normalize_path(&args.symbol);
    let result = graph.find_file_impact(&file_path, args.depth)?;

    if args.json {
        let output = JsonOutput {
            command: "impact",
            symbol: Some(file_path.clone()),
            data: &result,
            truncated: false,
            total: result.total_affected + result.test_files.len(),
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        formatter::print_impact(&file_path, &result);
    }

    Ok(())
}
