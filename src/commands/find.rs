/// `scope find <query>` — find code by intent using full-text search.
///
/// Searches the symbol index for symbols matching a natural-language query.
/// Uses FTS5 with BM25 ranking to return the most relevant results.
/// Returns ranked results with similarity scores.
///
/// In workspace mode (`--workspace`), performs sequential FTS5 queries
/// per member and merges results by score.
///
/// Examples:
///   scope find "handles authentication errors"
///   scope find "payment processing" --kind method
///   scope find "validates user input" --limit 5 --json
///   scope find "payment" --workspace
use anyhow::{bail, Result};
use clap::Args;
use serde::Serialize;
use std::path::Path;

use crate::config::workspace::WorkspaceConfig;
use crate::core::searcher::{SearchResult, Searcher};
use crate::output::formatter;
use crate::output::json::JsonOutput;
use crate::Context;

/// Arguments for the `scope find` command.
#[derive(Args, Debug)]
pub struct FindArgs {
    /// Natural language search query.
    ///
    /// Searches symbol names, signatures, and docstrings.
    /// Examples: "handles authentication errors", "sends email notifications"
    pub query: String,

    /// Filter by symbol kind: function, class, method, interface
    #[arg(long)]
    pub kind: Option<String>,

    /// Filter by language: typescript, csharp, python
    #[arg(long)]
    pub lang: Option<String>,

    /// Maximum number of results to show
    #[arg(long, default_value = "10")]
    pub limit: usize,

    /// Output as JSON instead of human-readable format
    #[arg(long, short = 'j')]
    pub json: bool,
}

/// A search result tagged with its source project (workspace mode).
#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceSearchResult {
    /// The workspace member name.
    pub project: String,
    /// The underlying search result.
    #[serde(flatten)]
    pub result: SearchResult,
}

/// Run the `scope find` command.
pub fn run(args: &FindArgs, ctx: &Context) -> Result<()> {
    match ctx {
        Context::SingleProject { root } => run_single(args, root),
        Context::Workspace {
            workspace_root,
            config,
            ..
        } => run_workspace(args, workspace_root, config),
    }
}

/// Run find for a single project.
fn run_single(args: &FindArgs, project_root: &Path) -> Result<()> {
    let scope_dir = project_root.join(".scope");

    if !scope_dir.exists() {
        bail!("No .scope/ directory found. Run 'scope init' first.");
    }

    let db_path = scope_dir.join("graph.db");
    if !db_path.exists() {
        bail!("No index found. Run 'scope index' to build one first.");
    }

    let searcher = Searcher::open(&db_path)?;

    let results = searcher.search(&args.query, args.limit, args.kind.as_deref())?;

    if args.json {
        let total = results.len();
        let output = JsonOutput {
            command: "find",
            symbol: None,
            data: &results,
            truncated: false,
            total,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        formatter::print_find_results(&args.query, &results);
    }

    Ok(())
}

/// Run find across all workspace members (sequential FTS5 per member).
fn run_workspace(args: &FindArgs, workspace_root: &Path, config: &WorkspaceConfig) -> Result<()> {
    let mut all_results: Vec<WorkspaceSearchResult> = Vec::new();

    for entry in &config.workspace.members {
        let name = WorkspaceConfig::resolve_member_name(entry);
        let member_path = workspace_root.join(&entry.path);
        let db_path = member_path.join(".scope").join("graph.db");

        if !db_path.exists() {
            continue;
        }

        let searcher = match Searcher::open(&db_path) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("Failed to open searcher for '{}': {}", name, e);
                continue;
            }
        };

        match searcher.search(&args.query, args.limit, args.kind.as_deref()) {
            Ok(results) => {
                for r in results {
                    all_results.push(WorkspaceSearchResult {
                        project: name.clone(),
                        result: r,
                    });
                }
            }
            Err(e) => {
                tracing::warn!("Search error in '{}': {}", name, e);
            }
        }
    }

    // Sort by score descending, truncate to limit
    all_results.sort_by(|a, b| {
        b.result
            .score
            .partial_cmp(&a.result.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    all_results.truncate(args.limit);

    let total = all_results.len();

    if args.json {
        let output = JsonOutput {
            command: "find",
            symbol: None,
            data: &all_results,
            truncated: false,
            total,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        formatter::print_workspace_find_results(&args.query, &all_results);
    }

    Ok(())
}
