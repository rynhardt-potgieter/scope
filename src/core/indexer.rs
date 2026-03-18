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
    pub fn index_full(
        &mut self,
        project_root: &Path,
        config: &ProjectConfig,
        graph: &mut Graph,
    ) -> Result<IndexStats> {
        let start = Instant::now();

        // Clear existing data
        graph.clear_all()?;

        // Walk the file tree
        let files = self.collect_files(project_root, config)?;
        let total_files = files.len();
        eprintln!("Indexing {total_files} files...");

        let mut total_symbols = 0usize;
        let mut total_edges = 0usize;
        let mut file_hashes: HashMap<String, String> = HashMap::new();
        let mut lang_stats: HashMap<String, (usize, usize)> = HashMap::new();

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

            total_symbols += sym_count;
            total_edges += edge_count;

            let entry = lang_stats.entry(lang.to_string()).or_insert((0, 0));
            entry.0 += 1;
            entry.1 += sym_count;
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
}

/// Compute a SHA-256 hash of a file's contents.
fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}
