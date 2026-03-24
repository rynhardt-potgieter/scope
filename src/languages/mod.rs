/// Language-specific extraction logic.
///
/// Each language module provides metadata extraction functions that
/// understand language-specific modifiers, access levels, and conventions.
///
/// The `LanguagePlugin` trait allows adding new language support without
/// modifying `parser.rs` — implement the trait and register the plugin.
pub mod csharp;
pub mod python;
pub mod rust_lang;
pub mod typescript;

use anyhow::Result;
use std::collections::HashMap;
use tree_sitter::Language;

use crate::core::graph::Edge;
use crate::core::parser::SupportedLanguage;

/// Raw edge data returned by language plugins before scope resolution.
pub struct RawEdge {
    /// The edge kind (e.g. "calls", "imports", "extends").
    pub kind: String,
    /// The target symbol identifier.
    pub target: String,
    /// The source line number.
    pub line: u32,
}

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
}
