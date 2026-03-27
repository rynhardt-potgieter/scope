//! Full-text search over symbol embeddings using SQLite FTS5.
//!
//! Provides BM25-ranked search for `scope find`. Symbols are indexed with
//! their name, signature, docstring, and parent context. Queries use
//! FTS5 MATCH syntax with porter stemming and unicode tokenisation.
//!
//! This is the MVP search backend. A future iteration can swap in
//! LanceDB + vector embeddings for true semantic search while keeping
//! the same `SearchResult` interface.

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

use crate::core::embedder::build_embedding_text;
use crate::core::graph::Symbol;

/// A single result from a search query.
#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    /// Symbol ID (matches `symbols.id` in the graph).
    pub id: String,
    /// Symbol display name.
    pub name: String,
    /// File path relative to project root.
    pub file_path: String,
    /// Symbol kind (function, class, method, etc.).
    pub kind: String,
    /// Relevance score: 0.0-1.0, higher = more relevant.
    pub score: f64,
    /// Start line of the symbol definition.
    pub line_start: u32,
    /// End line of the symbol definition.
    pub line_end: u32,
}

/// FTS5-backed full-text search engine for symbols.
///
/// Operates on the same SQLite database as the graph, using the
/// `symbols_fts` virtual table created in `schema.sql`.
pub struct Searcher {
    conn: Connection,
}

impl Searcher {
    /// Open a searcher on the graph database at the given path.
    ///
    /// The database must already have the FTS5 table (created by `ensure_schema`).
    pub fn open(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path)
            .with_context(|| format!("Failed to open search index at {}", db_path.display()))?;

        // Apply same performance pragmas as the graph
        conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA cache_size = -64000;
            PRAGMA temp_store = MEMORY;
            ",
        )?;

        Ok(Self { conn })
    }

    /// Index a batch of symbols into the FTS5 table.
    ///
    /// Builds the embedding text for each symbol and inserts it.
    /// Caller and callee maps provide relationship context for richer search.
    /// Call this after parsing symbols and edges from source files.
    pub fn index_symbols(
        &self,
        symbols: &[Symbol],
        callers: &HashMap<String, Vec<String>>,
        callees: &HashMap<String, Vec<String>>,
        importance: &HashMap<String, f64>,
    ) -> Result<()> {
        let mut stmt = self.conn.prepare(
            "INSERT OR REPLACE INTO symbols_fts (symbol_id, name, kind, file_path, body)
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;

        let empty: Vec<String> = Vec::new();
        for symbol in symbols {
            let sym_callers = callers.get(&symbol.id).unwrap_or(&empty);
            let sym_callees = callees.get(&symbol.id).unwrap_or(&empty);
            let imp = importance.get(&symbol.id).copied().unwrap_or(0.0);
            let body = build_embedding_text(symbol, sym_callers, sym_callees, imp);
            stmt.execute(params![
                symbol.id,
                symbol.name,
                symbol.kind,
                symbol.file_path,
                body,
            ])?;
        }

        Ok(())
    }

    /// Delete all FTS entries for a given file path.
    ///
    /// Used during incremental indexing when a file is removed or re-indexed.
    pub fn delete_file(&self, file_path: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM symbols_fts WHERE file_path = ?1",
            params![file_path],
        )?;
        Ok(())
    }

    /// Clear all FTS data (used before a full re-index).
    pub fn clear_all(&self) -> Result<()> {
        self.conn.execute("DELETE FROM symbols_fts", [])?;
        Ok(())
    }

    /// Search with vendor de-ranking applied.
    ///
    /// First-party results appear before vendor results; within each
    /// partition the original BM25 relevance order is preserved.
    pub fn search_with_vendor_derank(
        &self,
        query: &str,
        limit: usize,
        kind_filter: Option<&str>,
        vendor_patterns: &[String],
    ) -> Result<Vec<SearchResult>> {
        let results = self.search(query, limit, kind_filter)?;

        if vendor_patterns.is_empty() {
            return Ok(results);
        }

        let (first_party, vendor): (Vec<_>, Vec<_>) = results
            .into_iter()
            .partition(|r| !crate::config::project::is_vendor_path(&r.file_path, vendor_patterns));

        let mut combined = first_party;
        combined.extend(vendor);
        Ok(combined)
    }

    /// Search for symbols matching a natural-language query.
    ///
    /// Uses FTS5 MATCH with BM25 ranking. The query is automatically
    /// converted to an OR query so that partial matches still surface.
    /// Results are ranked by relevance and optionally filtered by kind.
    pub fn search(
        &self,
        query: &str,
        limit: usize,
        kind_filter: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        let fts_query = build_fts_query(query);

        if fts_query.is_empty() {
            return Ok(Vec::new());
        }

        // Build SQL with optional kind filter
        let (sql, has_kind_filter) = if kind_filter.is_some() {
            (
                "SELECT symbol_id, name, kind, file_path,
                        bm25(symbols_fts, 0.0, 5.0, 0.0, 2.0, 10.0) AS rank
                 FROM symbols_fts
                 WHERE symbols_fts MATCH ?1 AND kind = ?3
                 ORDER BY rank
                 LIMIT ?2",
                true,
            )
        } else {
            (
                "SELECT symbol_id, name, kind, file_path,
                        bm25(symbols_fts, 0.0, 5.0, 0.0, 2.0, 10.0) AS rank
                 FROM symbols_fts
                 WHERE symbols_fts MATCH ?1
                 ORDER BY rank
                 LIMIT ?2",
                false,
            )
        };

        let mut stmt = self.conn.prepare(sql)?;

        let raw_results = if has_kind_filter {
            let rows = stmt.query_map(
                params![fts_query, limit as i64, kind_filter.unwrap_or("")],
                map_fts_row,
            )?;
            rows.collect::<std::result::Result<Vec<_>, _>>()?
        } else {
            let rows = stmt.query_map(params![fts_query, limit as i64], map_fts_row)?;
            rows.collect::<std::result::Result<Vec<_>, _>>()?
        };

        // Convert BM25 ranks to 0.0-1.0 scores (BM25 returns negative values; lower = better)
        let results = normalize_scores(raw_results);

        // Enrich with line numbers from the symbols table
        let enriched = self.enrich_with_lines(&results)?;

        Ok(enriched)
    }

    /// Look up line_start and line_end for each search result from the symbols table.
    fn enrich_with_lines(&self, results: &[SearchResult]) -> Result<Vec<SearchResult>> {
        let mut enriched = Vec::new();
        let mut stmt = self
            .conn
            .prepare("SELECT line_start, line_end FROM symbols WHERE id = ?1")?;

        for result in results {
            let lines = stmt
                .query_row(params![result.id], |row| {
                    Ok((row.get::<_, u32>(0)?, row.get::<_, u32>(1)?))
                })
                .ok();

            let (line_start, line_end) = lines.unwrap_or((result.line_start, result.line_end));

            enriched.push(SearchResult {
                id: result.id.clone(),
                name: result.name.clone(),
                file_path: result.file_path.clone(),
                kind: result.kind.clone(),
                score: result.score,
                line_start,
                line_end,
            });
        }

        Ok(enriched)
    }
}

/// Map a rusqlite row to a `RawFtsResult`.
///
/// Used as a shared closure for `query_map` to avoid closure type mismatch.
fn map_fts_row(row: &rusqlite::Row) -> rusqlite::Result<RawFtsResult> {
    Ok(RawFtsResult {
        symbol_id: row.get(0)?,
        name: row.get(1)?,
        kind: row.get(2)?,
        file_path: row.get(3)?,
        rank: row.get(4)?,
    })
}

/// Raw FTS5 result before score normalisation.
struct RawFtsResult {
    symbol_id: String,
    name: String,
    kind: String,
    file_path: String,
    rank: f64,
}

/// Convert BM25 rank values to 0.0-1.0 similarity scores.
///
/// BM25 returns negative values where lower (more negative) = better match.
/// We invert and normalise so that higher = more relevant.
fn normalize_scores(raw: Vec<RawFtsResult>) -> Vec<SearchResult> {
    if raw.is_empty() {
        return Vec::new();
    }

    // Find the range of ranks for normalisation
    let min_rank = raw.iter().map(|r| r.rank).fold(f64::INFINITY, f64::min);
    let max_rank = raw.iter().map(|r| r.rank).fold(f64::NEG_INFINITY, f64::max);

    let range = max_rank - min_rank;

    raw.into_iter()
        .map(|r| {
            // Normalise to 0.0-1.0 where best match = 1.0
            let score = if range.abs() < f64::EPSILON {
                // All results have the same rank — give them all a high score
                0.95
            } else {
                // Invert: best (most negative) rank -> highest score
                // Map [min_rank, max_rank] to [1.0, 0.5] — even worst match gets 0.5
                let normalised = (r.rank - min_rank) / range;
                1.0 - (normalised * 0.5)
            };

            SearchResult {
                id: r.symbol_id,
                name: r.name,
                file_path: r.file_path,
                kind: r.kind,
                score,
                line_start: 0,
                line_end: 0,
            }
        })
        .collect()
}

/// Build an FTS5 query from a natural-language search string.
///
/// Splits the query into tokens and joins them with OR for partial matching.
/// Each token gets a `*` suffix for prefix matching, and camelCase tokens
/// are additionally split into component words for broader recall.
///
/// Examples:
/// - `"TransactionController"` -> `TransactionController* OR transaction* OR controller*`
/// - `"payment"` -> `payment*`
/// - `"authentication errors"` -> `authentication* OR errors*`
fn build_fts_query(query: &str) -> String {
    let tokens: Vec<&str> = query.split_whitespace().filter(|t| !t.is_empty()).collect();

    if tokens.is_empty() {
        return String::new();
    }

    let mut fts_tokens: Vec<String> = Vec::new();

    for token in &tokens {
        let cleaned: String = token
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if cleaned.is_empty() {
            continue;
        }

        // Add the full token with prefix matching
        fts_tokens.push(format!("{cleaned}*"));

        // Also split camelCase and add component words (min 3 chars)
        let split = crate::core::embedder::split_camel_case(&cleaned);
        for word in split.split_whitespace() {
            let lower = word.to_lowercase();
            if lower != cleaned.to_lowercase() && lower.len() >= 3 {
                fts_tokens.push(format!("{lower}*"));
            }
        }

        // Also split snake_case and add component words
        if cleaned.contains('_') {
            for word in crate::core::embedder::split_snake_case(&cleaned).split_whitespace() {
                let lower = word.to_lowercase();
                if lower.len() >= 3 {
                    fts_tokens.push(format!("{lower}*"));
                }
            }
        }
    }

    fts_tokens.dedup();
    fts_tokens.join(" OR ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_fts_query_simple() {
        assert_eq!(
            build_fts_query("authentication errors"),
            "authentication* OR errors*"
        );
    }

    #[test]
    fn test_build_fts_query_single_word() {
        assert_eq!(build_fts_query("payment"), "payment*");
    }

    #[test]
    fn test_build_fts_query_empty() {
        assert_eq!(build_fts_query(""), "");
        assert_eq!(build_fts_query("   "), "");
    }

    #[test]
    fn test_build_fts_query_special_chars() {
        assert_eq!(build_fts_query("foo-bar baz"), "foobar* OR baz*");
    }

    #[test]
    fn test_build_fts_query_camel_case_splitting() {
        assert_eq!(
            build_fts_query("TransactionController"),
            "TransactionController* OR transaction* OR controller*"
        );
    }

    #[test]
    fn test_build_fts_query_snake_case_splitting() {
        let query = build_fts_query("payment_retry");
        assert!(query.contains("payment_retry*"));
        assert!(query.contains("payment*"));
        assert!(query.contains("retry*"));
    }

    #[test]
    fn test_build_fts_query_camel_case_no_short_words() {
        // Short component words (< 3 chars) should be excluded
        assert_eq!(build_fts_query("GoTo"), "GoTo*");
    }

    #[test]
    fn test_normalize_scores_empty() {
        let results = normalize_scores(Vec::new());
        assert!(results.is_empty());
    }

    #[test]
    fn test_normalize_scores_single() {
        let raw = vec![RawFtsResult {
            symbol_id: "id1".to_string(),
            name: "foo".to_string(),
            kind: "function".to_string(),
            file_path: "test.ts".to_string(),
            rank: -5.0,
        }];
        let results = normalize_scores(raw);
        assert_eq!(results.len(), 1);
        assert!((results[0].score - 0.95).abs() < 0.01);
    }

    #[test]
    fn test_normalize_scores_multiple() {
        let raw = vec![
            RawFtsResult {
                symbol_id: "id1".to_string(),
                name: "best".to_string(),
                kind: "function".to_string(),
                file_path: "test.ts".to_string(),
                rank: -10.0, // best match (most negative)
            },
            RawFtsResult {
                symbol_id: "id2".to_string(),
                name: "worst".to_string(),
                kind: "function".to_string(),
                file_path: "test.ts".to_string(),
                rank: -2.0, // worst match (least negative)
            },
        ];
        let results = normalize_scores(raw);
        assert_eq!(results.len(), 2);
        // Best match should have score = 1.0
        assert!((results[0].score - 1.0).abs() < 0.01);
        // Worst match should have score = 0.5
        assert!((results[1].score - 0.5).abs() < 0.01);
    }
}
