/// `scope map` — show a structural overview of the entire repository.
///
/// Displays entry points, core symbols ranked by importance,
/// architecture layers, and key statistics. Designed to give
/// an LLM agent a complete mental model of the codebase in
/// ~500-1000 tokens, replacing multiple `scope sketch` calls.
///
/// Examples:
///   scope map              — full repository map
///   scope map --limit 5    — show top 5 core symbols
///   scope map --json       — JSON output
use anyhow::{bail, Result};
use clap::Args;
use serde::Serialize;
use std::path::Path;

use crate::commands::entrypoints::{classify_group, EntrypointInfo};
use crate::core::graph::Graph;
use crate::output::formatter;
use crate::output::json::JsonOutput;

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
#[derive(Args, Debug)]
pub struct MapArgs {
    /// Maximum symbols to show in the core symbols section
    #[arg(long, default_value = "10")]
    pub limit: usize,

    /// Output as JSON instead of human-readable format
    #[arg(long, short = 'j')]
    pub json: bool,
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
pub fn run(args: &MapArgs, project_root: &Path) -> Result<()> {
    let scope_dir = project_root.join(".scope");

    if !scope_dir.exists() {
        bail!("No .scope/ directory found. Run 'scope init' first.");
    }

    let db_path = scope_dir.join("graph.db");
    if !db_path.exists() {
        bail!("No index found. Run 'scope index' to build one first.");
    }

    let graph = Graph::open(&db_path)?;

    // 1. Gather statistics.
    let stats = MapStats {
        file_count: graph.file_count()?,
        symbol_count: graph.symbol_count()?,
        edge_count: graph.edge_count()?,
        languages: graph.get_languages()?,
    };

    // 2. Get entry points (reuse entrypoints logic).
    let raw_entrypoints = graph.get_entrypoints()?;

    // Collapse class entries: count child methods and skip individual methods.
    let mut class_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut class_method_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    for (sym, _) in &raw_entrypoints {
        if sym.kind == "class" {
            class_ids.insert(sym.id.clone());
            let methods = graph.get_methods(&sym.id).unwrap_or_default();
            class_method_counts.insert(sym.id.clone(), methods.len());
        }
    }

    let mut ep_infos: Vec<EntrypointInfo> = Vec::new();
    for (sym, outgoing) in &raw_entrypoints {
        if let Some(ref parent) = sym.parent_id {
            if class_ids.contains(parent) {
                continue;
            }
        }

        let method_count = class_method_counts.get(&sym.id).copied().unwrap_or(0);

        ep_infos.push(EntrypointInfo {
            name: sym.name.clone(),
            file_path: sym.file_path.clone(),
            method_count,
            outgoing_call_count: *outgoing,
            kind: sym.kind.clone(),
        });
    }

    // Group entry points by classification, limit to 8.
    let group_order = [
        "API Controllers",
        "Background Workers",
        "Event Handlers",
        "Other",
    ];
    let mut ep_groups: Vec<(String, Vec<EntrypointInfo>)> = Vec::new();
    let mut ep_total = 0usize;

    for &group_name in &group_order {
        let members: Vec<EntrypointInfo> = ep_infos
            .iter()
            .filter(|e| classify_group(&e.file_path) == group_name)
            .cloned()
            .collect();
        if !members.is_empty() {
            ep_total += members.len();
            ep_groups.push((group_name.to_string(), members));
        }
    }

    // 3. Get core symbols by importance.
    let raw_core = graph.get_symbols_by_importance(args.limit)?;
    let core_symbols: Vec<CoreSymbol> = raw_core
        .into_iter()
        .map(|(sym, count)| CoreSymbol {
            name: sym.name,
            kind: sym.kind,
            file_path: sym.file_path,
            caller_count: count,
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

    if args.json {
        let data = MapData {
            stats,
            entrypoints: ep_groups,
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
