/// `scope flow <start> <end>` — find call paths between two symbols.
///
/// Unlike `scope trace` (entry points → target), this finds paths between
/// any two arbitrary symbols through the call graph. Use it to understand
/// how data or control flows between two specific points in the codebase.
///
/// Examples:
///   scope flow PaymentService OrderController
///   scope flow processPayment handleWebhook --depth 5
///   scope flow "src/auth.ts::validate" "src/api.ts::respond" --json
use anyhow::{bail, Result};
use clap::Args;
use serde::Serialize;
use std::path::Path;

use crate::commands::resolve_symbol;
use crate::core::graph::Graph;
use crate::output::formatter;
use crate::output::json::JsonOutput;

/// Find call paths between two symbols.
///
/// Shows how <start> reaches <end> through the call graph.
/// Use this when you need to understand how data or control flows
/// between two specific points in the codebase.
///
/// Unlike `scope trace` (entry points → target), this finds paths
/// between any two symbols.
///
/// Examples:
///   scope flow PaymentService OrderController
///   scope flow processPayment handleWebhook --depth 5
///   scope flow "src/auth.ts::validate" "src/api.ts::respond" --json
#[derive(Args, Debug)]
pub struct FlowArgs {
    /// Source symbol name — where the path starts
    pub start: String,

    /// Target symbol name — where the path ends
    pub end: String,

    /// Maximum path length (number of edges to traverse)
    #[arg(long, default_value = "10")]
    pub depth: usize,

    /// Maximum number of paths to display
    #[arg(long, default_value = "5")]
    pub limit: usize,

    /// Output as JSON instead of human-readable format
    #[arg(long, short = 'j')]
    pub json: bool,
}

/// JSON-serializable output for the flow command.
#[derive(Debug, Serialize)]
pub struct FlowOutput {
    /// The start symbol name.
    pub start: String,
    /// The end symbol name.
    pub end: String,
    /// All discovered call paths from start to end.
    pub paths: Vec<FlowPath>,
    /// Total number of paths found (before limit truncation).
    pub total: usize,
    /// The depth limit that was applied.
    pub depth_limit: usize,
}

/// A single call path from start to end.
#[derive(Debug, Serialize)]
pub struct FlowPath {
    /// Ordered steps from start to end.
    pub steps: Vec<FlowStep>,
}

/// A single step in a flow path.
#[derive(Debug, Serialize)]
pub struct FlowStep {
    /// Display name of the symbol.
    pub name: String,
    /// File path where this symbol is defined.
    pub file_path: String,
    /// Line number of the symbol definition.
    pub line_start: u32,
    /// Symbol kind (function, class, method, etc.).
    pub kind: String,
}

/// Run the `scope flow` command.
pub fn run(args: &FlowArgs, project_root: &Path) -> Result<()> {
    let scope_dir = project_root.join(".scope");

    if !scope_dir.exists() {
        bail!("No .scope/ directory found. Run 'scope init' first.");
    }

    let db_path = scope_dir.join("graph.db");
    if !db_path.exists() {
        bail!("No index found. Run 'scope index' to build one first.");
    }

    let graph = Graph::open(&db_path)?;
    crate::commands::warn_if_stale(&graph, project_root);

    // Resolve start symbol (with disambiguation)
    let start_sym = resolve_symbol(&graph, &args.start)?;

    // Resolve end symbol (with disambiguation)
    let end_sym = resolve_symbol(&graph, &args.end)?;

    // Fetch one extra to detect truncation
    let raw_paths =
        graph.find_flow_paths(&start_sym.id, &end_sym.id, args.depth, args.limit + 1)?;
    let truncated = raw_paths.len() > args.limit;
    let total = raw_paths.len().min(args.limit);
    let raw_paths: Vec<_> = raw_paths.into_iter().take(args.limit).collect();

    // Convert to FlowPath structs
    let paths: Vec<FlowPath> = raw_paths
        .into_iter()
        .map(|steps| FlowPath {
            steps: steps
                .into_iter()
                .map(|s| FlowStep {
                    name: s.symbol_name,
                    file_path: s.file_path,
                    line_start: s.line,
                    kind: s.kind,
                })
                .collect(),
        })
        .collect();

    if args.json {
        let output = FlowOutput {
            start: args.start.clone(),
            end: args.end.clone(),
            paths,
            total,
            depth_limit: args.depth,
        };
        let json_envelope = JsonOutput {
            command: "flow",
            symbol: None,
            data: &output,
            truncated,
            total,
        };
        println!("{}", serde_json::to_string_pretty(&json_envelope)?);
    } else {
        formatter::print_flow(&args.start, &args.end, &paths, total, args.depth);
    }

    Ok(())
}
