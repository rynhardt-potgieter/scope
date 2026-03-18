//! SQLite-backed dependency graph storage.
//!
//! Stores symbols, edges, and file hashes. Provides query methods
//! for refs, deps, rdeps, and impact analysis.
use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// The dependency graph backed by SQLite.
pub struct Graph {
    conn: Connection,
}

/// A code symbol extracted from source and stored in the graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    /// Unique identifier: `"{file_path}::{name}::{kind}"`.
    pub id: String,
    /// The symbol name (e.g. `PaymentService`, `processPayment`).
    pub name: String,
    /// The kind of symbol (function, class, method, etc.).
    pub kind: String,
    /// File path relative to project root, always forward slashes.
    pub file_path: String,
    /// First line of the symbol definition (1-based).
    pub line_start: u32,
    /// Last line of the symbol definition (1-based).
    pub line_end: u32,
    /// Full type signature where available.
    pub signature: Option<String>,
    /// Extracted doc comment.
    pub docstring: Option<String>,
    /// Parent symbol ID (e.g. class ID for a method).
    pub parent_id: Option<String>,
    /// Source language.
    pub language: String,
    /// JSON blob with modifiers, parameters, return type, etc.
    pub metadata: String,
}

/// A relationship between two symbols.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    /// Source symbol ID.
    pub from_id: String,
    /// Target symbol ID (may reference external symbols not in the index).
    pub to_id: String,
    /// Edge kind: calls, imports, extends, implements, instantiates, references, references_type.
    pub kind: String,
    /// File where this edge was observed.
    pub file_path: String,
    /// Line number where the edge was observed.
    pub line: Option<u32>,
}

/// Result of comparing current file hashes against the stored index.
#[derive(Debug, Default)]
pub struct ChangedFiles {
    /// Files that are new (not previously indexed).
    pub added: Vec<String>,
    /// Files whose content hash has changed.
    pub modified: Vec<String>,
    /// Files that were previously indexed but no longer exist.
    pub deleted: Vec<String>,
}

impl ChangedFiles {
    /// Returns true if there are no changes.
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.modified.is_empty() && self.deleted.is_empty()
    }

    /// Total number of changed files.
    pub fn total(&self) -> usize {
        self.added.len() + self.modified.len() + self.deleted.len()
    }
}

/// Relationships of a class symbol: inheritance, interfaces, and dependencies.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ClassRelationships {
    /// Classes this class extends.
    pub extends: Vec<String>,
    /// Interfaces this class implements.
    pub implements: Vec<String>,
    /// Distinct symbol names from outgoing edges (imports, calls, etc.).
    pub dependencies: Vec<String>,
}

/// Information about a caller of a symbol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallerInfo {
    /// Display name of the caller (e.g. `OrderController.checkout`).
    pub name: String,
    /// Number of call sites from this caller.
    pub count: usize,
}

/// A reference to a symbol from elsewhere in the codebase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    /// The ID of the symbol making the reference.
    pub from_id: String,
    /// The human-readable name of the referencing symbol.
    pub from_name: String,
    /// The kind of reference (calls, imports, extends, etc.).
    pub kind: String,
    /// File path where the reference occurs.
    pub file_path: String,
    /// Line number of the reference, if known.
    pub line: Option<i64>,
    /// Context string (caller name or description).
    pub context: String,
}

/// A node in an impact analysis result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactNode {
    /// Symbol ID.
    pub id: String,
    /// Symbol name.
    pub name: String,
    /// File path where this symbol is defined.
    pub file_path: String,
    /// Symbol kind (function, class, method, etc.).
    pub kind: String,
    /// Depth in the impact graph (1 = direct caller).
    pub depth: usize,
}

/// Result of an impact analysis query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactResult {
    /// Nodes grouped by depth level: `(depth, nodes_at_that_depth)`.
    pub nodes_by_depth: Vec<(usize, Vec<ImpactNode>)>,
    /// Test files that are affected (separated from main results).
    pub test_files: Vec<ImpactNode>,
    /// Total number of distinct affected symbols (excluding test files).
    pub total_affected: usize,
}

/// A dependency of a symbol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// The name of the dependency.
    pub name: String,
    /// File path of the dependency, if it exists in the index.
    pub file_path: Option<String>,
    /// Kind of dependency relationship (imports, calls, extends, etc.).
    pub kind: String,
    /// True if the dependency is not in the index (external package).
    pub is_external: bool,
    /// Depth in the dependency tree (1 = direct).
    pub depth: usize,
}

impl Symbol {
    /// Build a `Symbol` from a rusqlite row.
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            name: row.get("name")?,
            kind: row.get("kind")?,
            file_path: row.get("file_path")?,
            line_start: row.get("line_start")?,
            line_end: row.get("line_end")?,
            signature: row.get("signature")?,
            docstring: row.get("docstring")?,
            parent_id: row.get("parent_id")?,
            language: row.get("language")?,
            metadata: row.get("metadata")?,
        })
    }
}

impl Graph {
    /// Open or create a graph database at the given path.
    ///
    /// Applies performance pragmas and ensures the schema is up to date.
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)
            .with_context(|| format!("Failed to open graph database at {}", path.display()))?;

        // Performance pragmas — safe for single-writer use
        conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA cache_size = -64000;
            PRAGMA temp_store = MEMORY;
            PRAGMA foreign_keys = ON;
            PRAGMA case_sensitive_like = ON;
        ",
        )?;

        Self::ensure_schema(&conn)?;

        Ok(Self { conn })
    }

    /// Create the schema tables and indexes if they do not exist.
    fn ensure_schema(conn: &Connection) -> Result<()> {
        conn.execute_batch(include_str!("../sql/schema.sql"))?;
        Ok(())
    }

    /// Find a symbol by exact name match, or by qualified name (Class.method).
    ///
    /// Lookup order:
    /// 1. Exact match on `symbols.name`. If multiple matches, prefer the one
    ///    with no `parent_id` (top-level symbol).
    /// 2. If not found and `name` contains `.`, split on `.` and try qualified
    ///    lookup: `parent.name = class_part AND s.name = method_part`.
    /// 3. Returns `None` for unknown symbols.
    pub fn find_symbol(&self, name: &str) -> Result<Option<Symbol>> {
        // Try exact match first, preferring top-level symbols (parent_id IS NULL first)
        let result = self
            .conn
            .query_row(
                "SELECT * FROM symbols WHERE name = ?1
                 ORDER BY (CASE WHEN parent_id IS NULL THEN 0 ELSE 1 END)
                 LIMIT 1",
                params![name],
                Symbol::from_row,
            )
            .optional()?;

        if result.is_some() {
            return Ok(result);
        }

        // Try qualified name (ClassName.methodName)
        if let Some((class, method)) = name.split_once('.') {
            return self
                .conn
                .query_row(
                    "SELECT s.* FROM symbols s
                     JOIN symbols parent ON s.parent_id = parent.id
                     WHERE parent.name = ?1 AND s.name = ?2",
                    params![class, method],
                    Symbol::from_row,
                )
                .optional()
                .map_err(Into::into);
        }

        Ok(None)
    }

    /// Get all child symbols (methods, properties) of a class/interface.
    ///
    /// Returns symbols where `parent_id = class_id`, ordered by `line_start`.
    pub fn get_methods(&self, class_id: &str) -> Result<Vec<Symbol>> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM symbols WHERE parent_id = ?1 ORDER BY line_start")?;
        let rows = stmt.query_map(params![class_id], Symbol::from_row)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    /// Count incoming call edges for a symbol (how many callers it has).
    pub fn get_caller_count(&self, symbol_id: &str) -> Result<usize> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM edges WHERE to_id = ?1 AND kind = 'calls'",
            params![symbol_id],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    /// Batch version of `get_caller_count` — returns a map of symbol_id to caller count.
    ///
    /// Efficiently fetches caller counts for multiple symbols in a single query.
    pub fn get_caller_counts(&self, symbol_ids: &[&str]) -> Result<HashMap<String, usize>> {
        let mut result = HashMap::new();
        if symbol_ids.is_empty() {
            return Ok(result);
        }

        // Use a single query with IN clause for efficiency
        let placeholders: Vec<String> = (1..=symbol_ids.len()).map(|i| format!("?{i}")).collect();
        let sql = format!(
            "SELECT to_id, COUNT(*) FROM edges
             WHERE to_id IN ({}) AND kind = 'calls'
             GROUP BY to_id",
            placeholders.join(", ")
        );

        let mut stmt = self.conn.prepare(&sql)?;
        let params: Vec<&dyn rusqlite::ToSql> = symbol_ids
            .iter()
            .map(|id| id as &dyn rusqlite::ToSql)
            .collect();
        let rows = stmt.query_map(params.as_slice(), |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;

        for row in rows {
            let (id, count) = row?;
            result.insert(id, count as usize);
        }

        Ok(result)
    }

    /// Get class relationships: extends, implements, and dependencies.
    pub fn get_class_relationships(&self, class_id: &str) -> Result<ClassRelationships> {
        let mut rels = ClassRelationships::default();

        // Get 'extends' edges from this class
        let mut stmt = self
            .conn
            .prepare("SELECT to_id FROM edges WHERE from_id = ?1 AND kind = 'extends'")?;
        let rows = stmt.query_map(params![class_id], |row| row.get::<_, String>(0))?;
        for row in rows {
            let to_id = row?;
            rels.extends.push(self.symbol_name_from_id(&to_id));
        }

        // Get 'implements' edges from this class
        let mut stmt = self
            .conn
            .prepare("SELECT to_id FROM edges WHERE from_id = ?1 AND kind = 'implements'")?;
        let rows = stmt.query_map(params![class_id], |row| row.get::<_, String>(0))?;
        for row in rows {
            let to_id = row?;
            rels.implements.push(self.symbol_name_from_id(&to_id));
        }

        // Get dependencies: distinct symbol names from outgoing edges of the class
        // and its methods (excluding extends/implements, which are already captured)
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT e.to_id FROM edges e
             WHERE (e.from_id = ?1 OR e.from_id IN (
                 SELECT id FROM symbols WHERE parent_id = ?1
             ))
             AND e.kind NOT IN ('extends', 'implements')
             AND e.to_id != ?1",
        )?;
        let rows = stmt.query_map(params![class_id], |row| row.get::<_, String>(0))?;
        for row in rows {
            let to_id = row?;
            let name = self.symbol_name_from_id(&to_id);
            if !rels.dependencies.contains(&name) {
                rels.dependencies.push(name);
            }
        }

        Ok(rels)
    }

    /// Get outgoing call edges from a symbol.
    ///
    /// Returns the names of symbols that this symbol calls.
    /// Note: edges may use `__module__` synthetic IDs for `from_id`, so results
    /// from a specific method may be incomplete.
    pub fn get_outgoing_calls(&self, symbol_id: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT e.to_id FROM edges e
             WHERE e.from_id = ?1 AND e.kind = 'calls'",
        )?;
        let rows = stmt.query_map(params![symbol_id], |row| row.get::<_, String>(0))?;
        let mut result = Vec::new();
        for row in rows {
            let to_id = row?;
            result.push(self.symbol_name_from_id(&to_id));
        }
        Ok(result)
    }

    /// Get incoming callers for a symbol, grouped by caller with count.
    ///
    /// Returns `(caller_display_name, call_count)` pairs.
    pub fn get_incoming_callers(&self, symbol_id: &str) -> Result<Vec<CallerInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT e.from_id, COUNT(*) as cnt FROM edges e
             WHERE e.to_id = ?1 AND e.kind = 'calls'
             GROUP BY e.from_id
             ORDER BY cnt DESC",
        )?;
        let rows = stmt.query_map(params![symbol_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;
        let mut result = Vec::new();
        for row in rows {
            let (from_id, count) = row?;
            let name = self.caller_display_name(&from_id);
            result.push(CallerInfo {
                name,
                count: count as usize,
            });
        }
        Ok(result)
    }

    /// Get symbols that implement a given interface.
    pub fn get_implementors(&self, interface_id: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT e.from_id FROM edges e
             WHERE e.to_id = ?1 AND e.kind = 'implements'",
        )?;
        let rows = stmt.query_map(params![interface_id], |row| row.get::<_, String>(0))?;
        let mut result = Vec::new();
        for row in rows {
            let from_id = row?;
            result.push(self.symbol_name_from_id(&from_id));
        }
        Ok(result)
    }

    /// Get all symbols in a file, ordered by line_start.
    pub fn get_file_symbols(&self, file_path: &str) -> Result<Vec<Symbol>> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM symbols WHERE file_path = ?1 ORDER BY line_start")?;
        let rows = stmt.query_map(params![file_path], Symbol::from_row)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    /// Extract a human-readable name from a symbol ID.
    ///
    /// If the ID corresponds to a symbol in the index, returns its name.
    /// Otherwise, extracts the name portion from the ID format `file::name::kind`.
    fn symbol_name_from_id(&self, id: &str) -> String {
        // Try to look up the symbol
        if let Ok(Some(sym)) = self
            .conn
            .query_row(
                "SELECT name FROM symbols WHERE id = ?1",
                params![id],
                |row| row.get::<_, String>(0),
            )
            .optional()
        {
            return sym;
        }

        // Fallback: parse the ID format "file::name::kind"
        if let Some(rest) = id.split("::").nth(1) {
            return rest.to_string();
        }

        id.to_string()
    }

    /// Build a display name for a caller, including parent class if available.
    ///
    /// For `__module__` synthetic IDs, extracts the file stem.
    fn caller_display_name(&self, from_id: &str) -> String {
        // Check if this is a real symbol
        let sym = self
            .conn
            .query_row(
                "SELECT s.name, p.name FROM symbols s
                 LEFT JOIN symbols p ON s.parent_id = p.id
                 WHERE s.id = ?1",
                params![from_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?)),
            )
            .optional();

        match sym {
            Ok(Some((name, Some(parent)))) => format!("{parent}.{name}"),
            Ok(Some((name, None))) => name,
            _ => {
                // Synthetic ID — extract something meaningful
                if from_id.contains("__module__") {
                    // Format: "file_path::__module__::module"
                    if let Some(file_part) = from_id.split("::").next() {
                        if let Some(filename) = file_part.rsplit('/').next() {
                            return filename
                                .rsplit_once('.')
                                .map_or(filename, |(name, _)| name)
                                .to_string();
                        }
                    }
                }
                self.symbol_name_from_id(from_id)
            }
        }
    }

    /// Find all references to a symbol, with optional kind filtering and limit.
    ///
    /// Returns `(references, total_count)` where `total_count` is the untruncated
    /// count used for displaying "N more" in truncated output.
    ///
    /// Matches edges where `to_id` is either:
    /// - The exact symbol ID (e.g. `src/payments/service.ts::PaymentService::class`)
    /// - The bare symbol name (e.g. `PaymentService`)
    /// - A relative-path qualified name ending with `::SymbolName`
    pub fn find_refs(
        &self,
        symbol_name: &str,
        kinds: Option<&[&str]>,
        limit: usize,
    ) -> Result<(Vec<Reference>, usize)> {
        let symbol = self.find_symbol(symbol_name)?.ok_or_else(|| {
            anyhow::anyhow!(
                "Symbol '{}' not found in index.\n\
                 Tip: Check spelling, or use 'sc find \"{}\"' for semantic search.",
                symbol_name,
                symbol_name
            )
        })?;

        // Collect all names to match against to_id
        let mut match_names = vec![symbol.name.clone(), symbol.id.clone()];

        // For classes, also include child method names
        if symbol.kind == "class" || symbol.kind == "struct" || symbol.kind == "interface" {
            let methods = self.get_methods(&symbol.id)?;
            for m in &methods {
                match_names.push(m.name.clone());
                match_names.push(m.id.clone());
            }
        }

        // Build the to_id matching clause:
        // Match exact name, exact ID, or to_id ending with ::Name
        let match_conditions = self.build_to_id_match_clause(&match_names, 1);
        let next_param = match_names.len() * 2 + 1; // each name uses 2 params (exact + LIKE)

        let (kind_clause, kind_values): (String, Vec<String>) = if let Some(k) = kinds {
            let placeholders: Vec<String> = (next_param..next_param + k.len())
                .map(|i| format!("?{i}"))
                .collect();
            (
                format!("AND e.kind IN ({})", placeholders.join(", ")),
                k.iter().map(|s| s.to_string()).collect(),
            )
        } else {
            (String::new(), Vec::new())
        };

        let mut param_values: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        for name in &match_names {
            param_values.push(Box::new(name.clone()));
            param_values.push(Box::new(format!("%::{name}")));
        }
        for kv in &kind_values {
            param_values.push(Box::new(kv.clone()));
        }

        // Count total
        let count_sql =
            format!("SELECT COUNT(*) FROM edges e WHERE ({match_conditions}) {kind_clause}");
        let mut count_stmt = self.conn.prepare(&count_sql)?;
        let params_ref: Vec<&dyn rusqlite::ToSql> = param_values
            .iter()
            .map(|b| b.as_ref() as &dyn rusqlite::ToSql)
            .collect();
        let total: i64 = count_stmt.query_row(params_ref.as_slice(), |row| row.get(0))?;
        let total = total as usize;

        // Fetch refs with limit
        let limit_idx = param_values.len() + 1;
        let fetch_sql = format!(
            "SELECT e.from_id, e.kind, e.file_path, e.line
             FROM edges e
             WHERE ({match_conditions}) {kind_clause}
             ORDER BY e.kind, e.file_path, e.line
             LIMIT ?{limit_idx}"
        );
        let mut stmt = self.conn.prepare(&fetch_sql)?;

        let mut fetch_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        for name in &match_names {
            fetch_params.push(Box::new(name.clone()));
            fetch_params.push(Box::new(format!("%::{name}")));
        }
        for kv in &kind_values {
            fetch_params.push(Box::new(kv.clone()));
        }
        fetch_params.push(Box::new(limit as i64));
        let fetch_ref: Vec<&dyn rusqlite::ToSql> = fetch_params
            .iter()
            .map(|b| b.as_ref() as &dyn rusqlite::ToSql)
            .collect();

        let rows = stmt.query_map(fetch_ref.as_slice(), |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<i64>>(3)?,
            ))
        })?;

        let mut refs = Vec::new();
        for row in rows {
            let (from_id, kind, file_path, line) = row?;
            let context = self.caller_display_name(&from_id);
            let from_name = self.symbol_name_from_id(&from_id);
            refs.push(Reference {
                from_id,
                from_name,
                kind,
                file_path,
                line,
                context,
            });
        }

        Ok((refs, total))
    }

    /// Build a SQL clause matching `to_id` against a set of symbol names.
    ///
    /// For each name, matches: `e.to_id = ?N OR e.to_id LIKE ?N` (pattern `%::Name`).
    /// `start_param` is the 1-based parameter index to begin with.
    fn build_to_id_match_clause(&self, names: &[String], start_param: usize) -> String {
        let mut conditions = Vec::new();
        let mut idx = start_param;
        for _name in names {
            conditions.push(format!("e.to_id = ?{idx} OR e.to_id LIKE ?{}", idx + 1));
            idx += 2;
        }
        conditions.join(" OR ")
    }

    /// Find references to a symbol, grouped by kind.
    ///
    /// Used for class symbols where refs should be displayed in groups
    /// (instantiated, extended, used as type, imported).
    #[allow(clippy::type_complexity)]
    pub fn find_refs_grouped(
        &self,
        symbol_name: &str,
        limit: usize,
    ) -> Result<(Vec<(String, Vec<Reference>)>, usize)> {
        let (refs, total) = self.find_refs(symbol_name, None, limit)?;

        // Group by kind, preserving insertion order
        let mut groups: Vec<(String, Vec<Reference>)> = Vec::new();
        for r in refs {
            if let Some(group) = groups.iter_mut().find(|(k, _)| *k == r.kind) {
                group.1.push(r);
            } else {
                let kind = r.kind.clone();
                groups.push((kind, vec![r]));
            }
        }

        Ok((groups, total))
    }

    /// Find all references to symbols in a file.
    ///
    /// Aggregates refs to every symbol defined in the given file path.
    pub fn find_file_refs(
        &self,
        file_path: &str,
        kinds: Option<&[&str]>,
        limit: usize,
    ) -> Result<(Vec<Reference>, usize)> {
        let symbols = self.get_file_symbols(file_path)?;
        if symbols.is_empty() {
            anyhow::bail!(
                "No symbols found for file '{}'.\n\
                 Tip: Check the path is relative to the project root. Run 'sc index' if the file is new.",
                file_path
            );
        }

        // Collect all names and IDs to match against to_id
        let mut match_names: Vec<String> = Vec::new();
        for sym in &symbols {
            match_names.push(sym.name.clone());
            match_names.push(sym.id.clone());
        }

        let match_conditions = self.build_to_id_match_clause(&match_names, 1);
        let next_param = match_names.len() * 2 + 1;

        let (kind_clause, kind_values): (String, Vec<String>) = if let Some(k) = kinds {
            let placeholders: Vec<String> = (next_param..next_param + k.len())
                .map(|i| format!("?{i}"))
                .collect();
            (
                format!("AND e.kind IN ({})", placeholders.join(", ")),
                k.iter().map(|s| s.to_string()).collect(),
            )
        } else {
            (String::new(), Vec::new())
        };

        let mut param_values: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        for name in &match_names {
            param_values.push(Box::new(name.clone()));
            param_values.push(Box::new(format!("%::{name}")));
        }
        for kv in &kind_values {
            param_values.push(Box::new(kv.clone()));
        }

        // Count
        let count_sql =
            format!("SELECT COUNT(*) FROM edges e WHERE ({match_conditions}) {kind_clause}");
        let mut count_stmt = self.conn.prepare(&count_sql)?;
        let params_ref: Vec<&dyn rusqlite::ToSql> = param_values
            .iter()
            .map(|b| b.as_ref() as &dyn rusqlite::ToSql)
            .collect();
        let total: i64 = count_stmt.query_row(params_ref.as_slice(), |row| row.get(0))?;
        let total = total as usize;

        // Fetch
        let limit_idx = param_values.len() + 1;
        let fetch_sql = format!(
            "SELECT e.from_id, e.kind, e.file_path, e.line
             FROM edges e
             WHERE ({match_conditions}) {kind_clause}
             ORDER BY e.kind, e.file_path, e.line
             LIMIT ?{limit_idx}"
        );
        let mut stmt = self.conn.prepare(&fetch_sql)?;
        let mut fetch_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        for name in &match_names {
            fetch_params.push(Box::new(name.clone()));
            fetch_params.push(Box::new(format!("%::{name}")));
        }
        for kv in &kind_values {
            fetch_params.push(Box::new(kv.clone()));
        }
        fetch_params.push(Box::new(limit as i64));
        let fetch_ref: Vec<&dyn rusqlite::ToSql> = fetch_params
            .iter()
            .map(|b| b.as_ref() as &dyn rusqlite::ToSql)
            .collect();

        let rows = stmt.query_map(fetch_ref.as_slice(), |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<i64>>(3)?,
            ))
        })?;

        let mut refs = Vec::new();
        for row in rows {
            let (from_id, kind, file_path, line) = row?;
            let context = self.caller_display_name(&from_id);
            let from_name = self.symbol_name_from_id(&from_id);
            refs.push(Reference {
                from_id,
                from_name,
                kind,
                file_path,
                line,
                context,
            });
        }

        Ok((refs, total))
    }

    /// Find dependencies of a symbol (outgoing edges).
    ///
    /// For depth 1: returns direct dependencies.
    /// For depth > 1: uses a recursive CTE to traverse transitive dependencies.
    /// For classes: includes dependencies from all child methods.
    ///
    /// Also includes edges from the `__module__` synthetic node for the symbol's
    /// file, since tree-sitter extractors often attribute edges to the module level.
    pub fn find_deps(&self, symbol_name: &str, max_depth: usize) -> Result<Vec<Dependency>> {
        let symbol = self.find_symbol(symbol_name)?.ok_or_else(|| {
            anyhow::anyhow!(
                "Symbol '{}' not found in index.\n\
                 Tip: Check spelling, or use 'sc find \"{}\"' for semantic search.",
                symbol_name,
                symbol_name
            )
        })?;

        // Collect source IDs: symbol itself, child methods, and __module__ synthetic IDs
        let mut source_ids = vec![symbol.id.clone()];
        if symbol.kind == "class" || symbol.kind == "struct" || symbol.kind == "interface" {
            let methods = self.get_methods(&symbol.id)?;
            for m in &methods {
                source_ids.push(m.id.clone());
            }
        }

        // Also include the __module__ synthetic ID for the symbol's file,
        // since many edges use it as from_id
        let module_id = format!("{}::__module__::function", symbol.file_path);
        if !source_ids.contains(&module_id) {
            source_ids.push(module_id);
        }

        if max_depth <= 1 {
            self.find_direct_deps(&source_ids)
        } else {
            self.find_transitive_deps(&source_ids, max_depth)
        }
    }

    /// Find dependencies of all symbols in a file.
    pub fn find_file_deps(&self, file_path: &str, max_depth: usize) -> Result<Vec<Dependency>> {
        let symbols = self.get_file_symbols(file_path)?;
        if symbols.is_empty() {
            anyhow::bail!(
                "No symbols found for file '{}'.\n\
                 Tip: Check the path is relative to the project root. Run 'sc index' if the file is new.",
                file_path
            );
        }

        let mut source_ids: Vec<String> = symbols.iter().map(|s| s.id.clone()).collect();

        // Also include the __module__ synthetic ID for this file
        let module_id = format!("{file_path}::__module__::function");
        if !source_ids.contains(&module_id) {
            source_ids.push(module_id);
        }

        if max_depth <= 1 {
            self.find_direct_deps(&source_ids)
        } else {
            self.find_transitive_deps(&source_ids, max_depth)
        }
    }

    /// Get direct (depth-1) dependencies from a set of source symbol IDs.
    fn find_direct_deps(&self, source_ids: &[String]) -> Result<Vec<Dependency>> {
        let placeholders: Vec<String> = (1..=source_ids.len()).map(|i| format!("?{i}")).collect();
        let id_clause = placeholders.join(", ");

        let sql = format!(
            "SELECT DISTINCT e.to_id, e.kind
             FROM edges e
             WHERE e.from_id IN ({id_clause})
             ORDER BY e.kind, e.to_id"
        );

        let mut stmt = self.conn.prepare(&sql)?;
        let params: Vec<&dyn rusqlite::ToSql> = source_ids
            .iter()
            .map(|id| id as &dyn rusqlite::ToSql)
            .collect();

        let rows = stmt.query_map(params.as_slice(), |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut deps = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for row in rows {
            let (to_id, kind) = row?;

            // Skip self-references
            if source_ids.contains(&to_id) {
                continue;
            }

            // Dedup by (name, kind) to avoid listing same dep multiple times
            let name = self.symbol_name_from_id(&to_id);
            let key = format!("{name}::{kind}");
            if !seen.insert(key) {
                continue;
            }

            // Check if the dep exists in the index — try by ID first, then by name
            let sym_info = self.resolve_dep_symbol(&to_id, &name)?;

            let (dep_name, file_path, is_external) = match sym_info {
                Some((n, fp)) => (n, Some(fp), false),
                None => (name, None, true),
            };

            deps.push(Dependency {
                name: dep_name,
                file_path,
                kind,
                is_external,
                depth: 1,
            });
        }

        Ok(deps)
    }

    /// Resolve a dependency target to a symbol in the index.
    ///
    /// Tries: exact ID match, then name match (for relative-path style to_ids).
    fn resolve_dep_symbol(
        &self,
        to_id: &str,
        extracted_name: &str,
    ) -> Result<Option<(String, String)>> {
        // Try exact ID match
        let by_id: Option<(String, String)> = self
            .conn
            .query_row(
                "SELECT name, file_path FROM symbols WHERE id = ?1",
                params![to_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()?;

        if by_id.is_some() {
            return Ok(by_id);
        }

        // Try by name — prefer top-level symbols (no parent)
        let by_name: Option<(String, String)> = self
            .conn
            .query_row(
                "SELECT name, file_path FROM symbols WHERE name = ?1
                 ORDER BY (CASE WHEN parent_id IS NULL THEN 0 ELSE 1 END)
                 LIMIT 1",
                params![extracted_name],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()?;

        Ok(by_name)
    }

    /// Get transitive dependencies using a recursive CTE.
    fn find_transitive_deps(
        &self,
        source_ids: &[String],
        max_depth: usize,
    ) -> Result<Vec<Dependency>> {
        // We need a temp table approach since CTEs can't easily take dynamic IN clauses
        // for recursive seeds. Instead, build the seed UNION for all source IDs.
        let seed_conditions: Vec<String> = (1..=source_ids.len())
            .map(|i| format!("SELECT e.to_id, e.kind, 1 FROM edges e WHERE e.from_id = ?{i}"))
            .collect();
        let seed_union = seed_conditions.join(" UNION ALL ");

        let depth_param_idx = source_ids.len() + 1;
        let sql = format!(
            "WITH RECURSIVE deps(id, kind, depth) AS (
                {seed_union}
                UNION
                SELECT e.to_id, e.kind, d.depth + 1
                FROM edges e
                JOIN deps d ON e.from_id = d.id
                WHERE d.depth < ?{depth_param_idx}
            )
            SELECT DISTINCT d.id, d.kind, MIN(d.depth) as min_depth
            FROM deps d
            GROUP BY d.id, d.kind
            ORDER BY min_depth, d.kind, d.id"
        );

        let mut stmt = self.conn.prepare(&sql)?;
        let mut param_values: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        for id in source_ids {
            param_values.push(Box::new(id.clone()));
        }
        param_values.push(Box::new(max_depth as i64));
        let params_ref: Vec<&dyn rusqlite::ToSql> = param_values
            .iter()
            .map(|b| b.as_ref() as &dyn rusqlite::ToSql)
            .collect();

        let rows = stmt.query_map(params_ref.as_slice(), |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })?;

        let mut deps = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for row in rows {
            let (to_id, kind, depth) = row?;

            // Skip self-references
            if source_ids.contains(&to_id) {
                continue;
            }

            let name = self.symbol_name_from_id(&to_id);
            let key = format!("{name}::{kind}");
            if !seen.insert(key) {
                continue;
            }

            let sym_info = self.resolve_dep_symbol(&to_id, &name)?;

            let (dep_name, file_path, is_external) = match sym_info {
                Some((n, fp)) => (n, Some(fp), false),
                None => (name, None, true),
            };

            deps.push(Dependency {
                name: dep_name,
                file_path,
                kind,
                is_external,
                depth: depth as usize,
            });
        }

        Ok(deps)
    }

    /// Check if a symbol is a class (or struct/interface — types that get grouped refs).
    pub fn is_class_like(&self, symbol_name: &str) -> Result<bool> {
        let symbol = self.find_symbol(symbol_name)?;
        Ok(symbol
            .map(|s| s.kind == "class" || s.kind == "struct" || s.kind == "interface")
            .unwrap_or(false))
    }

    /// Find the transitive impact (blast radius) of changing a symbol.
    ///
    /// Performs a recursive reverse dependency traversal: finds all symbols
    /// that directly or transitively depend on the given symbol. Results are
    /// grouped by depth and test files are separated.
    ///
    /// Uses the same name-matching pattern as `find_refs` (exact name, exact
    /// ID, or `LIKE '%::Name'`) to match `to_id` in the edges table.
    pub fn find_impact(&self, symbol_name: &str, max_depth: usize) -> Result<ImpactResult> {
        let symbol = self.find_symbol(symbol_name)?.ok_or_else(|| {
            anyhow::anyhow!(
                "Symbol '{}' not found in index.\n\
                 Tip: Check spelling, or use 'sc find \"{}\"' for semantic search.",
                symbol_name,
                symbol_name
            )
        })?;

        // Collect all IDs to seed the impact traversal
        let mut seed_ids = vec![symbol.id.clone()];

        // For classes, also include child methods as seeds
        if symbol.kind == "class" || symbol.kind == "struct" || symbol.kind == "interface" {
            let methods = self.get_methods(&symbol.id)?;
            for m in &methods {
                seed_ids.push(m.id.clone());
            }
        }

        self.run_impact_query(&seed_ids, max_depth)
    }

    /// Find the impact of changing any symbol in a file.
    ///
    /// Collects all symbols in the file and runs impact analysis for each,
    /// deduplicating results.
    pub fn find_file_impact(&self, file_path: &str, max_depth: usize) -> Result<ImpactResult> {
        let symbols = self.get_file_symbols(file_path)?;
        if symbols.is_empty() {
            anyhow::bail!(
                "No symbols found for file '{}'.\n\
                 Tip: Check the path is relative to the project root. Run 'sc index' if the file is new.",
                file_path
            );
        }

        let seed_ids: Vec<String> = symbols.iter().map(|s| s.id.clone()).collect();
        self.run_impact_query(&seed_ids, max_depth)
    }

    /// Execute the recursive CTE impact query for a set of seed symbol IDs.
    fn run_impact_query(&self, seed_ids: &[String], max_depth: usize) -> Result<ImpactResult> {
        // Build seed conditions: for each seed ID, match edges where
        // to_id equals the ID exactly, matches the name, or ends with ::Name
        let mut seed_unions: Vec<String> = Vec::new();
        let mut param_values: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        let mut idx = 1usize;

        for seed_id in seed_ids {
            // Extract the bare name from the ID
            let bare_name = self.symbol_name_from_id(seed_id);
            let like_pattern = format!("%::{bare_name}");

            seed_unions.push(format!(
                "SELECT e.from_id, 1, CAST(e.from_id AS TEXT) \
                 FROM edges e WHERE (e.to_id = ?{idx} OR e.to_id = ?{} OR e.to_id LIKE ?{})",
                idx + 1,
                idx + 2
            ));
            param_values.push(Box::new(seed_id.clone()));
            param_values.push(Box::new(bare_name));
            param_values.push(Box::new(like_pattern));
            idx += 3;
        }

        let seed_sql = seed_unions.join(" UNION ALL ");
        let depth_param = idx;
        param_values.push(Box::new(max_depth as i64));

        let sql = format!(
            "WITH RECURSIVE impact(id, depth, path) AS (
                {seed_sql}
                UNION ALL
                SELECT e.from_id, i.depth + 1, i.path || ',' || e.from_id
                FROM edges e
                JOIN impact i ON e.to_id = i.id
                WHERE i.depth < ?{depth_param}
                  AND (',' || i.path || ',') NOT LIKE '%,' || e.from_id || ',%'
            )
            SELECT DISTINCT i.id, MIN(i.depth) as min_depth, s.name, s.file_path, s.kind
            FROM impact i
            JOIN symbols s ON s.id = i.id
            GROUP BY i.id
            ORDER BY min_depth, s.file_path"
        );

        let mut stmt = self.conn.prepare(&sql)?;
        let params_ref: Vec<&dyn rusqlite::ToSql> = param_values
            .iter()
            .map(|b| b.as_ref() as &dyn rusqlite::ToSql)
            .collect();

        let rows = stmt.query_map(params_ref.as_slice(), |row| {
            Ok(ImpactNode {
                id: row.get(0)?,
                depth: row.get::<_, i64>(1)? as usize,
                name: row.get(2)?,
                file_path: row.get(3)?,
                kind: row.get(4)?,
            })
        })?;

        let mut all_nodes: Vec<ImpactNode> = Vec::new();
        for row in rows {
            let node = row?;
            // Skip seed IDs from appearing in the results
            if seed_ids.contains(&node.id) {
                continue;
            }
            all_nodes.push(node);
        }

        // Separate test files from non-test files
        let mut test_files: Vec<ImpactNode> = Vec::new();
        let mut non_test_nodes: Vec<ImpactNode> = Vec::new();

        for node in all_nodes {
            if is_test_file(&node.file_path) {
                test_files.push(node);
            } else {
                non_test_nodes.push(node);
            }
        }

        let total_affected = non_test_nodes.len();

        // Group non-test nodes by depth
        let mut depth_map: std::collections::BTreeMap<usize, Vec<ImpactNode>> =
            std::collections::BTreeMap::new();
        for node in non_test_nodes {
            depth_map.entry(node.depth).or_default().push(node);
        }

        let nodes_by_depth: Vec<(usize, Vec<ImpactNode>)> = depth_map.into_iter().collect();

        Ok(ImpactResult {
            nodes_by_depth,
            test_files,
            total_affected,
        })
    }

    /// Insert a batch of symbols and edges within a single transaction.
    ///
    /// Used during indexing to efficiently store all extracted data for a file.
    pub fn insert_file_data(
        &mut self,
        file_path: &str,
        symbols: &[Symbol],
        edges: &[Edge],
    ) -> Result<()> {
        let tx = self.conn.transaction()?;

        // Delete existing data for this file
        tx.execute("DELETE FROM edges WHERE file_path = ?1", params![file_path])?;
        tx.execute(
            "DELETE FROM symbols WHERE file_path = ?1",
            params![file_path],
        )?;

        // Insert symbols
        {
            let mut stmt = tx.prepare(
                "INSERT INTO symbols
                 (id, name, kind, file_path, line_start, line_end, signature, docstring, parent_id, language, metadata)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            )?;

            for symbol in symbols {
                stmt.execute(params![
                    symbol.id,
                    symbol.name,
                    symbol.kind,
                    symbol.file_path,
                    symbol.line_start,
                    symbol.line_end,
                    symbol.signature,
                    symbol.docstring,
                    symbol.parent_id,
                    symbol.language,
                    symbol.metadata,
                ])?;
            }
        }

        // Insert edges
        {
            let mut stmt = tx.prepare(
                "INSERT OR IGNORE INTO edges (from_id, to_id, kind, file_path, line)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )?;

            for edge in edges {
                stmt.execute(params![
                    edge.from_id,
                    edge.to_id,
                    edge.kind,
                    edge.file_path,
                    edge.line,
                ])?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    /// Delete all symbols, edges, and file hash data for a given file path.
    pub fn delete_file_data(&mut self, file_path: &str) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM edges WHERE file_path = ?1", params![file_path])?;
        tx.execute(
            "DELETE FROM symbols WHERE file_path = ?1",
            params![file_path],
        )?;
        tx.execute(
            "DELETE FROM file_hashes WHERE file_path = ?1",
            params![file_path],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Clear all data from the graph (used before a full re-index).
    pub fn clear_all(&mut self) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM edges", [])?;
        tx.execute("DELETE FROM symbols", [])?;
        tx.execute("DELETE FROM file_hashes", [])?;
        tx.commit()?;
        Ok(())
    }

    /// Get the total number of symbols in the index.
    pub fn symbol_count(&self) -> Result<usize> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM symbols", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    /// Get the total number of indexed files.
    pub fn file_count(&self) -> Result<usize> {
        let count: i64 =
            self.conn
                .query_row("SELECT COUNT(DISTINCT file_path) FROM symbols", [], |row| {
                    row.get(0)
                })?;
        Ok(count as usize)
    }

    /// Get the total number of edges in the index.
    pub fn edge_count(&self) -> Result<usize> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM edges", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    // -- File hash operations --

    /// Compare current file hashes against the stored index to find changes.
    pub fn get_changed_files(
        &self,
        current_hashes: &HashMap<String, String>,
    ) -> Result<ChangedFiles> {
        let mut changed = ChangedFiles::default();

        // Load stored hashes
        let stored: HashMap<String, String> = {
            let mut stmt = self
                .conn
                .prepare("SELECT file_path, hash FROM file_hashes")?;
            let rows: Vec<(String, String)> = stmt
                .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
                .filter_map(|r| r.ok())
                .collect();
            rows.into_iter().collect()
        };

        for (path, hash) in current_hashes {
            match stored.get(path) {
                None => changed.added.push(path.clone()),
                Some(old_hash) if old_hash != hash => changed.modified.push(path.clone()),
                _ => {} // unchanged
            }
        }

        for path in stored.keys() {
            if !current_hashes.contains_key(path) {
                changed.deleted.push(path.clone());
            }
        }

        Ok(changed)
    }

    /// Update the stored file hashes after indexing.
    pub fn update_file_hashes(&mut self, hashes: &HashMap<String, String>) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;

        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO file_hashes (file_path, hash, indexed_at)
                 VALUES (?1, ?2, ?3)",
            )?;

            for (path, hash) in hashes {
                stmt.execute(params![path, hash, now])?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    /// Delete a file hash entry.
    pub fn delete_file_hash(&self, file_path: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM file_hashes WHERE file_path = ?1",
            params![file_path],
        )?;
        Ok(())
    }
}

/// Check if a file path belongs to a test file.
///
/// Heuristic: returns `true` if the lowercase path contains common test path
/// segments or test file naming patterns.
pub fn is_test_file(file_path: &str) -> bool {
    let lower = file_path.to_lowercase().replace('\\', "/");
    lower.contains("/test/")
        || lower.contains("/tests/")
        || lower.contains(".test.")
        || lower.contains(".spec.")
        || lower.contains("_test.")
        || lower.contains("_spec.")
        || lower.starts_with("test/")
        || lower.starts_with("tests/")
}
