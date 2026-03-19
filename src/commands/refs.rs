/// `scope refs <symbol>` — find all references to a symbol.
///
/// Returns all call sites, imports, type annotations, and other references
/// across the codebase. Use before changing a function signature to find all callers.
///
/// For class symbols, references are grouped by kind (instantiated, extended,
/// imported, used as type). For functions/methods, a flat list is shown.
///
/// Examples:
///   scope refs processPayment              — all references to a function
///   scope refs PaymentService              — grouped references to a class
///   scope refs PaymentService --kind calls — only call sites
///   scope refs src/payments/service.ts     — all refs to symbols in a file
use anyhow::{bail, Result};
use clap::Args;
use std::collections::HashMap;
use std::path::Path;

use crate::core::graph::{Graph, Reference};
use crate::output::formatter;
use crate::output::json::JsonOutput;

/// Arguments for the `scope refs` command.
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

    /// Lines of surrounding code context to show per reference (default: 0)
    #[arg(long, short = 'c', default_value = "0")]
    pub context: usize,

    /// Output as JSON instead of human-readable format
    #[arg(long, short = 'j')]
    pub json: bool,
}

/// Arguments for the `scope callers` command (alias for refs --kind calls).
#[derive(Args, Debug)]
pub struct CallersArgs {
    /// Symbol name to find callers for
    pub symbol: String,

    /// Maximum callers to show (default: 20)
    #[arg(long, default_value = "20")]
    pub limit: usize,

    /// Lines of surrounding code context per caller (default: 0)
    #[arg(long, short = 'c', default_value = "0")]
    pub context: usize,

    /// Output as JSON instead of human-readable format
    #[arg(long, short = 'j')]
    pub json: bool,
}

/// Run the `scope callers` command (shorthand for refs --kind calls).
pub fn run_callers(args: &CallersArgs, project_root: &Path) -> Result<()> {
    let refs_args = RefsArgs {
        symbol: args.symbol.clone(),
        kind: Some("calls".to_string()),
        limit: args.limit,
        context: args.context,
        json: args.json,
    };
    run(&refs_args, project_root)
}

/// Enrich references with source line snippets from the actual files.
///
/// Groups refs by file path to avoid reading the same file multiple times.
/// Sets `snippet_line` to the source line at the reference location (always).
/// Sets `snippet` to surrounding context lines (only when `context_lines > 0`).
/// Gracefully degrades: if a file cannot be read, leaves fields as `None`.
fn enrich_refs_with_snippets(refs: &mut [Reference], project_root: &Path, context_lines: usize) {
    // Group ref indices by file_path
    let mut by_file: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, r) in refs.iter().enumerate() {
        by_file.entry(r.file_path.clone()).or_default().push(i);
    }

    for (file_path, indices) in &by_file {
        let full_path = project_root.join(file_path);
        let lines = match std::fs::read_to_string(&full_path) {
            Ok(content) => content.lines().map(String::from).collect::<Vec<_>>(),
            Err(_) => continue, // graceful degradation
        };

        for &idx in indices {
            let r = &mut refs[idx];
            let Some(line_num) = r.line else { continue };
            let line_idx = (line_num as usize).saturating_sub(1);
            if line_idx >= lines.len() {
                continue;
            }

            // Always set snippet_line to the actual source line
            r.snippet_line = Some(lines[line_idx].trim_end().to_string());

            // Set multi-line context if requested
            if context_lines > 0 {
                let start = line_idx.saturating_sub(context_lines);
                let end = (line_idx + context_lines + 1).min(lines.len());
                let ctx: Vec<String> = lines[start..end]
                    .iter()
                    .map(|l| l.trim_end().to_string())
                    .collect();
                r.snippet = Some(ctx);
            }
        }
    }
}

use super::looks_like_file_path;

/// Run the `scope refs` command.
pub fn run(args: &RefsArgs, project_root: &Path) -> Result<()> {
    let scope_dir = project_root.join(".scope");

    if !scope_dir.exists() {
        bail!("No .scope/ directory found. Run 'scope init' first.");
    }

    let db_path = scope_dir.join("graph.db");
    if !db_path.exists() {
        bail!("No index found. Run 'scope index' to build one first.");
    }

    let graph = Graph::open(&db_path)?;

    if looks_like_file_path(&args.symbol) {
        return run_file_refs(args, &graph, project_root);
    }

    run_symbol_refs(args, &graph, project_root)
}

/// Find refs for a single symbol.
fn run_symbol_refs(args: &RefsArgs, graph: &Graph, project_root: &Path) -> Result<()> {
    let kinds: Option<Vec<&str>> = args.kind.as_deref().map(|k| vec![k]);
    let kinds_slice = kinds.as_deref();

    // Check if this is a class-like symbol for grouped output
    let is_class = graph.is_class_like(&args.symbol)?;

    if is_class && kinds_slice.is_none() {
        // Grouped output for class symbols
        let (mut groups, total) = graph.find_refs_grouped(&args.symbol, args.limit)?;

        // Enrich all refs in all groups with source snippets
        for (_kind, refs) in &mut groups {
            enrich_refs_with_snippets(refs, project_root, args.context);
        }

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
        let (mut refs, total) = graph.find_refs(&args.symbol, kinds_slice, args.limit)?;

        // Enrich refs with source snippets
        enrich_refs_with_snippets(&mut refs, project_root, args.context);

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
fn run_file_refs(args: &RefsArgs, graph: &Graph, project_root: &Path) -> Result<()> {
    let file_path = formatter::normalize_path(&args.symbol);
    let kinds: Option<Vec<&str>> = args.kind.as_deref().map(|k| vec![k]);
    let kinds_slice = kinds.as_deref();

    let (mut refs, total) = graph.find_file_refs(&file_path, kinds_slice, args.limit)?;

    // Enrich refs with source snippets
    enrich_refs_with_snippets(&mut refs, project_root, args.context);

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
