/// Core modules for Scope's indexing and querying engine.
///
/// - `graph` — SQLite-backed dependency graph storage
/// - `workspace_graph` — workspace-level query facade over multiple graphs
/// - `indexer` — orchestrates full and incremental indexing
/// - `parser` — tree-sitter parsing and symbol extraction
/// - `embedder` — text construction for FTS5 search indexing
/// - `searcher` — FTS5-backed full-text search over symbols
/// - `watcher` — file system watching with lock-based single-instance guard
pub mod embedder;
pub mod graph;
pub mod indexer;
pub mod parser;
pub mod searcher;
pub mod watcher;
pub mod workspace_graph;
