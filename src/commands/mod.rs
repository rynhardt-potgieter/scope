/// CLI subcommand implementations for Scope.
///
/// Each module corresponds to one `scope` subcommand.
pub mod deps;
pub mod diff;
pub mod entrypoints;
pub mod find;
pub mod flow;
pub mod impact;
pub mod index;
pub mod init;
pub mod map;
pub mod rdeps;
pub mod refs;
pub mod setup;
pub mod similar;
pub mod sketch;
pub mod source;
pub mod status;
pub mod summary;
pub mod trace;
pub mod workspace;

use crate::core::graph::{Graph, Symbol};

/// Resolve a symbol name, bailing with a disambiguation list if ambiguous.
///
/// If the name contains `::` it's treated as an exact ID prefix match.
/// Otherwise delegates to `Graph::find_symbol` (single result) and falls
/// back to `find_all_matching_symbols` when the caller needs to know
/// about ambiguity.
pub fn resolve_symbol(graph: &Graph, name: &str) -> anyhow::Result<Symbol> {
    // Allow exact ID prefix: "src/core/graph.rs::find_symbol"
    if name.contains("::") {
        let sym = graph.find_symbol_by_id_prefix(name)?;
        if let Some(s) = sym {
            return Ok(s);
        }
    }

    // Try normal resolution first
    if let Some(sym) = graph.find_symbol(name)? {
        // Check if ambiguous
        let all = graph.find_all_matching_symbols(name)?;
        if all.len() > 1 {
            let mut msg = format!(
                "Ambiguous symbol '{}' matches {} definitions:\n",
                name,
                all.len()
            );
            for (i, s) in all.iter().enumerate() {
                msg.push_str(&format!(
                    "  {}. {} ({})  {}:{}\n",
                    i + 1,
                    s.name,
                    s.kind,
                    s.file_path,
                    s.line_start,
                ));
            }
            msg.push_str(&format!(
                "\nUse a qualified name to disambiguate:\n  scope <cmd> {}::{}::{}",
                all[0].file_path, all[0].name, all[0].kind,
            ));
            anyhow::bail!("{msg}");
        }
        return Ok(sym);
    }

    anyhow::bail!(
        "Symbol '{}' not found in index.\n\
         Tip: Check spelling, or use 'scope find \"{}\"' for semantic search.",
        name,
        name,
    );
}

/// Emit a stderr warning if the index has stale files.
///
/// Runs a quick mtime check against `file_hashes.indexed_at`. Costs one
/// table scan (~1ms for 100 files) so it's cheap enough to run on every
/// query command.
pub fn warn_if_stale(graph: &Graph, project_root: &std::path::Path) {
    if let Ok(true) = graph.has_stale_files(project_root) {
        eprintln!(
            "Warning: file(s) changed since last index. Run `scope index` for accurate results."
        );
    }
}

/// Check if an input string looks like a file path rather than a symbol name.
pub fn looks_like_file_path(input: &str) -> bool {
    input.contains('/')
        || input.contains('\\')
        || input.ends_with(".ts")
        || input.ends_with(".tsx")
        || input.ends_with(".js")
        || input.ends_with(".jsx")
        || input.ends_with(".cs")
        || input.ends_with(".rs")
        || input.ends_with(".py")
        || input.ends_with(".go")
        || input.ends_with(".java")
}
