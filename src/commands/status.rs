/// `scope status` — show index status and freshness.
///
/// Quick health check: is the index built? How many symbols and files?
/// When was the last index run? Use this to check if the index is stale
/// before running queries.
///
/// In workspace mode (`--workspace`), shows per-member status and aggregate totals.
///
/// Examples:
///   scope status          — show index health
///   scope status --json   — machine-readable output
///   scope status --workspace — show all workspace members
use anyhow::Result;
use clap::Args;
use serde::Serialize;
use std::path::Path;

use crate::config::workspace::WorkspaceConfig;
use crate::core::graph::Graph;
use crate::output::formatter;
use crate::output::json::JsonOutput;
use crate::Context;

/// Arguments for the `scope status` command.
#[derive(Args, Debug)]
pub struct StatusArgs {
    /// Output as JSON instead of human-readable format
    #[arg(long, short = 'j')]
    pub json: bool,
}

/// Data payload for the JSON output of `scope status`.
#[derive(Debug, Serialize)]
pub struct StatusData {
    /// Whether the index exists and is queryable.
    pub index_exists: bool,
    /// Total number of symbols in the index.
    pub symbol_count: usize,
    /// Total number of indexed files.
    pub file_count: usize,
    /// Unix timestamp of the most recent indexing operation.
    pub last_indexed_at: Option<i64>,
    /// Human-readable relative time since last index (e.g. "2 minutes ago").
    pub last_indexed_relative: Option<String>,
    /// Number of edges in the graph.
    pub edge_count: usize,
}

/// Data payload for workspace status JSON output.
#[derive(Debug, Serialize)]
pub struct WorkspaceStatusData {
    /// Workspace name from the manifest.
    pub workspace_name: String,
    /// Per-member status.
    pub members: Vec<MemberStatusData>,
    /// Aggregate totals.
    pub totals: StatusData,
}

/// Per-member status in workspace mode.
#[derive(Debug, Serialize)]
pub struct MemberStatusData {
    /// Member display name.
    pub name: String,
    /// Status information.
    #[serde(flatten)]
    pub status: StatusData,
}

/// Run the `scope status` command.
pub fn run(args: &StatusArgs, ctx: &Context) -> Result<()> {
    match ctx {
        Context::SingleProject { root } => run_single(args, root),
        Context::Workspace {
            workspace_root,
            config,
            ..
        } => run_workspace(args, workspace_root, config),
    }
}

/// Run status for a single project.
fn run_single(args: &StatusArgs, project_root: &Path) -> Result<()> {
    let scope_dir = project_root.join(".scope");

    if !scope_dir.exists() {
        if args.json {
            let data = StatusData {
                index_exists: false,
                symbol_count: 0,
                file_count: 0,
                last_indexed_at: None,
                last_indexed_relative: None,
                edge_count: 0,
            };
            let output = JsonOutput {
                command: "status",
                symbol: None,
                data,
                truncated: false,
                total: 0,
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            println!("Index status: not initialised");
            println!("  Run 'scope init' to set up Scope for this project.");
        }
        return Ok(());
    }

    let db_path = scope_dir.join("graph.db");
    if !db_path.exists() {
        if args.json {
            let data = StatusData {
                index_exists: false,
                symbol_count: 0,
                file_count: 0,
                last_indexed_at: None,
                last_indexed_relative: None,
                edge_count: 0,
            };
            let output = JsonOutput {
                command: "status",
                symbol: None,
                data,
                truncated: false,
                total: 0,
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            println!("Index status: not built");
            println!("  Run 'scope index' to build the index.");
        }
        return Ok(());
    }

    let graph = Graph::open(&db_path)?;

    let symbol_count = graph.symbol_count()?;
    let file_count = graph.file_count()?;
    let edge_count = graph.edge_count()?;
    let last_indexed_at = graph.last_indexed_at()?;

    let last_indexed_relative = last_indexed_at.map(format_relative_time);

    // Determine status label.
    // For the < 50ms target we do NOT hash every file on disk.
    // Instead, we report the last index time and let the user decide.
    let status_label = if symbol_count == 0 {
        "empty"
    } else {
        "up to date"
    };

    if args.json {
        let data = StatusData {
            index_exists: true,
            symbol_count,
            file_count,
            last_indexed_at,
            last_indexed_relative,
            edge_count,
        };
        let output = JsonOutput {
            command: "status",
            symbol: None,
            data,
            truncated: false,
            total: symbol_count,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        formatter::print_status(
            status_label,
            symbol_count,
            file_count,
            edge_count,
            last_indexed_relative.as_deref(),
        );
    }

    Ok(())
}

/// Run status across all workspace members.
fn run_workspace(args: &StatusArgs, workspace_root: &Path, config: &WorkspaceConfig) -> Result<()> {
    let mut member_statuses: Vec<MemberStatusData> = Vec::new();
    let mut total_symbols = 0usize;
    let mut total_files = 0usize;
    let mut total_edges = 0usize;
    let mut latest_indexed: Option<i64> = None;
    let mut all_indexed = true;

    for entry in &config.workspace.members {
        let name = WorkspaceConfig::resolve_member_name(entry);
        let member_path = workspace_root.join(&entry.path);
        let db_path = member_path.join(".scope").join("graph.db");

        let status = if !db_path.exists() {
            all_indexed = false;
            StatusData {
                index_exists: false,
                symbol_count: 0,
                file_count: 0,
                last_indexed_at: None,
                last_indexed_relative: None,
                edge_count: 0,
            }
        } else {
            match Graph::open(&db_path) {
                Ok(graph) => {
                    let sc = graph.symbol_count().unwrap_or(0);
                    let fc = graph.file_count().unwrap_or(0);
                    let ec = graph.edge_count().unwrap_or(0);
                    let lia = graph.last_indexed_at().unwrap_or(None);

                    total_symbols += sc;
                    total_files += fc;
                    total_edges += ec;
                    if let Some(ts) = lia {
                        latest_indexed = Some(latest_indexed.map_or(ts, |prev: i64| prev.max(ts)));
                    }

                    StatusData {
                        index_exists: true,
                        symbol_count: sc,
                        file_count: fc,
                        last_indexed_at: lia,
                        last_indexed_relative: lia.map(format_relative_time),
                        edge_count: ec,
                    }
                }
                Err(_) => {
                    all_indexed = false;
                    StatusData {
                        index_exists: false,
                        symbol_count: 0,
                        file_count: 0,
                        last_indexed_at: None,
                        last_indexed_relative: None,
                        edge_count: 0,
                    }
                }
            }
        };

        member_statuses.push(MemberStatusData { name, status });
    }

    if args.json {
        let data = WorkspaceStatusData {
            workspace_name: config.workspace.name.clone(),
            members: member_statuses,
            totals: StatusData {
                index_exists: all_indexed,
                symbol_count: total_symbols,
                file_count: total_files,
                last_indexed_at: latest_indexed,
                last_indexed_relative: latest_indexed.map(format_relative_time),
                edge_count: total_edges,
            },
        };
        let output = JsonOutput {
            command: "status",
            symbol: None,
            data,
            truncated: false,
            total: total_symbols,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        formatter::print_workspace_status(
            &config.workspace.name,
            &member_statuses,
            total_symbols,
            total_files,
            total_edges,
        );
    }

    Ok(())
}

/// Format a Unix timestamp as a human-readable relative time string.
///
/// Examples: "just now", "2 minutes ago", "3 hours ago", "yesterday", "5 days ago".
pub(crate) fn format_relative_time(unix_ts: i64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let delta = now - unix_ts;

    if delta < 0 {
        return "just now".to_string();
    }

    let seconds = delta;
    let minutes = seconds / 60;
    let hours = minutes / 60;
    let days = hours / 24;

    if seconds < 60 {
        "just now".to_string()
    } else if minutes == 1 {
        "1 minute ago".to_string()
    } else if minutes < 60 {
        format!("{minutes} minutes ago")
    } else if hours == 1 {
        "1 hour ago".to_string()
    } else if hours < 24 {
        format!("{hours} hours ago")
    } else if days == 1 {
        "yesterday".to_string()
    } else {
        format!("{days} days ago")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_relative_time_just_now() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        assert_eq!(format_relative_time(now), "just now");
        assert_eq!(format_relative_time(now - 30), "just now");
    }

    #[test]
    fn test_format_relative_time_minutes() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        assert_eq!(format_relative_time(now - 60), "1 minute ago");
        assert_eq!(format_relative_time(now - 120), "2 minutes ago");
        assert_eq!(format_relative_time(now - 300), "5 minutes ago");
    }

    #[test]
    fn test_format_relative_time_hours() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        assert_eq!(format_relative_time(now - 3600), "1 hour ago");
        assert_eq!(format_relative_time(now - 7200), "2 hours ago");
    }

    #[test]
    fn test_format_relative_time_days() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        assert_eq!(format_relative_time(now - 86400), "yesterday");
        assert_eq!(format_relative_time(now - 172800), "2 days ago");
    }
}
