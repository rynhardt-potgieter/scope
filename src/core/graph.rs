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
    pub fn find_symbol(&self, name: &str) -> Result<Option<Symbol>> {
        // Try exact match first
        let result = self
            .conn
            .query_row(
                "SELECT * FROM symbols WHERE name = ?1 LIMIT 1",
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
