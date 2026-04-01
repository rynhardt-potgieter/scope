/// Language-specific extraction logic.
///
/// Each language module provides metadata extraction functions that
/// understand language-specific modifiers, access levels, and conventions.
///
/// The `LanguagePlugin` trait allows adding new language support without
/// modifying `parser.rs` — implement the trait and register the plugin.
pub mod csharp;
pub mod go_lang;
pub mod java;
pub mod python;
pub mod rust_lang;
pub mod typescript;

use anyhow::Result;
use std::collections::HashMap;
use tree_sitter::Language;

use crate::core::graph::Edge;
use crate::core::parser::SupportedLanguage;

/// Trait that each language plugin implements.
///
/// Adding a new language means implementing this trait and registering
/// the plugin in `CodeParser::new()` — `parser.rs` never changes.
pub trait LanguagePlugin: Send + Sync {
    /// Which language this plugin handles.
    fn language(&self) -> SupportedLanguage;

    /// File extensions this language matches (e.g., `["ts", "tsx"]`).
    fn extensions(&self) -> &[&str];

    /// The tree-sitter Language grammar.
    fn ts_language(&self) -> Language;

    /// Source text of the symbols.scm query.
    fn symbol_query_source(&self) -> &str;

    /// Source text of the edges.scm query.
    fn edge_query_source(&self) -> &str;

    /// Map a tree-sitter node kind to a Scope symbol kind.
    ///
    /// For example, `"function_declaration"` maps to `"function"`,
    /// `"class_declaration"` maps to `"class"`.
    fn infer_symbol_kind(&self, node_kind: &str) -> &str;

    /// Node types that constitute a scope boundary.
    fn scope_node_types(&self) -> &[&str];

    /// Node types for class body nodes (used in `find_parent_class`).
    fn class_body_node_types(&self) -> &[&str];

    /// Node types for class declaration nodes (used in `find_parent_class`).
    fn class_decl_node_types(&self) -> &[&str];

    /// Extract language-specific metadata from a symbol node as a JSON string.
    fn extract_metadata(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        kind: &str,
    ) -> Result<String>;

    /// Extract edges from a single query pattern match.
    ///
    /// `captures` maps capture names to `(text, line)` pairs.
    fn extract_edge(
        &self,
        pattern_index: usize,
        captures: &HashMap<String, (String, u32)>,
        file_path: &str,
        enclosing_scope_id: Option<&str>,
    ) -> Vec<Edge>;

    /// Extract a docstring from the definition node.
    ///
    /// Default implementation looks for a comment node as the previous sibling
    /// (works for TypeScript, C#, Rust). Override for languages where docstrings
    /// are string literals inside the body (e.g., Python).
    fn extract_docstring(&self, node: &tree_sitter::Node, source: &str) -> Option<String> {
        let prev = node.prev_sibling()?;
        if prev.kind() == "comment" {
            let text = prev.utf8_text(source.as_bytes()).ok()?;
            Some(text.trim().to_string())
        } else {
            None
        }
    }

    /// Returns symbol names that are too generic to be useful in search results.
    ///
    /// These names are de-ranked (not excluded) in FTS5 search — they never
    /// receive an importance boost regardless of caller count.
    fn generic_name_stopwords(&self) -> &[&str] {
        &[]
    }
}

/// Resolve the `from_id` for an outgoing edge.
///
/// If there is an `enclosing_scope_id` (i.e. the edge originates inside a
/// function or method), that ID is used directly.  Otherwise a synthetic
/// module-level ID of the form `"{file_path}::__module__::{kind}"` is
/// returned, where `kind` is either `"function"` or `"class"`.
pub fn resolve_scope_id(enclosing_scope_id: Option<&str>, file_path: &str, kind: &str) -> String {
    enclosing_scope_id
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{file_path}::__module__::{kind}"))
}

/// Build an [`Edge`] with the given fields.
///
/// Convenience wrapper so individual language plugins don't have to repeat
/// the full struct literal each time.
pub fn make_edge(
    from_id: impl Into<String>,
    to_id: impl Into<String>,
    kind: &str,
    file_path: &str,
    line: u32,
) -> Edge {
    Edge {
        from_id: from_id.into(),
        to_id: to_id.into(),
        kind: kind.to_string(),
        file_path: file_path.to_string(),
        line: Some(line),
    }
}

/// Look up stopwords for a language by name string.
///
/// Used by the embedder to check generic names without needing
/// a full `LanguagePlugin` instance.
pub fn stopwords_for_language(language: &str) -> &'static [&'static str] {
    match language {
        "typescript" => typescript::TypeScriptPlugin.generic_name_stopwords(),
        "csharp" => csharp::CSharpPlugin.generic_name_stopwords(),
        "python" => python::PythonPlugin.generic_name_stopwords(),
        "rust" => rust_lang::RustPlugin.generic_name_stopwords(),
        "go" => go_lang::GoPlugin.generic_name_stopwords(),
        "java" => java::JavaPlugin.generic_name_stopwords(),
        _ => &[],
    }
}
