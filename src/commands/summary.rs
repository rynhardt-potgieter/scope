/// `scope summary <symbol>` — one-line summary of a symbol.
///
/// Returns a single line with name, kind, location, signature, caller count,
/// and dependency count. Costs ~30 tokens — use when an agent just needs
/// "what is this?" without a full sketch.
///
/// Examples:
///   scope summary PaymentService
///   scope summary Graph.find_symbol
///   scope summary src/core/graph.rs  (summarises the file)
use anyhow::{bail, Result};
use clap::Args;
use std::path::Path;

use crate::core::graph::Graph;
use crate::output::json::JsonOutput;

use super::looks_like_file_path;

/// Arguments for the `scope summary` command.
#[derive(Args, Debug)]
pub struct SummaryArgs {
    /// Symbol name or file path to summarise.
    pub symbol: String,

    /// Output as JSON instead of human-readable format
    #[arg(long, short = 'j')]
    pub json: bool,
}

/// Run the `scope summary` command.
pub fn run(args: &SummaryArgs, project_root: &Path) -> Result<()> {
    let db_path = project_root.join(".scope").join("graph.db");
    if !db_path.exists() {
        bail!("No index found. Run `scope index` first.");
    }

    let graph = Graph::open(&db_path)?;
    crate::commands::warn_if_stale(&graph, project_root);

    if looks_like_file_path(&args.symbol) {
        return run_file_summary(args, &graph);
    }

    let sym = graph
        .find_symbol(&args.symbol)?
        .ok_or_else(|| anyhow::anyhow!("Symbol '{}' not found in index.", args.symbol))?;

    let callers = graph.get_caller_count(&sym.id)?;

    // Count outgoing deps (calls + imports)
    let outgoing = graph.get_outgoing_calls(&sym.id)?;
    let dep_count = outgoing.len();

    // For classes/structs, count methods
    let method_count = if matches!(sym.kind.as_str(), "class" | "struct" | "interface") {
        graph.get_methods(&sym.id)?.len()
    } else {
        0
    };

    if args.json {
        let data = serde_json::json!({
            "name": sym.name,
            "kind": sym.kind,
            "file_path": sym.file_path,
            "line_start": sym.line_start,
            "line_end": sym.line_end,
            "signature": sym.signature,
            "callers": callers,
            "deps": dep_count,
            "methods": method_count,
        });
        let envelope = JsonOutput {
            command: "summary",
            symbol: Some(sym.name.clone()),
            data: &data,
            truncated: false,
            total: 1,
        };
        println!("{}", serde_json::to_string_pretty(&envelope)?);
    } else {
        let sig = sym.signature.as_deref().unwrap_or("");
        let sig_short = sig.lines().next().unwrap_or(sig);
        let lines = sym.line_end - sym.line_start + 1;
        let mut parts = vec![format!(
            "{} ({})  {}:{}–{}  {} lines",
            sym.name, sym.kind, sym.file_path, sym.line_start, sym.line_end, lines,
        )];
        if !sig_short.is_empty() {
            parts.push(format!("  {sig_short}"));
        }
        let mut stats = Vec::new();
        if callers > 0 {
            stats.push(format!("{callers} callers"));
        }
        if dep_count > 0 {
            stats.push(format!("{dep_count} deps"));
        }
        if method_count > 0 {
            stats.push(format!("{method_count} methods"));
        }
        if !stats.is_empty() {
            parts.push(format!("  {}", stats.join(", ")));
        }
        println!("{}", parts.join("\n"));
    }

    Ok(())
}

/// Summarise a file: count of symbols, lines, top-level items.
fn run_file_summary(args: &SummaryArgs, graph: &Graph) -> Result<()> {
    let file_path = crate::output::formatter::normalize_path(&args.symbol);
    let symbols = graph.get_file_symbols(&file_path)?;

    if symbols.is_empty() {
        bail!("No symbols found for file '{}'.", file_path);
    }

    let top_level: Vec<_> = symbols.iter().filter(|s| s.parent_id.is_none()).collect();
    let kinds: Vec<_> = top_level.iter().map(|s| format!("{} {}", s.kind, s.name)).collect();

    if args.json {
        let data = serde_json::json!({
            "file_path": file_path,
            "symbol_count": symbols.len(),
            "top_level": kinds,
        });
        let envelope = JsonOutput {
            command: "summary",
            symbol: Some(file_path.clone()),
            data: &data,
            truncated: false,
            total: symbols.len(),
        };
        println!("{}", serde_json::to_string_pretty(&envelope)?);
    } else {
        println!(
            "{file_path}  {} symbols: {}",
            symbols.len(),
            kinds.join(", "),
        );
    }

    Ok(())
}
