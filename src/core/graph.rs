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
