/// `scope map` — show a structural overview of the entire repository.
///
/// Displays entry points, core symbols ranked by importance,
/// architecture layers, and key statistics. Designed to give
/// an LLM agent a complete mental model of the codebase in
/// ~500-1000 tokens, replacing multiple `scope sketch` calls.
///
/// In workspace mode (`--workspace`), shows a unified map across all
/// workspace members with per-project stats, entry points, and core symbols.
///
/// Examples:
///   scope map              — full repository map
///   scope map --limit 5    — show top 5 core symbols
///   scope map --json       — JSON output
///   scope map --workspace  — unified workspace map
use anyhow::{bail, Result};
use clap::Args;
use serde::Serialize;
use std::path::Path;

use crate::commands::entrypoints::EntrypointInfo;
use crate::config::workspace::WorkspaceConfig;
use crate::core::graph::Graph;
use crate::core::workspace_graph::WorkspaceGraph;
use crate::output::formatter;
use crate::output::json::JsonOutput;
use crate::Context;

/// Show a structural overview of the entire repository.
///
/// Displays entry points, core symbols ranked by importance,
/// architecture layers, and key statistics. Designed to give
/// an LLM agent a complete mental model of the codebase in
/// ~500-1000 tokens — replacing multiple scope sketch calls.
///
/// Use this at the start of any complex task to understand the
/// repo before diving into specific files.
///
/// Examples:
///   scope map              — full repository map
///   scope map --limit 5    — show top 5 core symbols
///   scope map --json       — JSON output
///   scope map --workspace  — unified workspace map
#[derive(Args, Debug)]
pub struct MapArgs {
    /// Maximum symbols to show in the core symbols section
    #[arg(long, default_value = "10")]
    pub limit: usize,

    /// Output as JSON instead of human-readable format
    #[arg(long, short = 'j')]
    pub json: bool,

    /// Compact JSON output for agents — limits entrypoints to top 10 per
    /// category and omits full method_count/outgoing_call_count detail.
    /// Implies --json.
    #[arg(long)]
    pub compact: bool,
}

/// Statistics for the repository map.
#[derive(Debug, Serialize)]
pub struct MapStats {
    /// Total number of indexed files.
    pub file_count: usize,
    /// Total number of symbols.
    pub symbol_count: usize,
    /// Total number of edges.
    pub edge_count: usize,
    /// Languages found in the index.
    pub languages: Vec<String>,
}

/// A core symbol entry for map output.
#[derive(Debug, Serialize)]
pub struct CoreSymbol {
    /// Symbol name.
    pub name: String,
    /// Symbol kind (function, method).
    pub kind: String,
    /// File path relative to project root.
    pub file_path: String,
    /// Number of incoming callers.
    pub caller_count: usize,
    /// Project name (only set in workspace mode).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

/// A directory statistics entry for map output.
#[derive(Debug, Serialize)]
pub struct DirStats {
    /// Directory name (with trailing slash).
    pub directory: String,
    /// Number of files in this directory.
    pub file_count: usize,
    /// Number of symbols in this directory.
    pub symbol_count: usize,
}

/// Full JSON data payload for the map command.
#[derive(Debug, Serialize)]
pub struct MapData {
    /// Repository statistics.
    pub stats: MapStats,
    /// Entry points grouped by type.
    pub entrypoints: Vec<(String, Vec<EntrypointInfo>)>,
    /// Core symbols by importance.
    pub core_symbols: Vec<CoreSymbol>,
    /// Directory-level architecture.
    pub architecture: Vec<DirStats>,
}

/// Run the `scope map` command.
pub fn run(args: &MapArgs, ctx: &Context) -> Result<()> {
    match ctx {
        Context::SingleProject { root } => run_single(args, root),
        Context::Workspace {
            workspace_root,
            config,
            ..
        } => run_workspace(args, workspace_root, config),
    }
}

/// Run map for a single project.
fn run_single(args: &MapArgs, project_root: &Path) -> Result<()> {
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

    // 1. Gather statistics.
    let stats = MapStats {
        file_count: graph.file_count()?,
        symbol_count: graph.symbol_count()?,
        edge_count: graph.edge_count()?,
        languages: graph.get_languages()?,
    };

    // 2. Get entry points (reuse shared collapse_and_group logic).
    let raw_entrypoints = graph.get_entrypoints()?;
    let (ep_groups, ep_total, _ep_file_count) =
        crate::commands::entrypoints::collapse_and_group(&raw_entrypoints, &graph);

    // 3. Get core symbols by importance.
    let raw_core = graph.get_symbols_by_importance(args.limit)?;
    let core_symbols: Vec<CoreSymbol> = raw_core
        .into_iter()
        .map(|(sym, count)| CoreSymbol {
            name: sym.name,
            kind: sym.kind,
            file_path: sym.file_path,
            caller_count: count,
            project: None,
        })
        .collect();

    // 4. Get directory-level architecture.
    let raw_dirs = graph.get_directory_stats()?;
    let architecture: Vec<DirStats> = raw_dirs
        .into_iter()
        .map(|(dir, files, symbols)| DirStats {
            directory: dir,
            file_count: files,
            symbol_count: symbols,
        })
        .collect();

    // 5. Detect project name from the directory name.
    let project_name = project_root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    if args.json || args.compact {
        // In compact mode, limit entrypoints to top 10 per category to cut token cost.
        let ep_data = if args.compact {
            ep_groups
                .into_iter()
                .map(|(cat, entries)| {
                    let truncated: Vec<_> = entries.into_iter().take(10).collect();
                    (cat, truncated)
                })
                .collect()
        } else {
            ep_groups
        };
        let data = MapData {
            stats,
            entrypoints: ep_data,
            core_symbols,
            architecture,
        };
        let output = JsonOutput {
            command: "map",
            symbol: None,
            data: &data,
            truncated: false,
            total: ep_total,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        formatter::print_map(
            project_name,
            &stats,
            &ep_groups,
            &core_symbols,
            &architecture,
        );
    }

    Ok(())
}

/// Run map across all workspace members.
fn run_workspace(args: &MapArgs, workspace_root: &Path, config: &WorkspaceConfig) -> Result<()> {
    let members: Vec<(String, std::path::PathBuf)> = config
        .workspace
        .members
        .iter()
        .map(|entry| {
            let name = WorkspaceConfig::resolve_member_name(entry);
            let path = workspace_root.join(&entry.path);
            (name, path)
        })
        .collect();

    let wg = WorkspaceGraph::open(members)?;

    // 1. Aggregate statistics.
    let stats = MapStats {
        file_count: wg.file_count(),
        symbol_count: wg.symbol_count(),
        edge_count: wg.edge_count(),
        languages: wg.get_languages(),
    };

    // 2. Get entry points per project.
    let ws_entrypoints = wg.get_entrypoints();

    // Flatten into grouped format: merge all members' entrypoints then group by type.
    let mut all_ep_infos: Vec<(EntrypointInfo, String)> = Vec::new();
    for (project_name, entries) in &ws_entrypoints {
        for (sym, outgoing) in entries {
            all_ep_infos.push((
                EntrypointInfo {
                    name: format!("{project_name}::{}", sym.name),
                    file_path: sym.file_path.clone(),
                    method_count: 0,
                    outgoing_call_count: *outgoing,
                    kind: sym.kind.clone(),
                },
                project_name.clone(),
            ));
        }
    }

    // Group by type for display.
    let group_order = [
        "API Controllers",
        "Background Workers",
        "Event Handlers",
        "Other",
    ];
    let mut ep_groups: Vec<(String, Vec<EntrypointInfo>)> = Vec::new();
    for &group_name in &group_order {
        let members: Vec<EntrypointInfo> = all_ep_infos
            .iter()
            .filter(|(e, _)| {
                crate::commands::entrypoints::classify_group(&e.file_path) == group_name
            })
            .map(|(e, _)| e.clone())
            .collect();
        if !members.is_empty() {
            ep_groups.push((group_name.to_string(), members));
        }
    }
    let ep_total = all_ep_infos.len();

    // 3. Core symbols from each member, merged and re-sorted.
    let per_member_limit = args.limit.saturating_mul(2);
    let mut all_core: Vec<CoreSymbol> = Vec::new();

    for member in wg.members() {
        match member.graph.get_symbols_by_importance(per_member_limit) {
            Ok(symbols) => {
                for (sym, count) in symbols {
                    all_core.push(CoreSymbol {
                        name: sym.name,
                        kind: sym.kind,
                        file_path: sym.file_path,
                        caller_count: count,
                        project: Some(member.name.clone()),
                    });
                }
            }
            Err(e) => {
                tracing::warn!("Error getting core symbols from '{}': {}", member.name, e);
            }
        }
    }
    all_core.sort_by(|a, b| b.caller_count.cmp(&a.caller_count));
    all_core.truncate(args.limit);

    // 4. Directory stats per project.
    let mut architecture: Vec<DirStats> = Vec::new();
    for member in wg.members() {
        match member.graph.get_directory_stats() {
            Ok(dirs) => {
                for (dir, files, symbols) in dirs {
                    architecture.push(DirStats {
                        directory: format!("{}/{}", member.name, dir),
                        file_count: files,
                        symbol_count: symbols,
                    });
                }
            }
            Err(e) => {
                tracing::warn!(
                    "Error getting directory stats from '{}': {}",
                    member.name,
                    e
                );
            }
        }
    }
    architecture.sort_by(|a, b| b.symbol_count.cmp(&a.symbol_count));

    if args.json || args.compact {
        let data = MapData {
            stats,
            entrypoints: ep_groups,
            core_symbols: all_core,
            architecture,
        };
        let output = JsonOutput {
            command: "map",
            symbol: None,
            data: &data,
            truncated: false,
            total: ep_total,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        formatter::print_map(
            &config.workspace.name,
            &stats,
            &ep_groups,
            &all_core,
            &architecture,
        );
    }

    Ok(())
}
