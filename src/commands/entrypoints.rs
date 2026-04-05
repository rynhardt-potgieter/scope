/// `scope entrypoints` — list entry points in the codebase.
///
/// Shows symbols with no incoming call edges, grouped by type:
/// API controllers, background workers, event handlers, and other.
/// These are the starting points for request flows.
///
/// In workspace mode (`--workspace`), shows entry points from all members,
/// grouped first by project, then by type.
///
/// Examples:
///   scope entrypoints            — list all entry points
///   scope entrypoints --json     — JSON output
///   scope entrypoints --workspace — across all workspace members
use anyhow::{bail, Result};
use clap::Args;
use serde::Serialize;
use std::path::Path;

use crate::config::workspace::WorkspaceConfig;
use crate::core::graph::Graph;
use crate::core::workspace_graph::WorkspaceGraph;
use crate::output::formatter;
use crate::output::json::JsonOutput;
use crate::Context;

/// List entry points — API controllers, workers, and event handlers.
///
/// Shows symbols with no incoming call edges, grouped by type.
/// These are the starting points for request flows: HTTP endpoints,
/// background workers, event handlers, and standalone functions.
///
/// Examples:
///   scope entrypoints
///   scope entrypoints --json
///   scope entrypoints --workspace
#[derive(Args, Debug)]
pub struct EntrypointsArgs {
    /// Output as JSON instead of human-readable format
    #[arg(long, short = 'j')]
    pub json: bool,
}

/// Information about an entry point for display and JSON output.
#[derive(Debug, Clone, Serialize)]
pub struct EntrypointInfo {
    /// Symbol name.
    pub name: String,
    /// File path relative to project root.
    pub file_path: String,
    /// Number of child methods (for class-level entries).
    pub method_count: usize,
    /// Number of outgoing call edges (fan-out).
    pub outgoing_call_count: usize,
    /// Symbol kind (function, method, class).
    pub kind: String,
}

/// Classify an entry point into a group based on its file path.
pub fn classify_group(file_path: &str) -> &'static str {
    let lower = file_path.to_lowercase();
    if lower.contains("controller") {
        "API Controllers"
    } else if lower.contains("worker") || lower.contains("job") {
        "Background Workers"
    } else if lower.contains("handler") || lower.contains("listener") {
        "Event Handlers"
    } else {
        "Other"
    }
}

/// Collapse and group raw entrypoints into classified groups.
///
/// Collapses class-level entries (counts child methods, skips individual
/// methods belonging to entry-point classes), then groups by classification
/// (API Controllers, Background Workers, Event Handlers, Other).
///
/// Returns `(groups, total_count, file_count)`.
pub(crate) fn collapse_and_group(
    raw_entrypoints: &[(crate::core::graph::Symbol, usize)],
    graph: &Graph,
) -> (Vec<(String, Vec<EntrypointInfo>)>, usize, usize) {
    let mut class_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut class_method_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    for (sym, _) in raw_entrypoints {
        if sym.kind == "class" {
            class_ids.insert(sym.id.clone());
            let methods = graph.get_methods(&sym.id).unwrap_or_default();
            class_method_counts.insert(sym.id.clone(), methods.len());
        }
    }

    let mut infos: Vec<EntrypointInfo> = Vec::new();
    for (sym, outgoing) in raw_entrypoints {
        if let Some(ref parent) = sym.parent_id {
            if class_ids.contains(parent) {
                continue;
            }
        }

        let method_count = class_method_counts.get(&sym.id).copied().unwrap_or(0);

        infos.push(EntrypointInfo {
            name: sym.name.clone(),
            file_path: sym.file_path.clone(),
            method_count,
            outgoing_call_count: *outgoing,
            kind: sym.kind.clone(),
        });
    }

    let unique_files: std::collections::HashSet<&str> =
        infos.iter().map(|e| e.file_path.as_str()).collect();
    let file_count = unique_files.len();
    let total = infos.len();

    let group_order = [
        "API Controllers",
        "Background Workers",
        "Event Handlers",
        "Other",
    ];
    let mut groups: Vec<(String, Vec<EntrypointInfo>)> = Vec::new();

    for &group_name in &group_order {
        let members: Vec<EntrypointInfo> = infos
            .iter()
            .filter(|e| classify_group(&e.file_path) == group_name)
            .cloned()
            .collect();
        if !members.is_empty() {
            groups.push((group_name.to_string(), members));
        }
    }

    (groups, total, file_count)
}

/// Run the `scope entrypoints` command.
pub fn run(args: &EntrypointsArgs, ctx: &Context) -> Result<()> {
    match ctx {
        Context::SingleProject { root } => run_single(args, root),
        Context::Workspace {
            workspace_root,
            config,
            ..
        } => run_workspace(args, workspace_root, config),
    }
}

/// Run entrypoints for a single project.
fn run_single(args: &EntrypointsArgs, project_root: &Path) -> Result<()> {
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
    let raw_entrypoints = graph.get_entrypoints()?;
    let (groups, total, file_count) = collapse_and_group(&raw_entrypoints, &graph);

    if args.json {
        let output = JsonOutput {
            command: "entrypoints",
            symbol: None,
            data: &groups,
            truncated: false,
            total,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        formatter::print_entrypoints(&groups, total, file_count);
    }

    Ok(())
}

/// Run entrypoints across all workspace members.
fn run_workspace(
    args: &EntrypointsArgs,
    workspace_root: &Path,
    config: &WorkspaceConfig,
) -> Result<()> {
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

    // Build workspace-aware grouped output using collapse_and_group per member
    // so class entry points are collapsed consistently with single-project mode.
    let mut all_groups: Vec<(String, Vec<EntrypointInfo>)> = Vec::new();
    let mut total = 0usize;
    let mut file_count = 0usize;

    for member in wg.members() {
        let raw = member.graph.get_entrypoints().unwrap_or_default();
        let (member_groups, member_total, member_files) = collapse_and_group(&raw, &member.graph);

        total += member_total;
        file_count += member_files;

        for (group_name, mut entries) in member_groups {
            // Prefix names with project name for workspace display
            for info in &mut entries {
                info.name = format!("{}::{}", member.name, info.name);
            }
            let label = format!("{} \u{2014} {group_name}", member.name);
            all_groups.push((label, entries));
        }
    }

    if args.json {
        let output = JsonOutput {
            command: "entrypoints",
            symbol: None,
            data: &all_groups,
            truncated: false,
            total,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        formatter::print_entrypoints(&all_groups, total, file_count);
    }

    Ok(())
}
