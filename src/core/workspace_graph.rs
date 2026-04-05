//! Workspace-level query facade over multiple independent project graphs.
//!
//! `WorkspaceGraph` opens N `Graph` instances (one per workspace member)
//! and fans out queries, merging results with project-name tags. This is
//! a pure query-time aggregation layer — no data is written to any shared
//! database. Each member's `.scope/graph.db` remains fully independent.
//!
//! Symbol IDs are never modified on disk. Project prefixing happens at
//! the output boundary only.

use std::path::PathBuf;

use anyhow::Result;
use serde::Serialize;

use crate::core::graph::{Graph, Reference, Symbol};

/// A workspace-level query facade over multiple independent project graphs.
pub struct WorkspaceGraph {
    members: Vec<WorkspaceMember>,
}

/// A single project within a workspace.
pub struct WorkspaceMember {
    /// Human-readable project name from the manifest.
    pub name: String,
    /// Absolute path to the project root.
    #[allow(dead_code)]
    pub root: PathBuf,
    /// Open graph connection.
    pub graph: Graph,
}

/// A symbol result tagged with its source project.
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)] // Part of workspace facade, not yet wired to commands
pub struct WorkspaceSymbol {
    /// The workspace member name this symbol belongs to.
    pub project: String,
    /// Symbol ID (relative to project root, not prefixed).
    pub id: String,
    /// Symbol name.
    pub name: String,
    /// Symbol kind (function, class, method, etc.).
    pub kind: String,
    /// File path relative to the project root.
    pub file_path: String,
    /// First line of the symbol definition (1-based).
    pub line_start: u32,
    /// Last line of the symbol definition (1-based).
    pub line_end: u32,
}

/// A reference result tagged with its source project.
#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceRef {
    /// The workspace member name this reference belongs to.
    pub project: String,
    /// The underlying reference.
    #[serde(flatten)]
    pub reference: Reference,
}

impl WorkspaceGraph {
    /// Open all member graphs.
    ///
    /// For each `(name, root)` pair, attempts to open `root/.scope/graph.db`.
    /// Members with missing `graph.db` are warned about via `tracing::warn!`
    /// and skipped (partial workspace support).
    ///
    /// If there are more than 20 members, emits a performance warning.
    pub fn open(members: Vec<(String, PathBuf)>) -> Result<Self> {
        let total_requested = members.len();

        if total_requested > 20 {
            tracing::warn!(
                "Workspace has {} members. Consider using --project <name> to target a specific member.",
                total_requested
            );
        }

        let mut opened: Vec<WorkspaceMember> = Vec::new();

        for (name, root) in members {
            let db_path = root.join(".scope").join("graph.db");
            if !db_path.exists() {
                tracing::warn!(
                    "Member '{}' has no graph.db at {}. Skipping (run 'scope index' in that project).",
                    name,
                    db_path.display()
                );
                continue;
            }

            match Graph::open(&db_path) {
                Ok(graph) => {
                    opened.push(WorkspaceMember { name, root, graph });
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to open graph for member '{}': {}. Skipping.",
                        name,
                        e
                    );
                }
            }
        }

        let skipped = total_requested.saturating_sub(opened.len());
        if skipped > 0 {
            eprintln!(
                "Warning: {} of {} workspace members not indexed (run 'scope workspace index')",
                skipped, total_requested
            );
        }

        Ok(Self { members: opened })
    }

    /// Number of successfully opened members.
    #[cfg(test)]
    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    /// List member names.
    #[allow(dead_code)] // Part of workspace facade
    pub fn member_names(&self) -> Vec<&str> {
        self.members.iter().map(|m| m.name.as_str()).collect()
    }

    /// Get a reference to the internal members for direct iteration.
    pub fn members(&self) -> &[WorkspaceMember] {
        &self.members
    }

    /// Get deduplicated union of languages across all members.
    pub fn get_languages(&self) -> Vec<String> {
        let mut langs: Vec<String> = Vec::new();
        for member in &self.members {
            match member.graph.get_languages() {
                Ok(member_langs) => {
                    for lang in member_langs {
                        if !langs.contains(&lang) {
                            langs.push(lang);
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to get languages for member '{}': {e}", member.name);
                }
            }
        }
        langs.sort();
        langs
    }

    /// Find a symbol by name across all workspace members.
    ///
    /// Returns all matches tagged with their project name. The caller
    /// decides how to disambiguate (e.g. prompt user or use `--project`).
    #[allow(dead_code)] // Part of workspace facade
    pub fn find_symbol(&self, name: &str) -> Vec<WorkspaceSymbol> {
        let mut results = Vec::new();

        for member in &self.members {
            match member.graph.find_symbol(name) {
                Ok(Some(sym)) => {
                    results.push(WorkspaceSymbol {
                        project: member.name.clone(),
                        id: sym.id,
                        name: sym.name,
                        kind: sym.kind,
                        file_path: sym.file_path,
                        line_start: sym.line_start,
                        line_end: sym.line_end,
                    });
                }
                Ok(None) => {}
                Err(e) => {
                    tracing::warn!(
                        "Error querying member '{}' for symbol '{}': {}",
                        member.name,
                        name,
                        e
                    );
                }
            }
        }

        results
    }

    /// Find references to a symbol across all workspace members.
    ///
    /// Fans out to each member with `limit * 2` to allow fair merging,
    /// then merges results sorted by `(kind, from_name)` and applies
    /// the global limit.
    pub fn find_refs(
        &self,
        symbol_name: &str,
        kinds: Option<&[&str]>,
        limit: usize,
    ) -> Vec<WorkspaceRef> {
        let per_member_limit = limit.saturating_mul(2).max(limit);
        let mut all_refs: Vec<WorkspaceRef> = Vec::new();

        for member in &self.members {
            match member.graph.find_refs(symbol_name, kinds, per_member_limit) {
                Ok((refs, _total)) => {
                    for r in refs {
                        all_refs.push(WorkspaceRef {
                            project: member.name.clone(),
                            reference: r,
                        });
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "Error querying refs in member '{}': {e}. Results may be incomplete.",
                        member.name
                    );
                }
            }
        }

        // Sort by (kind, from_name) for deterministic output
        all_refs.sort_by(|a, b| {
            a.reference
                .kind
                .cmp(&b.reference.kind)
                .then_with(|| a.reference.from_name.cmp(&b.reference.from_name))
        });

        all_refs.truncate(limit);
        all_refs
    }

    /// Get entry points from all workspace members.
    ///
    /// Returns a vec of `(project_name, entrypoints)` for each member.
    #[allow(dead_code)] // Part of workspace facade
    pub fn get_entrypoints(&self) -> Vec<(String, Vec<(Symbol, usize)>)> {
        let mut results = Vec::new();

        for member in &self.members {
            match member.graph.get_entrypoints() {
                Ok(entries) => {
                    if !entries.is_empty() {
                        results.push((member.name.clone(), entries));
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "Error getting entrypoints from member '{}': {}",
                        member.name,
                        e
                    );
                }
            }
        }

        results
    }

    /// Total symbol count across all workspace members.
    pub fn symbol_count(&self) -> usize {
        self.members
            .iter()
            .map(|m| m.graph.symbol_count().unwrap_or(0))
            .sum()
    }

    /// Total edge count across all workspace members.
    pub fn edge_count(&self) -> usize {
        self.members
            .iter()
            .map(|m| m.graph.edge_count().unwrap_or(0))
            .sum()
    }

    /// Total file count across all workspace members.
    pub fn file_count(&self) -> usize {
        self.members
            .iter()
            .map(|m| m.graph.file_count().unwrap_or(0))
            .sum()
    }
}

/// Prefix a symbol ID with the project name for workspace display.
#[cfg(test)]
pub fn workspace_display_id(project: &str, id: &str) -> String {
    format!("{project}::{id}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::graph::Symbol;
    use tempfile::TempDir;

    /// Helper: create a test symbol.
    fn test_symbol(name: &str, kind: &str, file_path: &str) -> Symbol {
        Symbol {
            id: format!("{file_path}::{name}::{kind}"),
            name: name.to_string(),
            kind: kind.to_string(),
            file_path: file_path.to_string(),
            line_start: 1,
            line_end: 50,
            signature: None,
            docstring: None,
            parent_id: None,
            language: "typescript".to_string(),
            metadata: "{}".to_string(),
        }
    }

    /// Helper: create a workspace member directory with a graph.db containing a test symbol.
    fn create_member_with_db(dir: &TempDir, name: &str) -> PathBuf {
        let member_root = dir.path().join(name);
        let scope_dir = member_root.join(".scope");
        std::fs::create_dir_all(&scope_dir).unwrap();

        let db_path = scope_dir.join("graph.db");
        let mut graph = Graph::open(&db_path).unwrap();

        let sym = test_symbol(&format!("{name}Service"), "class", "src/main.ts");
        graph.insert_file_data("src/main.ts", &[sym], &[]).unwrap();

        member_root
    }

    #[test]
    fn open_skips_members_without_graph_db() {
        let dir = TempDir::new().unwrap();

        let api_root = create_member_with_db(&dir, "api");
        let worker_root = dir.path().join("worker");
        std::fs::create_dir_all(&worker_root).unwrap();

        let wg = WorkspaceGraph::open(vec![
            ("api".to_string(), api_root),
            ("worker".to_string(), worker_root),
        ])
        .unwrap();

        assert_eq!(wg.member_count(), 1);
        assert_eq!(wg.member_names(), vec!["api"]);
    }

    #[test]
    fn find_symbol_returns_results_from_multiple_members() {
        let dir = TempDir::new().unwrap();
        let api_root = create_member_with_db(&dir, "api");
        let worker_root = create_member_with_db(&dir, "worker");

        let wg = WorkspaceGraph::open(vec![
            ("api".to_string(), api_root),
            ("worker".to_string(), worker_root),
        ])
        .unwrap();

        let api_results = wg.find_symbol("apiService");
        let worker_results = wg.find_symbol("workerService");

        assert_eq!(api_results.len(), 1);
        assert_eq!(api_results[0].project, "api");
        assert_eq!(worker_results.len(), 1);
        assert_eq!(worker_results[0].project, "worker");
    }

    #[test]
    fn find_symbol_returns_empty_for_unknown() {
        let dir = TempDir::new().unwrap();
        let api_root = create_member_with_db(&dir, "api");

        let wg = WorkspaceGraph::open(vec![("api".to_string(), api_root)]).unwrap();
        let results = wg.find_symbol("NonExistent");
        assert!(results.is_empty());
    }

    #[test]
    fn symbol_count_sums_across_members() {
        let dir = TempDir::new().unwrap();
        let api_root = create_member_with_db(&dir, "api");
        let worker_root = create_member_with_db(&dir, "worker");

        let wg = WorkspaceGraph::open(vec![
            ("api".to_string(), api_root),
            ("worker".to_string(), worker_root),
        ])
        .unwrap();

        assert_eq!(wg.symbol_count(), 2);
    }

    #[test]
    fn workspace_display_id_prefixes_correctly() {
        let display = workspace_display_id("api", "src/main.ts::PaymentService::class");
        assert_eq!(display, "api::src/main.ts::PaymentService::class");
    }

    #[test]
    fn empty_workspace_has_zero_counts() {
        let wg = WorkspaceGraph::open(vec![]).unwrap();
        assert_eq!(wg.member_count(), 0);
        assert_eq!(wg.symbol_count(), 0);
        assert_eq!(wg.edge_count(), 0);
        assert_eq!(wg.file_count(), 0);
    }
}
