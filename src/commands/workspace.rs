/// `scope workspace` — manage multi-project workspaces.
///
/// Subcommands:
///   init  — discover projects and create scope-workspace.toml
///   list  — show workspace members and their index status
///   index — index all workspace members sequentially
///
/// Examples:
///   scope workspace init                — discover and create manifest
///   scope workspace init --name my-ws   — set workspace name
///   scope workspace list                — show all members
///   scope workspace list --json         — machine-readable output
///   scope workspace index               — incremental index all members
///   scope workspace index --full        — full re-index all members
use anyhow::{bail, Result};
use clap::{Args, Subcommand};
use serde::Serialize;
use std::path::Path;
use std::time::Instant;

use crate::config::workspace::WorkspaceConfig;
use crate::config::ProjectConfig;
use crate::core::graph::Graph;
use crate::core::indexer::Indexer;
use crate::core::searcher::Searcher;
use crate::output::formatter;
use crate::output::json::JsonOutput;

/// Manage multi-project workspaces.
///
/// A workspace groups multiple Scope projects (each with its own .scope/
/// directory) and enables federated queries across all members.
///
/// Use `scope workspace init` to create a workspace manifest by discovering
/// existing Scope projects in subdirectories. Use `scope workspace list`
/// to check the health of all members.
#[derive(Args, Debug)]
pub struct WorkspaceArgs {
    #[command(subcommand)]
    pub command: WorkspaceCommands,
}

/// Workspace subcommands.
#[derive(Subcommand, Debug)]
pub enum WorkspaceCommands {
    /// Discover projects in the current directory tree and create scope-workspace.toml.
    ///
    /// Walks subdirectories (max depth 3) looking for .scope/config.toml markers.
    /// Each discovered project becomes a [[workspace.members]] entry.
    ///
    /// If projects have not been initialised yet, run `scope init` in each
    /// project first, then run `scope workspace init` from the parent directory.
    ///
    /// Examples:
    ///   scope workspace init                    — discover and create manifest
    ///   scope workspace init --name my-workspace  — set workspace name
    Init(WorkspaceInitArgs),

    /// Show all workspace members and their index status.
    ///
    /// Reads scope-workspace.toml and checks each member for .scope/graph.db
    /// existence, symbol count, and last indexed time. Use this to verify the
    /// workspace is healthy before querying.
    ///
    /// Output columns: name, path, status, files, symbols, last indexed.
    ///
    /// Examples:
    ///   scope workspace list
    ///   scope workspace list --json
    List(WorkspaceListArgs),

    /// Index all workspace members.
    ///
    /// Reads scope-workspace.toml and indexes each member project sequentially.
    /// Members without .scope/config.toml are skipped with a warning.
    ///
    /// Examples:
    ///   scope workspace index              — incremental index all members
    ///   scope workspace index --full       — full re-index all members
    ///   scope workspace index --json       — machine-readable output
    Index(WorkspaceIndexArgs),
}

/// Arguments for `scope workspace init`.
#[derive(Args, Debug)]
pub struct WorkspaceInitArgs {
    /// Workspace name. Defaults to the current directory name.
    #[arg(long)]
    pub name: Option<String>,
}

/// Arguments for `scope workspace list`.
#[derive(Args, Debug)]
pub struct WorkspaceListArgs {
    /// Output as JSON instead of human-readable format.
    #[arg(long, short = 'j')]
    pub json: bool,
}

/// Arguments for `scope workspace index`.
#[derive(Args, Debug)]
pub struct WorkspaceIndexArgs {
    /// Rebuild the entire index from scratch for all members.
    #[arg(long)]
    pub full: bool,

    /// Output as JSON instead of human-readable format.
    #[arg(long, short = 'j')]
    pub json: bool,
}

/// Status information for a single workspace member.
#[derive(Debug, Serialize)]
pub struct MemberStatus {
    /// Member display name.
    pub name: String,
    /// Path relative to workspace root.
    pub path: String,
    /// Index status: "indexed", "not initialised", or "not indexed".
    pub status: String,
    /// Number of indexed files (0 if not indexed).
    pub file_count: usize,
    /// Number of symbols in the index (0 if not indexed).
    pub symbol_count: usize,
    /// Unix timestamp of last indexing, if available.
    pub last_indexed_at: Option<i64>,
}

/// JSON data payload for `scope workspace list`.
#[derive(Debug, Serialize)]
pub struct WorkspaceListData {
    /// Workspace name from the manifest.
    pub workspace_name: String,
    /// Status of each member.
    pub members: Vec<MemberStatus>,
}

/// Result of indexing a single workspace member.
#[derive(Debug, Serialize)]
pub struct MemberIndexResult {
    /// Member display name.
    pub name: String,
    /// Path relative to workspace root.
    pub path: String,
    /// Whether the member was indexed successfully.
    pub status: String,
    /// Indexing mode: "full", "incremental", or "skipped".
    pub mode: String,
    /// Number of symbols after indexing.
    pub symbol_count: usize,
    /// Number of edges after indexing.
    pub edge_count: usize,
    /// Duration in seconds.
    pub duration_secs: f64,
}

/// JSON data payload for `scope workspace index`.
#[derive(Debug, Serialize)]
pub struct WorkspaceIndexData {
    /// Workspace name from the manifest.
    pub workspace_name: String,
    /// Per-member indexing results.
    pub members: Vec<MemberIndexResult>,
    /// Total symbols across all indexed members.
    pub total_symbols: usize,
    /// Total edges across all indexed members.
    pub total_edges: usize,
    /// Total duration in seconds.
    pub total_duration_secs: f64,
}

/// Run the `scope workspace` command.
pub fn run(args: &WorkspaceArgs, project_root: &Path) -> Result<()> {
    match &args.command {
        WorkspaceCommands::Init(init_args) => run_init(init_args, project_root),
        WorkspaceCommands::List(list_args) => run_list(list_args, project_root),
        WorkspaceCommands::Index(index_args) => run_index(index_args, project_root),
    }
}

/// Run `scope workspace init` — discover projects and write scope-workspace.toml.
fn run_init(args: &WorkspaceInitArgs, project_root: &Path) -> Result<()> {
    let manifest_path = project_root.join("scope-workspace.toml");

    if manifest_path.exists() {
        bail!("Workspace already initialized. Edit scope-workspace.toml directly.");
    }

    // Discover projects by walking subdirectories (max depth 3)
    let mut members: Vec<(String, String)> = Vec::new();

    discover_projects(project_root, project_root, 0, 3, &mut members)?;

    if members.is_empty() {
        bail!(
            "No Scope projects found in subdirectories.\n\
             Run 'scope init' in each project directory first, then retry."
        );
    }

    // Sort members by path for deterministic output
    members.sort_by(|a, b| a.0.cmp(&b.0));

    // Determine workspace name
    let ws_name = args.name.clone().unwrap_or_else(|| {
        project_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("workspace")
            .to_string()
    });

    let toml_content = WorkspaceConfig::generate_toml(&ws_name, &members);
    std::fs::write(&manifest_path, toml_content)?;

    // Report to stderr (progress/info goes to stderr)
    let member_names: Vec<&str> = members.iter().map(|(_, name)| name.as_str()).collect();
    eprintln!(
        "Found {} projects: {}",
        members.len(),
        member_names.join(", ")
    );
    eprintln!("Created scope-workspace.toml");

    Ok(())
}

/// Recursively discover Scope projects by looking for `.scope/config.toml`.
fn discover_projects(
    base_root: &Path,
    current: &Path,
    depth: usize,
    max_depth: usize,
    members: &mut Vec<(String, String)>,
) -> Result<()> {
    if depth > max_depth {
        return Ok(());
    }

    let entries = match std::fs::read_dir(current) {
        Ok(entries) => entries,
        Err(e) => {
            tracing::warn!(
                "Cannot read directory {}: {e}. Skipping.",
                current.display()
            );
            return Ok(());
        }
    };

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        // Skip hidden directories and common non-project dirs
        let dir_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        if dir_name.starts_with('.')
            || dir_name == "node_modules"
            || dir_name == "target"
            || dir_name == "dist"
            || dir_name == "build"
        {
            continue;
        }

        // Check if this directory has .scope/config.toml
        let scope_config = path.join(".scope").join("config.toml");
        if scope_config.exists() {
            // Compute relative path from workspace root
            let rel_path = path
                .strip_prefix(base_root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");

            let name = dir_name;
            members.push((rel_path, name));

            // Don't recurse into discovered projects (they're self-contained)
            continue;
        }

        // Recurse into subdirectories
        discover_projects(base_root, &path, depth + 1, max_depth, members)?;
    }

    Ok(())
}

/// Run `scope workspace list` — show workspace members and their status.
fn run_list(args: &WorkspaceListArgs, project_root: &Path) -> Result<()> {
    // Find scope-workspace.toml (in CWD or walk upward)
    let manifest_path = find_workspace_manifest(project_root)?;
    let workspace_root = manifest_path.parent().unwrap_or(project_root);

    let config = WorkspaceConfig::load(&manifest_path)?;

    let mut member_statuses: Vec<MemberStatus> = Vec::new();

    for entry in &config.workspace.members {
        let name = WorkspaceConfig::resolve_member_name(entry);
        let member_path = workspace_root.join(&entry.path);

        let scope_dir = member_path.join(".scope");
        let db_path = scope_dir.join("graph.db");

        let status = if !scope_dir.exists() {
            MemberStatus {
                name,
                path: entry.path.clone(),
                status: "not initialised".to_string(),
                file_count: 0,
                symbol_count: 0,
                last_indexed_at: None,
            }
        } else if !db_path.exists() {
            MemberStatus {
                name,
                path: entry.path.clone(),
                status: "not indexed".to_string(),
                file_count: 0,
                symbol_count: 0,
                last_indexed_at: None,
            }
        } else {
            match Graph::open(&db_path) {
                Ok(graph) => {
                    let symbol_count = graph.symbol_count().unwrap_or(0);
                    let file_count = graph.file_count().unwrap_or(0);
                    let last_indexed_at = graph.last_indexed_at().unwrap_or(None);

                    MemberStatus {
                        name,
                        path: entry.path.clone(),
                        status: "indexed".to_string(),
                        file_count,
                        symbol_count,
                        last_indexed_at,
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to open graph for member '{}': {e}", name);
                    MemberStatus {
                        name,
                        path: entry.path.clone(),
                        status: format!("error: {e}"),
                        file_count: 0,
                        symbol_count: 0,
                        last_indexed_at: None,
                    }
                }
            }
        };

        member_statuses.push(status);
    }

    if args.json {
        let data = WorkspaceListData {
            workspace_name: config.workspace.name.clone(),
            members: member_statuses,
        };
        let output = JsonOutput {
            command: "workspace list",
            symbol: None,
            data,
            truncated: false,
            total: config.workspace.members.len(),
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        formatter::print_workspace_list(&config.workspace.name, &member_statuses);
    }

    Ok(())
}

/// Run `scope workspace index` — index all workspace members sequentially.
fn run_index(args: &WorkspaceIndexArgs, project_root: &Path) -> Result<()> {
    let manifest_path = find_workspace_manifest(project_root)?;
    let workspace_root = manifest_path.parent().unwrap_or(project_root);

    let config = WorkspaceConfig::load(&manifest_path)?;
    let total_members = config.workspace.members.len();
    let overall_start = Instant::now();

    let mut results: Vec<MemberIndexResult> = Vec::new();
    let mut indexed_count: usize = 0;
    let mut total_symbols: usize = 0;
    let mut total_edges: usize = 0;

    for entry in &config.workspace.members {
        let name = WorkspaceConfig::resolve_member_name(entry);
        let member_path = workspace_root.join(&entry.path);
        let scope_dir = member_path.join(".scope");
        let config_path = scope_dir.join("config.toml");

        // Skip members without .scope/config.toml
        if !config_path.exists() {
            eprintln!("[{}] Skipped: no .scope/config.toml found", name);
            results.push(MemberIndexResult {
                name,
                path: entry.path.clone(),
                status: "skipped".to_string(),
                mode: "skipped".to_string(),
                symbol_count: 0,
                edge_count: 0,
                duration_secs: 0.0,
            });
            continue;
        }

        let member_start = Instant::now();

        // Load project config
        let project_config = match ProjectConfig::load(&scope_dir) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[{}] Error loading config: {}", name, e);
                results.push(MemberIndexResult {
                    name,
                    path: entry.path.clone(),
                    status: "error".to_string(),
                    mode: if args.full { "full" } else { "incremental" }.to_string(),
                    symbol_count: 0,
                    edge_count: 0,
                    duration_secs: member_start.elapsed().as_secs_f64(),
                });
                continue;
            }
        };

        // Open graph database
        let db_path = scope_dir.join("graph.db");
        let mut graph = match Graph::open(&db_path) {
            Ok(g) => g,
            Err(e) => {
                eprintln!("[{}] Error opening graph: {}", name, e);
                results.push(MemberIndexResult {
                    name,
                    path: entry.path.clone(),
                    status: "error".to_string(),
                    mode: if args.full { "full" } else { "incremental" }.to_string(),
                    symbol_count: 0,
                    edge_count: 0,
                    duration_secs: member_start.elapsed().as_secs_f64(),
                });
                continue;
            }
        };

        // Create indexer
        let mut indexer = match Indexer::new() {
            Ok(i) => i,
            Err(e) => {
                eprintln!("[{}] Error creating indexer: {}", name, e);
                results.push(MemberIndexResult {
                    name,
                    path: entry.path.clone(),
                    status: "error".to_string(),
                    mode: if args.full { "full" } else { "incremental" }.to_string(),
                    symbol_count: 0,
                    edge_count: 0,
                    duration_secs: member_start.elapsed().as_secs_f64(),
                });
                continue;
            }
        };

        // Open search index (optional)
        let searcher = match Searcher::open(&db_path) {
            Ok(s) => Some(s),
            Err(e) => {
                tracing::warn!("[{}] Search index unavailable: {e}", name);
                None
            }
        };

        // Run indexing
        let mode = if args.full { "full" } else { "incremental" };
        let (symbol_count, edge_count) = if args.full {
            match indexer.index_full(&member_path, &project_config, &mut graph, searcher.as_ref()) {
                Ok(stats) => (stats.symbol_count, stats.edge_count),
                Err(e) => {
                    eprintln!("[{}] Error during indexing: {}", name, e);
                    results.push(MemberIndexResult {
                        name,
                        path: entry.path.clone(),
                        status: "error".to_string(),
                        mode: mode.to_string(),
                        symbol_count: 0,
                        edge_count: 0,
                        duration_secs: member_start.elapsed().as_secs_f64(),
                    });
                    continue;
                }
            }
        } else {
            match indexer.index_incremental(
                &member_path,
                &project_config,
                &mut graph,
                searcher.as_ref(),
            ) {
                Ok(stats) => (stats.symbol_count, stats.edge_count),
                Err(e) => {
                    eprintln!("[{}] Error during indexing: {}", name, e);
                    results.push(MemberIndexResult {
                        name,
                        path: entry.path.clone(),
                        status: "error".to_string(),
                        mode: mode.to_string(),
                        symbol_count: 0,
                        edge_count: 0,
                        duration_secs: member_start.elapsed().as_secs_f64(),
                    });
                    continue;
                }
            }
        };

        let duration = member_start.elapsed();
        total_symbols += symbol_count;
        total_edges += edge_count;
        indexed_count += 1;

        if !args.json {
            eprintln!(
                "[{}] Indexed: {} symbols, {} edges ({:.1}s)",
                name,
                symbol_count,
                edge_count,
                duration.as_secs_f64()
            );
        }

        results.push(MemberIndexResult {
            name,
            path: entry.path.clone(),
            status: "indexed".to_string(),
            mode: mode.to_string(),
            symbol_count,
            edge_count,
            duration_secs: duration.as_secs_f64(),
        });
    }

    let total_duration = overall_start.elapsed();

    if args.json {
        let data = WorkspaceIndexData {
            workspace_name: config.workspace.name.clone(),
            members: results,
            total_symbols,
            total_edges,
            total_duration_secs: total_duration.as_secs_f64(),
        };
        let output = JsonOutput {
            command: "workspace index",
            symbol: None,
            data,
            truncated: false,
            total: total_members,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        eprintln!(
            "Workspace indexed: {}/{} members, {} total symbols",
            indexed_count, total_members, total_symbols
        );
    }

    Ok(())
}

/// Walk upward from the given directory looking for `scope-workspace.toml`.
///
/// Returns the path to the manifest file if found.
pub fn find_workspace_manifest(start: &Path) -> Result<std::path::PathBuf> {
    let mut current = start.to_path_buf();

    loop {
        let candidate = current.join("scope-workspace.toml");
        if candidate.exists() {
            return Ok(candidate);
        }

        if !current.pop() {
            break;
        }
    }

    bail!(
        "No scope-workspace.toml found.\n\
         Run 'scope workspace init' to create one."
    );
}
