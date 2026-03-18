/// Core modules for Scope's indexing and querying engine.
///
/// - `graph` — SQLite-backed dependency graph storage
/// - `indexer` — orchestrates full and incremental indexing
/// - `parser` — tree-sitter parsing and symbol extraction
/// - `embedder` — embedding generation (stub)
/// - `searcher` — vector similarity search (stub)
/// - `watcher` — file system watching (stub)
pub mod embedder;
pub mod graph;
pub mod indexer;
pub mod parser;
pub mod searcher;
pub mod watcher;
