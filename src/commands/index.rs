/// `sc index` — build or refresh the code index.
///
/// Walks the project's source files, parses them with tree-sitter,
/// and stores symbols and edges in the SQLite graph database.
///
/// By default, runs incrementally: only re-indexes changed files.
/// Use `--full` to force a complete rebuild.
use anyhow::{bail, Result};
use clap::Args;
use std::path::Path;

use crate::config::ProjectConfig;
use crate::core::graph::Graph;
use crate::core::indexer::Indexer;
use crate::output::formatter;

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

    // Create indexer
    let mut indexer = Indexer::new()?;

    if args.full {
        run_full_index(args, &mut indexer, project_root, &config, &mut graph)
    } else {
        run_incremental_index(args, &mut indexer, project_root, &config, &mut graph)
    }
}

/// Run a full index rebuild.
fn run_full_index(
    args: &IndexArgs,
    indexer: &mut Indexer,
    project_root: &Path,
    config: &ProjectConfig,
    graph: &mut Graph,
) -> Result<()> {
    let stats = indexer.index_full(project_root, config, graph)?;

    if args.json {
        let output = serde_json::json!({
            "command": "index",
            "mode": "full",
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

/// Run an incremental index (default).
fn run_incremental_index(
    args: &IndexArgs,
    indexer: &mut Indexer,
    project_root: &Path,
    config: &ProjectConfig,
    graph: &mut Graph,
) -> Result<()> {
    let stats = indexer.index_incremental(project_root, config, graph)?;

    if stats.up_to_date {
        if args.json {
            let output = serde_json::json!({
                "command": "index",
                "mode": "incremental",
                "up_to_date": true,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            eprintln!("Index up to date.");
        }
        return Ok(());
    }

    if args.json {
        let output = serde_json::json!({
            "command": "index",
            "mode": "incremental",
            "up_to_date": false,
            "modified": stats.modified,
            "added": stats.added,
            "deleted": stats.deleted,
            "symbol_count": stats.symbol_count,
            "edge_count": stats.edge_count,
            "duration_secs": stats.duration.as_secs_f64(),
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        formatter::print_incremental_result(
            &stats.modified,
            &stats.added,
            &stats.deleted,
            stats.duration.as_secs_f64(),
        );
    }

    Ok(())
}
