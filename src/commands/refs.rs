/// `scope refs <symbol>` — find all references to a symbol.
///
/// Returns all call sites, imports, type annotations, and other references
/// across the codebase. Use before changing a function signature to find all callers.
///
/// For class symbols, references are grouped by kind (instantiated, extended,
/// imported, used as type). For functions/methods, a flat list is shown.
///
/// In workspace mode (`--workspace`), fans out to all members and tags
/// results with the source project name.
///
/// Examples:
///   scope refs processPayment              — all references to a function
///   scope refs PaymentService              — grouped references to a class
///   scope refs PaymentService --kind calls — only call sites
///   scope refs src/payments/service.ts     — all refs to symbols in a file
///   scope refs processPayment --workspace  — search across workspace
use anyhow::{bail, Result};
use clap::Args;
use std::collections::HashMap;
use std::path::Path;

use crate::config::project::is_vendor_path;
use crate::config::workspace::WorkspaceConfig;
use crate::config::ProjectConfig;
use crate::core::graph::Graph;
use crate::core::graph::Reference;
use crate::core::workspace_graph::WorkspaceGraph;
use crate::output::formatter;
use crate::output::json::JsonOutput;
use crate::Context;

/// Arguments for the `scope refs` command.
#[derive(Args, Debug)]
pub struct RefsArgs {
    /// Symbol name or file path to find references for.
    ///
    /// Pass a function/method name to see all call sites.
    /// Pass a class name to see references grouped by kind.
    /// Pass a file path to see references to all symbols in that file.
    ///
    /// Examples: processPayment, PaymentService, src/payments/service.ts
    pub symbol: String,

    /// Filter by edge kind: calls, imports, extends, implements, instantiates, references
    #[arg(long)]
    pub kind: Option<String>,

    /// Maximum number of references to show (default: 20)
    #[arg(long, default_value = "20")]
    pub limit: usize,

    /// Lines of surrounding code context to show per reference (default: 0)
    #[arg(long, short = 'c', default_value = "0")]
    pub context: usize,

    /// Output as JSON instead of human-readable format
    #[arg(long, short = 'j')]
    pub json: bool,
}

/// Arguments for the `scope callers` command (alias for refs --kind calls).
#[derive(Args, Debug)]
pub struct CallersArgs {
    /// Symbol name to find callers for
    pub symbol: String,

    /// Traversal depth (default 1 = direct callers only, 2+ = transitive callers)
    #[arg(long, default_value = "1")]
    pub depth: usize,

    /// Maximum callers to show (default: 20, only applies at depth 1)
    #[arg(long, default_value = "20")]
    pub limit: usize,

    /// Lines of surrounding code context per caller (default: 0, only applies at depth 1)
    #[arg(long, short = 'c', default_value = "0")]
    pub context: usize,

    /// Output as JSON instead of human-readable format
    #[arg(long, short = 'j')]
    pub json: bool,
}

/// Run the `scope callers` command (shorthand for refs --kind calls).
///
/// When `depth == 1`: delegates to `run()` with `kind=Some("calls")` (flat caller list).
/// When `depth > 1`: performs transitive impact analysis via `graph.find_impact()`.
pub fn run_callers(args: &CallersArgs, project_root: &Path) -> Result<()> {
    if args.depth > 1 {
        return run_callers_transitive(args, project_root, "callers");
    }

    let refs_args = RefsArgs {
        symbol: args.symbol.clone(),
        kind: Some("calls".to_string()),
        limit: args.limit,
        context: args.context,
        json: args.json,
    };
    let ctx = Context::SingleProject {
        root: project_root.to_path_buf(),
    };
    run(&refs_args, &ctx)
}

/// Run transitive caller analysis (depth > 1) using the impact graph query.
///
/// The `command_label` is used in JSON output to identify the command
/// (e.g. `"callers"` or `"impact"` for backward compatibility).
pub(super) fn run_callers_transitive(
    args: &CallersArgs,
    project_root: &Path,
    command_label: &'static str,
) -> Result<()> {
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
            command: command_label,
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

/// Enrich references with source line snippets from the actual files.
///
/// Groups refs by file path to avoid reading the same file multiple times.
/// Sets `snippet_line` to the source line at the reference location (always).
/// Sets `snippet` to surrounding context lines (only when `context_lines > 0`).
/// Gracefully degrades: if a file cannot be read, leaves fields as `None`.
fn enrich_refs_with_snippets(refs: &mut [Reference], project_root: &Path, context_lines: usize) {
    // Group ref indices by file_path
    let mut by_file: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, r) in refs.iter().enumerate() {
        by_file.entry(r.file_path.clone()).or_default().push(i);
    }

    for (file_path, indices) in &by_file {
        let full_path = project_root.join(file_path);
        let lines = match std::fs::read_to_string(&full_path) {
            Ok(content) => content.lines().map(String::from).collect::<Vec<_>>(),
            Err(_) => continue, // graceful degradation
        };

        for &idx in indices {
            let r = &mut refs[idx];
            let Some(line_num) = r.line else { continue };
            let line_idx = (line_num as usize).saturating_sub(1);
            if line_idx >= lines.len() {
                continue;
            }

            // Always set snippet_line to the actual source line
            r.snippet_line = Some(lines[line_idx].trim_end().to_string());

            // Set multi-line context if requested
            if context_lines > 0 {
                let start = line_idx.saturating_sub(context_lines);
                let end = (line_idx + context_lines + 1).min(lines.len());
                let ctx: Vec<String> = lines[start..end]
                    .iter()
                    .map(|l| l.trim_end().to_string())
                    .collect();
                r.snippet = Some(ctx);
            }
        }
    }
}

use super::looks_like_file_path;

/// Partition references so first-party results appear before vendor results.
///
/// Within each partition the original order is preserved.
fn derank_vendor_refs(refs: Vec<Reference>, vendor_patterns: &[String]) -> Vec<Reference> {
    if vendor_patterns.is_empty() {
        return refs;
    }
    let (first_party, vendor): (Vec<_>, Vec<_>) = refs
        .into_iter()
        .partition(|r| !is_vendor_path(&r.file_path, vendor_patterns));
    let mut combined = first_party;
    combined.extend(vendor);
    combined
}

/// Run the `scope refs` command.
pub fn run(args: &RefsArgs, ctx: &Context) -> Result<()> {
    match ctx {
        Context::SingleProject { root } => run_single(args, root),
        Context::Workspace {
            workspace_root,
            config,
            ..
        } => run_workspace(args, workspace_root, config),
    }
}

/// Run refs for a single project.
fn run_single(args: &RefsArgs, project_root: &Path) -> Result<()> {
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

    if looks_like_file_path(&args.symbol) {
        return run_file_refs(args, &graph, project_root);
    }

    run_symbol_refs(args, &graph, project_root)
}

/// Find refs for a single symbol.
fn run_symbol_refs(args: &RefsArgs, graph: &Graph, project_root: &Path) -> Result<()> {
    let kinds: Option<Vec<&str>> = args.kind.as_deref().map(|k| vec![k]);
    let kinds_slice = kinds.as_deref();

    // Load vendor patterns for de-ranking
    let vendor_patterns = ProjectConfig::load(&project_root.join(".scope"))
        .map(|c| c.index.vendor_patterns)
        .unwrap_or_default();

    // Check if this is a class-like symbol for grouped output
    let is_class = graph.is_class_like(&args.symbol)?;

    if is_class && kinds_slice.is_none() {
        // Grouped output for class symbols
        let (mut groups, total) = graph.find_refs_grouped(&args.symbol, args.limit)?;

        // De-rank vendor refs within each group
        for (_kind, refs) in &mut groups {
            let taken = std::mem::take(refs);
            *refs = derank_vendor_refs(taken, &vendor_patterns);
        }

        // Enrich all refs in all groups with source snippets
        for (_kind, refs) in &mut groups {
            enrich_refs_with_snippets(refs, project_root, args.context);
        }

        if args.json {
            let data = serde_json::json!({
                "groups": groups.iter().map(|(kind, refs)| {
                    serde_json::json!({
                        "kind": kind,
                        "refs": refs,
                    })
                }).collect::<Vec<_>>(),
            });
            let output = JsonOutput {
                command: "refs",
                symbol: Some(args.symbol.clone()),
                data,
                truncated: total > args.limit,
                total,
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            formatter::print_refs_grouped(&args.symbol, &groups, total);
        }
    } else {
        // Flat output for functions/methods or filtered queries
        let (refs, total) = graph.find_refs(&args.symbol, kinds_slice, args.limit)?;

        // De-rank vendor refs, then enrich with source snippets
        let mut refs = derank_vendor_refs(refs, &vendor_patterns);
        enrich_refs_with_snippets(&mut refs, project_root, args.context);

        if args.json {
            let output = JsonOutput {
                command: "refs",
                symbol: Some(args.symbol.clone()),
                data: &refs,
                truncated: total > args.limit,
                total,
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            formatter::print_refs(&args.symbol, &refs, total);
        }
    }

    Ok(())
}

/// Find refs to all symbols in a file.
fn run_file_refs(args: &RefsArgs, graph: &Graph, project_root: &Path) -> Result<()> {
    let file_path = formatter::normalize_path(&args.symbol);
    let kinds: Option<Vec<&str>> = args.kind.as_deref().map(|k| vec![k]);
    let kinds_slice = kinds.as_deref();

    let (mut refs, total) = graph.find_file_refs(&file_path, kinds_slice, args.limit)?;

    // Enrich refs with source snippets
    enrich_refs_with_snippets(&mut refs, project_root, args.context);

    if args.json {
        let output = JsonOutput {
            command: "refs",
            symbol: Some(file_path.clone()),
            data: &refs,
            truncated: total > args.limit,
            total,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        formatter::print_file_refs(&file_path, &refs, total);
    }

    Ok(())
}

/// Run refs across all workspace members.
fn run_workspace(args: &RefsArgs, workspace_root: &Path, config: &WorkspaceConfig) -> Result<()> {
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

    let kinds: Option<Vec<&str>> = args.kind.as_deref().map(|k| vec![k]);
    let kinds_slice = kinds.as_deref();

    let ws_refs = wg.find_refs(&args.symbol, kinds_slice, args.limit);

    if ws_refs.is_empty() {
        if args.json {
            let output = JsonOutput {
                command: "refs",
                symbol: Some(args.symbol.clone()),
                data: &Vec::<serde_json::Value>::new(),
                truncated: false,
                total: 0,
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            println!("No references to '{}' found across workspace.", args.symbol);
        }
        return Ok(());
    }

    let total = ws_refs.len();

    if args.json {
        let output = JsonOutput {
            command: "refs",
            symbol: Some(args.symbol.clone()),
            data: &ws_refs,
            truncated: false,
            total,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        formatter::print_workspace_refs(&args.symbol, &ws_refs, total);
    }

    Ok(())
}
