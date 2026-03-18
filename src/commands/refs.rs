/// `sc refs <symbol>` — find all references to a symbol.
///
/// Returns all call sites, imports, type annotations, and other references
/// across the codebase. Use before changing a function signature to find all callers.
///
/// For class symbols, references are grouped by kind (instantiated, extended,
/// imported, used as type). For functions/methods, a flat list is shown.
///
/// Examples:
///   sc refs processPayment              — all references to a function
///   sc refs PaymentService              — grouped references to a class
///   sc refs PaymentService --kind calls — only call sites
///   sc refs src/payments/service.ts     — all refs to symbols in a file
use anyhow::{bail, Result};
use clap::Args;
use std::path::Path;

use crate::core::graph::Graph;
use crate::output::formatter;
use crate::output::json::JsonOutput;

/// Arguments for the `sc refs` command.
#[derive(Args, Debug)]
pub struct RefsArgs {
    /// Symbol name or file path to find references for.
    ///
    /// Pass a function/method name to see all call sites.
    /// Pass a class name to see references grouped by kind.
    /// Pass a file path to see references to all symbols in that file.
    ///
    /// Examples: processPayment, PaymentService, src/payments/service.ts
    pub symbol: String,

    /// Filter by edge kind: calls, imports, extends, implements, instantiates, references
    #[arg(long)]
    pub kind: Option<String>,

    /// Maximum number of references to show (default: 20)
    #[arg(long, default_value = "20")]
    pub limit: usize,

    /// Output as JSON instead of human-readable format
    #[arg(long, short = 'j')]
    pub json: bool,
}

use super::looks_like_file_path;

/// Run the `sc refs` command.
pub fn run(args: &RefsArgs, project_root: &Path) -> Result<()> {
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
        return run_file_refs(args, &graph);
    }

    run_symbol_refs(args, &graph)
}

/// Find refs for a single symbol.
fn run_symbol_refs(args: &RefsArgs, graph: &Graph) -> Result<()> {
    let kinds: Option<Vec<&str>> = args.kind.as_deref().map(|k| vec![k]);
    let kinds_slice = kinds.as_deref();

    // Check if this is a class-like symbol for grouped output
    let is_class = graph.is_class_like(&args.symbol)?;

    if is_class && kinds_slice.is_none() {
        // Grouped output for class symbols
        let (groups, total) = graph.find_refs_grouped(&args.symbol, args.limit)?;

        if args.json {
            let data = serde_json::json!({
                "groups": groups.iter().map(|(kind, refs)| {
                    serde_json::json!({
                        "kind": kind,
                        "refs": refs,
                    })
                }).collect::<Vec<_>>(),
            });
            let output = JsonOutput {
                command: "refs",
                symbol: Some(args.symbol.clone()),
                data,
                truncated: total > args.limit,
                total,
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            formatter::print_refs_grouped(&args.symbol, &groups, total);
        }
    } else {
        // Flat output for functions/methods or filtered queries
        let (refs, total) = graph.find_refs(&args.symbol, kinds_slice, args.limit)?;

        if args.json {
            let output = JsonOutput {
                command: "refs",
                symbol: Some(args.symbol.clone()),
                data: &refs,
                truncated: total > args.limit,
                total,
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            formatter::print_refs(&args.symbol, &refs, total);
        }
    }

    Ok(())
}

/// Find refs to all symbols in a file.
fn run_file_refs(args: &RefsArgs, graph: &Graph) -> Result<()> {
    let file_path = formatter::normalize_path(&args.symbol);
    let kinds: Option<Vec<&str>> = args.kind.as_deref().map(|k| vec![k]);
    let kinds_slice = kinds.as_deref();

    let (refs, total) = graph.find_file_refs(&file_path, kinds_slice, args.limit)?;

    if args.json {
        let output = JsonOutput {
            command: "refs",
            symbol: Some(file_path.clone()),
            data: &refs,
            truncated: total > args.limit,
            total,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        formatter::print_file_refs(&file_path, &refs, total);
    }

    Ok(())
}
