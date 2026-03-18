/// `sc deps <symbol>` — show what a symbol depends on.
///
/// Lists direct imports, calls, and type references. Use `--depth 2`
/// for transitive dependencies. Pass a file path to see dependencies
/// of all symbols in that file.
///
/// Examples:
///   sc deps PaymentService               — direct dependencies of a class
///   sc deps PaymentService --depth 2     — transitive dependencies
///   sc deps src/payments/service.ts      — dependencies of a whole file
use anyhow::{bail, Result};
use clap::Args;
use std::path::Path;

use crate::core::graph::Graph;
use crate::output::formatter;
use crate::output::json::JsonOutput;

/// Arguments for the `sc deps` command.
#[derive(Args, Debug)]
pub struct DepsArgs {
    /// Symbol name or file path to show dependencies for.
    ///
    /// Pass a class or function name to see what it imports and calls.
    /// Pass a file path to see dependencies of all symbols in that file.
    ///
    /// Examples: PaymentService, processPayment, src/payments/service.ts
    pub symbol: String,

    /// Transitive dependency depth (default: 1, direct only)
    #[arg(long, default_value = "1")]
    pub depth: usize,

    /// Output as JSON instead of human-readable format
    #[arg(long, short = 'j')]
    pub json: bool,
}

use super::looks_like_file_path;

/// Run the `sc deps` command.
pub fn run(args: &DepsArgs, project_root: &Path) -> Result<()> {
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
        return run_file_deps(args, &graph);
    }

    run_symbol_deps(args, &graph)
}

/// Show dependencies for a single symbol.
fn run_symbol_deps(args: &DepsArgs, graph: &Graph) -> Result<()> {
    let deps = graph.find_deps(&args.symbol, args.depth)?;

    if args.json {
        let output = JsonOutput {
            command: "deps",
            symbol: Some(args.symbol.clone()),
            data: &deps,
            truncated: false,
            total: deps.len(),
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        formatter::print_deps(&args.symbol, &deps, args.depth);
    }

    Ok(())
}

/// Show dependencies for all symbols in a file.
fn run_file_deps(args: &DepsArgs, graph: &Graph) -> Result<()> {
    let file_path = formatter::normalize_path(&args.symbol);
    let deps = graph.find_file_deps(&file_path, args.depth)?;

    if args.json {
        let output = JsonOutput {
            command: "deps",
            symbol: Some(file_path.clone()),
            data: &deps,
            truncated: false,
            total: deps.len(),
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        formatter::print_file_deps(&file_path, &deps, args.depth);
    }

    Ok(())
}
