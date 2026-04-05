/// `scope rdeps <symbol>` — show what depends on a symbol (reverse dependencies).
///
/// Critical before any refactor or deletion. Shows all symbols and files
/// that depend on the given symbol. Uses the same transitive impact
/// analysis as `scope callers --depth N`.
use anyhow::{bail, Result};
use clap::Args;
use std::path::Path;

use crate::core::graph::Graph;
use crate::output::formatter;
use crate::output::json::JsonOutput;

/// Arguments for the `scope rdeps` command.
#[derive(Args, Debug)]
pub struct RdepsArgs {
    /// Symbol name to show reverse dependencies for.
    pub symbol: String,

    /// Transitive reverse dependency depth (default: 1, direct only)
    #[arg(long, default_value = "1")]
    pub depth: usize,

    /// Output as JSON instead of human-readable format
    #[arg(long, short = 'j')]
    pub json: bool,
}

use super::looks_like_file_path;

/// Run the `scope rdeps` command.
pub fn run(args: &RdepsArgs, project_root: &Path) -> Result<()> {
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

    let result = if looks_like_file_path(&args.symbol) {
        let file_path = formatter::normalize_path(&args.symbol);
        graph.find_file_impact(&file_path, args.depth)?
    } else {
        graph.find_impact(&args.symbol, args.depth)?
    };

    if args.json {
        let output = JsonOutput {
            command: "rdeps",
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
