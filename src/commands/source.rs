/// `scope source <symbol>` — fetch full source of a specific symbol.
///
/// Returns the exact source code of the symbol, including its full definition.
/// Only call this when ready to read or edit the implementation.
use anyhow::{bail, Context, Result};
use clap::Args;
use std::path::Path;

use crate::core::graph::Graph;

/// Arguments for the `scope source` command.
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

/// Run the `scope source` command.
pub fn run(args: &SourceArgs, project_root: &Path) -> Result<()> {
    let db_path = project_root.join(".scope").join("graph.db");
    if !db_path.exists() {
        bail!("No index found. Run `scope index` first.");
    }

    let graph = Graph::open(&db_path)?;
    let sym = graph
        .find_symbol(&args.symbol)?
        .ok_or_else(|| anyhow::anyhow!("Symbol '{}' not found in index.", args.symbol))?;

    let full_path = project_root.join(&sym.file_path);
    let content = std::fs::read_to_string(&full_path)
        .with_context(|| format!("Could not read {}", full_path.display()))?;

    let lines: Vec<&str> = content.lines().collect();
    let start = (sym.line_start as usize).saturating_sub(1);
    let end = (sym.line_end as usize).min(lines.len());

    if start >= lines.len() {
        bail!(
            "Symbol '{}' line range {}-{} is out of bounds for {}",
            args.symbol,
            sym.line_start,
            sym.line_end,
            sym.file_path
        );
    }

    let source_lines = &lines[start..end];

    if args.json {
        let output = serde_json::json!({
            "symbol": sym.name,
            "kind": sym.kind,
            "file_path": sym.file_path,
            "line_start": sym.line_start,
            "line_end": sym.line_end,
            "signature": sym.signature,
            "source": source_lines.join("\n"),
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!(
            "// {}  {}:{}–{}",
            sym.name, sym.file_path, sym.line_start, sym.line_end
        );
        for line in source_lines {
            println!("{}", line);
        }
    }

    Ok(())
}
