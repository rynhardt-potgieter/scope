/// `scope index` — build or refresh the code index.
///
/// Walks the project's source files, parses them with tree-sitter,
/// and stores symbols and edges in the SQLite graph database.
///
/// By default, runs incrementally: only re-indexes changed files.
/// Use `--full` to force a complete rebuild.
/// Use `--watch` to keep running and re-index automatically on file changes.
use anyhow::{bail, Result};
use chrono::Utc;
use clap::Args;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::config::ProjectConfig;
use crate::core::graph::Graph;
use crate::core::indexer::Indexer;
use crate::core::searcher::Searcher;
use crate::core::watcher::{WatchLock, Watcher};
use crate::output::formatter;

/// Arguments for the `scope index` command.
#[derive(Args, Debug)]
pub struct IndexArgs {
    /// Force a full rebuild of the index (ignore incremental cache)
    #[arg(long)]
    pub full: bool,

    /// Output as JSON instead of human-readable format
    #[arg(long, short = 'j')]
    pub json: bool,

    /// Watch for file changes and re-index automatically.
    ///
    /// Runs an initial index, then watches for file changes and
    /// re-indexes incrementally when source files are modified.
    /// Runs until interrupted with Ctrl+C.
    ///
    /// With --json, emits NDJSON events to stdout:
    ///   {"event":"start",...}   — when watching begins
    ///   {"event":"reindex",...} — after each re-index
    ///   {"event":"stop",...}    — on shutdown
    #[arg(long)]
    pub watch: bool,
}

/// Run the `scope index` command.
pub fn run(args: &IndexArgs, project_root: &Path) -> Result<()> {
    let scope_dir = project_root.join(".scope");

    if !scope_dir.exists() {
        bail!("No .scope/ directory found. Run 'scope init' first.");
    }

    // Load config
    let config = ProjectConfig::load(&scope_dir)?;

    // Open graph database
    let db_path = scope_dir.join("graph.db");
    let mut graph = Graph::open(&db_path)?;

    // Create indexer
    let mut indexer = Indexer::new()?;

    // Open search index (FTS5) — optional, skip with warning if it fails
    let searcher = match Searcher::open(&db_path) {
        Ok(s) => Some(s),
        Err(e) => {
            tracing::warn!("Search index unavailable: {e}");
            None
        }
    };

    if args.watch {
        run_watch(
            args,
            &mut indexer,
            project_root,
            &config,
            &mut graph,
            searcher.as_ref(),
            &scope_dir,
            &db_path,
        )
    } else if args.full {
        run_full_index(
            args,
            &mut indexer,
            project_root,
            &config,
            &mut graph,
            searcher.as_ref(),
        )
    } else {
        run_incremental_index(
            args,
            &mut indexer,
            project_root,
            &config,
            &mut graph,
            searcher.as_ref(),
        )
    }
}

/// Run a full index rebuild.
fn run_full_index(
    args: &IndexArgs,
    indexer: &mut Indexer,
    project_root: &Path,
    config: &ProjectConfig,
    graph: &mut Graph,
    searcher: Option<&Searcher>,
) -> Result<()> {
    let stats = indexer.index_full(project_root, config, graph, searcher)?;

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
    searcher: Option<&Searcher>,
) -> Result<()> {
    let stats = indexer.index_incremental(project_root, config, graph, searcher)?;

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

/// Run watch mode: initial index then watch for changes.
#[allow(clippy::too_many_arguments)]
fn run_watch(
    args: &IndexArgs,
    indexer: &mut Indexer,
    project_root: &Path,
    config: &ProjectConfig,
    graph: &mut Graph,
    searcher: Option<&Searcher>,
    scope_dir: &Path,
    db_path: &Path,
) -> Result<()> {
    // Acquire watch lock
    let lock = WatchLock::new(scope_dir);
    lock.acquire()?;

    // Set up Ctrl+C handler
    let running = Arc::new(AtomicBool::new(true));
    let running_ctrlc = running.clone();
    ctrlc::set_handler(move || {
        running_ctrlc.store(false, Ordering::SeqCst);
    })
    .map_err(|e| anyhow::anyhow!("Failed to set Ctrl+C handler: {e}"))?;

    // Run initial index
    if args.full {
        let stats = indexer.index_full(project_root, config, graph, searcher)?;
        if !args.json {
            for ls in &stats.language_stats {
                eprintln!(
                    "  {:<12} {} files  {} symbols",
                    ls.language, ls.file_count, ls.symbol_count
                );
            }
            eprintln!(
                "Initial index: {:.1}s. {} symbols, {} edges.",
                stats.duration.as_secs_f64(),
                stats.symbol_count,
                stats.edge_count
            );
        }
    } else {
        let stats = indexer.index_incremental(project_root, config, graph, searcher)?;
        if !args.json {
            if stats.up_to_date {
                eprintln!("Index up to date.");
            } else {
                let total = stats.modified.len() + stats.added.len() + stats.deleted.len();
                eprintln!(
                    "Initial index: {:.1}s. {} files changed.",
                    stats.duration.as_secs_f64(),
                    total
                );
            }
        }
    }

    // Build supported extensions list from config languages
    let supported_extensions = get_supported_extensions(config);

    // Emit start event
    let watch_start = Instant::now();
    if args.json {
        let start_event = serde_json::json!({
            "event": "start",
            "project": config.project.name,
            "languages": config.project.languages,
            "timestamp": Utc::now().to_rfc3339(),
        });
        println!("{}", serde_json::to_string(&start_event)?);
    } else {
        eprintln!("Watching for changes... (Ctrl+C to stop)");
    }

    // Create watcher
    let watcher = Watcher::new(
        project_root.to_path_buf(),
        config.index.ignore.clone(),
        supported_extensions,
        Duration::from_millis(300),
    );

    let (rx, _debouncer) = watcher.start()?;

    let mut total_reindexes: u64 = 0;
    let mut total_files_processed: u64 = 0;

    // Event loop
    while running.load(Ordering::SeqCst) {
        // Use a short timeout so we can check the running flag periodically
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(changed_paths) => {
                let batch_start = Instant::now();
                let files_changed = changed_paths.len();

                // Re-open graph for each batch to avoid stale connections
                *graph = match Graph::open(db_path) {
                    Ok(g) => g,
                    Err(e) => {
                        eprintln!(
                            "Warning: failed to open graph for re-index: {e}. Skipping batch."
                        );
                        continue;
                    }
                };

                // Re-open searcher for each batch
                let batch_searcher = match Searcher::open(db_path) {
                    Ok(s) => Some(s),
                    Err(e) => {
                        tracing::warn!("Search index unavailable for re-index: {e}");
                        None
                    }
                };

                // Get symbol/edge counts before re-index for delta calculation
                let symbols_before = graph.symbol_count().unwrap_or(0);
                let edges_before = graph.edge_count().unwrap_or(0);

                // Run incremental index — skip batch on transient failure
                let stats = match indexer.index_incremental(
                    project_root,
                    config,
                    graph,
                    batch_searcher.as_ref(),
                ) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Warning: re-index failed: {e}. Skipping batch.");
                        continue;
                    }
                };

                let duration_ms = batch_start.elapsed().as_millis() as u64;

                // Compute deltas
                let symbols_after = stats.symbol_count;
                let edges_after = stats.edge_count;
                let symbols_added = symbols_after.saturating_sub(symbols_before);
                let symbols_removed = symbols_before.saturating_sub(symbols_after);
                let edges_added = edges_after.saturating_sub(edges_before);
                let edges_removed = edges_before.saturating_sub(edges_after);

                total_reindexes += 1;
                total_files_processed += files_changed as u64;

                if args.json {
                    let reindex_event = serde_json::json!({
                        "event": "reindex",
                        "files_changed": files_changed,
                        "symbols_added": symbols_added,
                        "symbols_removed": symbols_removed,
                        "edges_added": edges_added,
                        "edges_removed": edges_removed,
                        "duration_ms": duration_ms,
                        "timestamp": Utc::now().to_rfc3339(),
                    });
                    println!("{}", serde_json::to_string(&reindex_event)?);
                } else {
                    let total_changed =
                        stats.modified.len() + stats.added.len() + stats.deleted.len();
                    if total_changed > 0 {
                        eprintln!(
                            "Re-indexed {} file{} in {}ms ({} symbols, {} edges)",
                            total_changed,
                            if total_changed == 1 { "" } else { "s" },
                            duration_ms,
                            symbols_after,
                            edges_after
                        );
                    }
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // No events — just loop and check running flag
                continue;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                // Watcher thread died
                tracing::warn!("File watcher disconnected unexpectedly");
                break;
            }
        }
    }

    // Print shutdown summary
    let uptime_secs = watch_start.elapsed().as_secs();

    if args.json {
        let stop_event = serde_json::json!({
            "event": "stop",
            "total_reindexes": total_reindexes,
            "total_files_processed": total_files_processed,
            "uptime_seconds": uptime_secs,
            "timestamp": Utc::now().to_rfc3339(),
        });
        println!("{}", serde_json::to_string(&stop_event)?);
    } else {
        eprintln!(
            "Stopped. {} re-index{}, {} files processed, uptime {}s.",
            total_reindexes,
            if total_reindexes == 1 { "" } else { "es" },
            total_files_processed,
            uptime_secs
        );
    }

    // Lock is released by Drop
    Ok(())
}

/// Get supported file extensions from the project config languages.
fn get_supported_extensions(config: &ProjectConfig) -> Vec<String> {
    let mut extensions = Vec::new();
    for lang in &config.project.languages {
        match lang.to_lowercase().as_str() {
            "typescript" => {
                extensions.push("ts".to_string());
                extensions.push("tsx".to_string());
            }
            "csharp" | "c#" => {
                extensions.push("cs".to_string());
            }
            "python" => {
                extensions.push("py".to_string());
            }
            "go" => {
                extensions.push("go".to_string());
            }
            "java" => {
                extensions.push("java".to_string());
            }
            "rust" => {
                extensions.push("rs".to_string());
            }
            _ => {
                tracing::warn!("Unknown language for watch mode: {lang}");
            }
        }
    }
    extensions
}
