/// Orchestrates full and incremental indexing of a codebase.
///
/// Walks the file tree, parses source files, and stores symbols and edges
/// in the SQLite graph database.
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

use crate::config::ProjectConfig;
use crate::core::graph::Graph;
use crate::core::parser::{CodeParser, SupportedLanguage};
use crate::core::searcher::Searcher;

/// Statistics from an indexing run.
#[derive(Debug, Default)]
pub struct IndexStats {
    /// Number of files indexed.
    pub file_count: usize,
    /// Number of symbols extracted.
    pub symbol_count: usize,
    /// Number of edges extracted.
    pub edge_count: usize,
    /// Duration of the indexing run.
    pub duration: std::time::Duration,
    /// Per-language breakdown.
    pub language_stats: Vec<LanguageStats>,
}

/// Statistics from an incremental indexing run.
#[derive(Debug, Default)]
pub struct IncrementalStats {
    /// Files that were modified.
    pub modified: Vec<String>,
    /// Files that were added.
    pub added: Vec<String>,
    /// Files that were deleted.
    pub deleted: Vec<String>,
    /// Total symbols after update.
    pub symbol_count: usize,
    /// Total edges after update.
    pub edge_count: usize,
    /// Duration of the indexing run.
    pub duration: std::time::Duration,
    /// True if nothing changed (index up to date).
    pub up_to_date: bool,
}

/// Per-language statistics from an indexing run.
#[derive(Debug)]
pub struct LanguageStats {
    /// Language name.
    pub language: String,
    /// Number of files of this language.
    pub file_count: usize,
    /// Number of symbols extracted from this language.
    pub symbol_count: usize,
}

/// The indexer orchestrates parsing and storage.
pub struct Indexer {
    parser: CodeParser,
}

impl Indexer {
    /// Create a new indexer with language support initialised.
    pub fn new() -> Result<Self> {
        let parser = CodeParser::new()?;
        Ok(Self { parser })
    }

    /// Perform a full index of the project.
    ///
    /// Clears all existing data and re-indexes every supported file.
    /// If a `Searcher` is provided, symbols are also indexed for full-text search.
    pub fn index_full(
        &mut self,
        project_root: &Path,
        config: &ProjectConfig,
        graph: &mut Graph,
        searcher: Option<&Searcher>,
    ) -> Result<IndexStats> {
        let start = Instant::now();

        // Clear existing data
        graph.clear_all()?;
        if let Some(s) = searcher {
            if let Err(e) = s.clear_all() {
                tracing::warn!("Failed to clear search index: {e}");
            }
        }

        // Walk the file tree
        let files = self.collect_files(project_root, config)?;
        let total_files = files.len();
        eprintln!("Indexing {total_files} files...");

        let mut total_symbols = 0usize;
        let mut total_edges = 0usize;
        let mut file_hashes: HashMap<String, String> = HashMap::new();
        let mut lang_stats: HashMap<String, (usize, usize)> = HashMap::new();
        let mut all_symbols: Vec<crate::core::graph::Symbol> = Vec::new();

        for (rel_path, abs_path, lang) in &files {
            let source = std::fs::read_to_string(abs_path)
                .with_context(|| format!("Failed to read {}", abs_path.display()))?;

            // Compute file hash
            let hash = compute_hash(&source);
            file_hashes.insert(rel_path.clone(), hash);

            // Extract symbols and edges
            let symbols = self.parser.extract_symbols(rel_path, &source, *lang)?;
            let edges = self.parser.extract_edges(rel_path, &source, *lang)?;

            let sym_count = symbols.len();
            let edge_count = edges.len();

            // Store in graph
            graph.insert_file_data(rel_path, &symbols, &edges)?;

            // Collect symbols for FTS indexing (done after all edges are in the graph)
            if searcher.is_some() {
                all_symbols.extend(symbols);
            }

            total_symbols += sym_count;
            total_edges += edge_count;

            let entry = lang_stats.entry(lang.to_string()).or_insert((0, 0));
            entry.0 += 1;
            entry.1 += sym_count;
        }

        // Index symbols for full-text search with relationship context.
        // This is done after all symbols and edges are in the graph so that
        // caller/callee relationships are available for cross-file enrichment.
        if let Some(s) = searcher {
            let callers = graph.get_all_caller_names().unwrap_or_default();
            let callees = graph.get_all_callee_names().unwrap_or_default();
            let importance_scores = graph.compute_importance_scores()?;
            if let Err(e) = s.index_symbols(&all_symbols, &callers, &callees, &importance_scores) {
                tracing::warn!("Failed to index symbols for search: {e}");
            }
        }

        // Update file hashes
        graph.update_file_hashes(&file_hashes)?;

        let duration = start.elapsed();

        let language_stats: Vec<LanguageStats> = lang_stats
            .into_iter()
            .map(|(language, (file_count, symbol_count))| LanguageStats {
                language,
                file_count,
                symbol_count,
            })
            .collect();

        Ok(IndexStats {
            file_count: total_files,
            symbol_count: total_symbols,
            edge_count: total_edges,
            duration,
            language_stats,
        })
    }

    /// Perform an incremental index of the project.
    ///
    /// Compares file hashes to detect added, modified, and deleted files.
    /// Only re-parses changed files. Returns early if nothing changed.
    /// If a `Searcher` is provided, the search index is updated in sync.
    pub fn index_incremental(
        &mut self,
        project_root: &Path,
        config: &ProjectConfig,
        graph: &mut Graph,
        searcher: Option<&Searcher>,
    ) -> Result<IncrementalStats> {
        let start = Instant::now();

        // Collect all current files and compute hashes
        let files = self.collect_files(project_root, config)?;
        let mut current_hashes: HashMap<String, String> = HashMap::new();
        let mut file_map: HashMap<String, (std::path::PathBuf, SupportedLanguage)> = HashMap::new();

        for (rel_path, abs_path, lang) in &files {
            let source = std::fs::read_to_string(abs_path)
                .with_context(|| format!("Failed to read {}", abs_path.display()))?;
            let hash = compute_hash(&source);
            current_hashes.insert(rel_path.clone(), hash);
            file_map.insert(rel_path.clone(), (abs_path.clone(), *lang));
        }

        // Compare against stored hashes
        let changed = graph.get_changed_files(&current_hashes)?;

        if changed.is_empty() {
            return Ok(IncrementalStats {
                up_to_date: true,
                duration: start.elapsed(),
                ..Default::default()
            });
        }

        // Process deleted files
        for file_path in &changed.deleted {
            graph.delete_file_data(file_path)?;
            if let Some(s) = searcher {
                if let Err(e) = s.delete_file(file_path) {
                    tracing::warn!("Failed to remove search entries for {file_path}: {e}");
                }
            }
        }

        // Process modified and added files
        let files_to_reindex: Vec<String> = changed
            .modified
            .iter()
            .chain(changed.added.iter())
            .cloned()
            .collect();

        let mut updated_hashes: HashMap<String, String> = HashMap::new();
        let mut all_reindexed_symbols: Vec<crate::core::graph::Symbol> = Vec::new();

        for rel_path in &files_to_reindex {
            let (abs_path, lang) = file_map
                .get(rel_path)
                .ok_or_else(|| anyhow::anyhow!("File {} not found in file map", rel_path))?;

            let source = std::fs::read_to_string(abs_path)
                .with_context(|| format!("Failed to read {}", abs_path.display()))?;

            let symbols = self.parser.extract_symbols(rel_path, &source, *lang)?;
            let edges = self.parser.extract_edges(rel_path, &source, *lang)?;

            // Atomic per-file update: delete old data, insert new
            graph.insert_file_data(rel_path, &symbols, &edges)?;

            // Delete old search entries for this file
            if let Some(s) = searcher {
                if let Err(e) = s.delete_file(rel_path) {
                    tracing::warn!("Failed to clear search entries for {rel_path}: {e}");
                }
            }

            // Collect symbols for FTS re-indexing with relationship context
            if searcher.is_some() {
                all_reindexed_symbols.extend(symbols);
            }

            // Track the hash for this file
            if let Some(hash) = current_hashes.get(rel_path) {
                updated_hashes.insert(rel_path.clone(), hash.clone());
            }
        }

        // Re-index FTS for changed files with relationship context from graph
        if let Some(s) = searcher {
            let callers = graph.get_all_caller_names().unwrap_or_default();
            let callees = graph.get_all_callee_names().unwrap_or_default();
            let importance_scores = graph.compute_importance_scores()?;
            if let Err(e) = s.index_symbols(
                &all_reindexed_symbols,
                &callers,
                &callees,
                &importance_scores,
            ) {
                tracing::warn!("Failed to index symbols for search: {e}");
            }
        }

        // Update file hashes for changed/added files
        graph.update_file_hashes(&updated_hashes)?;

        // Remove hashes for deleted files (already done by delete_file_data)

        let duration = start.elapsed();

        Ok(IncrementalStats {
            modified: changed.modified,
            added: changed.added,
            deleted: changed.deleted,
            symbol_count: graph.symbol_count()?,
            edge_count: graph.edge_count()?,
            duration,
            up_to_date: false,
        })
    }

    /// Collect all supported files in the project, respecting .gitignore and config ignore patterns.
    fn collect_files(
        &self,
        project_root: &Path,
        config: &ProjectConfig,
    ) -> Result<Vec<(String, std::path::PathBuf, SupportedLanguage)>> {
        let mut files = Vec::new();

        let mut builder = ignore::WalkBuilder::new(project_root);
        builder.hidden(true).git_ignore(true).git_global(false);

        // Add custom ignore patterns from config
        let mut overrides = ignore::overrides::OverrideBuilder::new(project_root);
        for pattern in &config.index.ignore {
            // Negate the pattern to make it an ignore rule
            overrides
                .add(&format!("!{pattern}"))
                .with_context(|| format!("Invalid ignore pattern: {pattern}"))?;
        }
        let overrides = overrides.build()?;
        builder.overrides(overrides);

        // Pre-scan for nested projects (subdirectories with their own .scope/config.toml).
        // Files within these directories should not be indexed by the parent project.
        let nested_roots: Vec<std::path::PathBuf> = self.find_nested_projects(project_root);

        for entry in builder.build() {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            // Skip .scope/ directory
            if path
                .strip_prefix(project_root)
                .map(|p| p.starts_with(".scope"))
                .unwrap_or(false)
            {
                continue;
            }

            // Skip files inside nested projects (subdirs with their own .scope/config.toml)
            if nested_roots.iter().any(|nested| path.starts_with(nested)) {
                continue;
            }

            // Check if the file is a supported language
            if !self.parser.is_supported(path) {
                continue;
            }

            // Only index languages that are configured
            let lang = match CodeParser::detect_language(path) {
                Ok(l) => l,
                Err(_) => continue,
            };

            if !config
                .project
                .languages
                .iter()
                .any(|cl| cl.to_lowercase() == lang.as_str())
            {
                continue;
            }

            // Compute relative path with forward slashes
            let rel_path = path
                .strip_prefix(project_root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");

            files.push((rel_path, path.to_path_buf(), lang));
        }

        Ok(files)
    }

    /// Find subdirectories that contain their own `.scope/config.toml`.
    ///
    /// These are nested projects that should not be indexed by the parent.
    /// Only searches 3 levels deep to keep it fast.
    fn find_nested_projects(&self, project_root: &Path) -> Vec<std::path::PathBuf> {
        let mut nested = Vec::new();
        scan_for_nested(project_root, project_root, 0, 3, &mut nested);
        nested
    }
}

/// Recursively scan for nested `.scope/config.toml` directories.
fn scan_for_nested(
    project_root: &Path,
    current: &Path,
    depth: usize,
    max_depth: usize,
    results: &mut Vec<std::path::PathBuf>,
) {
    if depth > max_depth {
        return;
    }

    let entries = match std::fs::read_dir(current) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let dir_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => continue,
        };

        // Skip hidden dirs and common non-project dirs
        if dir_name.starts_with('.')
            || dir_name == "node_modules"
            || dir_name == "target"
            || dir_name == "dist"
            || dir_name == "build"
        {
            continue;
        }

        // Check if this subdir is a nested project
        if path != project_root && path.join(".scope").join("config.toml").exists() {
            results.push(path);
            // Don't recurse into nested projects
            continue;
        }

        scan_for_nested(project_root, &path, depth + 1, max_depth, results);
    }
}

/// Compute a SHA-256 hash of a file's contents.
fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}
