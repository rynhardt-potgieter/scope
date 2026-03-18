/// `sc index` — build or refresh the code index.
///
/// Walks the project's source files, parses them with tree-sitter,
/// and stores symbols and edges in the SQLite graph database.
use anyhow::{bail, Result};
use clap::Args;
use std::path::Path;

use crate::config::ProjectConfig;
use crate::core::graph::Graph;
use crate::core::indexer::Indexer;

/// Arguments for the `sc index` command.
#[derive(Args, Debug)]
pub struct IndexArgs {
    /// Force a full rebuild of the index (ignore incremental cache)
    #[arg(long)]
    pub full: bool,

    /// Output as JSON instead of human-readable format
    #[arg(long, short = 'j')]
    pub json: bool,
}

/// Run the `sc index` command.
pub fn run(args: &IndexArgs, project_root: &Path) -> Result<()> {
    let scope_dir = project_root.join(".scope");

    if !scope_dir.exists() {
        bail!("No .scope/ directory found. Run 'sc init' first.");
    }

    // Load config
    let config = ProjectConfig::load(&scope_dir)?;

    // Open graph database
    let db_path = scope_dir.join("graph.db");
    let mut graph = Graph::open(&db_path)?;

    if !args.full {
        eprintln!("Note: incremental indexing not yet available, performing full index.");
    }

    // Create indexer and run
    let mut indexer = Indexer::new()?;
    let stats = indexer.index_full(project_root, &config, &mut graph)?;

    if args.json {
        let output = serde_json::json!({
            "command": "index",
            "file_count": stats.file_count,
            "symbol_count": stats.symbol_count,
            "edge_count": stats.edge_count,
            "duration_secs": stats.duration.as_secs_f64(),
            "languages": stats.language_stats.iter().map(|ls| {
                serde_json::json!({
                    "language": ls.language,
                    "file_count": ls.file_count,
                    "symbol_count": ls.symbol_count,
                })
            }).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        for ls in &stats.language_stats {
            eprintln!(
                "  {:<12} {} files  {} symbols",
                ls.language, ls.file_count, ls.symbol_count
            );
        }
        eprintln!(
            "Built in {:.1}s. {} symbols, {} edges.",
            stats.duration.as_secs_f64(),
            stats.symbol_count,
            stats.edge_count
        );
    }

    Ok(())
}
