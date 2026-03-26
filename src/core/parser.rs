/// tree-sitter parsing and symbol/edge extraction.
///
/// Uses tree-sitter queries stored in `src/queries/<language>/` to extract
/// symbol definitions and relationships from source code.
///
/// Language-specific logic is provided by `LanguagePlugin` implementations
/// in `src/languages/`. Adding a new language requires only implementing
/// the trait and registering it in `CodeParser::new()`.
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use tree_sitter::{Parser, Query, QueryCursor};

use crate::core::graph::{Edge, Symbol};
use crate::languages::LanguagePlugin;

/// Supported programming languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportedLanguage {
    /// TypeScript (.ts, .tsx)
    TypeScript,
    /// C# (.cs) — planned
    CSharp,
    /// Python (.py) — planned
    Python,
    /// Go (.go) — planned
    Go,
    /// Java (.java) — planned
    Java,
    /// Rust (.rs) — planned
    Rust,
}

impl SupportedLanguage {
    /// Returns the language name as a lowercase string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TypeScript => "typescript",
            Self::CSharp => "csharp",
            Self::Python => "python",
            Self::Go => "go",
            Self::Java => "java",
            Self::Rust => "rust",
        }
    }
}

impl std::fmt::Display for SupportedLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::TypeScript => "TypeScript",
            Self::CSharp => "C#",
            Self::Python => "Python",
            Self::Go => "Go",
            Self::Java => "Java",
            Self::Rust => "Rust",
        };
        write!(f, "{name}")
    }
}

/// A registered language plugin with its compiled queries.
struct PluginEntry {
    /// The language plugin implementation.
    plugin: Box<dyn LanguagePlugin>,
    /// Compiled query for extracting symbol definitions.
    symbol_query: Query,
    /// Compiled query for extracting edges (calls, imports, etc.).
    edge_query: Query,
}

/// The code parser that uses tree-sitter to extract symbols and edges.
pub struct CodeParser {
    parser: Parser,
    plugins: Vec<PluginEntry>,
}

impl CodeParser {
    /// Create a new parser with all supported language plugins registered.
    pub fn new() -> Result<Self> {
        let parser = Parser::new();
        let mut plugins = Vec::new();

        // Register all language plugins.
        // To add a new language, create a LanguagePlugin impl and add it here.
        let all_plugins: Vec<Box<dyn LanguagePlugin>> = vec![
            Box::new(crate::languages::typescript::TypeScriptPlugin),
            Box::new(crate::languages::csharp::CSharpPlugin),
            Box::new(crate::languages::python::PythonPlugin),
            Box::new(crate::languages::rust_lang::RustPlugin),
            Box::new(crate::languages::go_lang::GoPlugin),
            Box::new(crate::languages::java::JavaPlugin),
        ];

        for plugin in all_plugins {
            let ts_lang = plugin.ts_language();
            let lang_name = plugin.language().to_string();

            let symbol_query = Query::new(&ts_lang, plugin.symbol_query_source())
                .with_context(|| format!("Failed to compile {lang_name} symbol query"))?;
            let edge_query = Query::new(&ts_lang, plugin.edge_query_source())
                .with_context(|| format!("Failed to compile {lang_name} edge query"))?;

            plugins.push(PluginEntry {
                plugin,
                symbol_query,
                edge_query,
            });
        }

        Ok(Self { parser, plugins })
    }

    /// Find the plugin entry for a given language.
    fn find_plugin(&self, lang: SupportedLanguage) -> Option<&PluginEntry> {
        self.plugins.iter().find(|e| e.plugin.language() == lang)
    }

    /// Detect the language of a file based on its extension.
    pub fn detect_language(path: &Path) -> Result<SupportedLanguage> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| anyhow::anyhow!("No file extension: {}", path.display()))?;

        match ext {
            "ts" | "tsx" => Ok(SupportedLanguage::TypeScript),
            "cs" => Ok(SupportedLanguage::CSharp),
            "py" => Ok(SupportedLanguage::Python),
            "go" => Ok(SupportedLanguage::Go),
            "java" => Ok(SupportedLanguage::Java),
            "rs" => Ok(SupportedLanguage::Rust),
            other => anyhow::bail!("Unsupported file extension: .{other}"),
        }
    }

    /// Check if a file extension is supported for parsing (has a loaded grammar).
    pub fn is_supported(&self, path: &Path) -> bool {
        let ext = match path.extension().and_then(|e| e.to_str()) {
            Some(e) => e,
            None => return false,
        };
        self.plugins
            .iter()
            .any(|entry| entry.plugin.extensions().contains(&ext))
    }

    /// Get the file extensions supported by a language.
    pub fn extensions_for(&self, lang: SupportedLanguage) -> &[&str] {
        self.find_plugin(lang)
            .map(|e| e.plugin.extensions())
            .unwrap_or(&[])
    }

    /// Extract symbol definitions from a source file.
    pub fn extract_symbols(
        &mut self,
        file_path: &str,
        source: &str,
        lang: SupportedLanguage,
    ) -> Result<Vec<Symbol>> {
        let entry = self
            .find_plugin(lang)
            .ok_or_else(|| anyhow::anyhow!("Language {:?} not loaded", lang))?;

        let ts_lang = entry.plugin.ts_language();

        self.parser
            .set_language(&ts_lang)
            .context("Failed to set parser language")?;

        let tree = self
            .parser
            .parse(source, None)
            .ok_or_else(|| anyhow::anyhow!("Parse failed for {file_path}"))?;

        let mut cursor = QueryCursor::new();

        // We need to borrow entry immutably while iterating, but self.parser
        // was already used above. Re-find the plugin for the iteration.
        let entry = self
            .find_plugin(lang)
            .ok_or_else(|| anyhow::anyhow!("Language {:?} not loaded", lang))?;

        let matches = cursor.matches(&entry.symbol_query, tree.root_node(), source.as_bytes());

        let mut symbols = Vec::new();
        let capture_names = entry.symbol_query.capture_names();

        for m in matches {
            let mut name_text: Option<String> = None;
            let mut def_node = None;
            let mut _params_text: Option<String> = None;
            let mut _return_type_text: Option<String> = None;

            for capture in m.captures {
                let capture_name = &capture_names[capture.index as usize];
                let text = capture
                    .node
                    .utf8_text(source.as_bytes())
                    .unwrap_or_default();

                match &**capture_name {
                    "name" => name_text = Some(text.to_string()),
                    "definition" => def_node = Some(capture.node),
                    "params" => _params_text = Some(text.to_string()),
                    "return_type" => _return_type_text = Some(text.to_string()),
                    _ => {}
                }
            }

            let Some(name) = name_text else { continue };
            let Some(def) = def_node else { continue };

            let kind = entry.plugin.infer_symbol_kind(def.kind()).to_string();
            let line = def.start_position().row as u32 + 1;
            let id = format!("{file_path}::{name}::{kind}::{line}");

            // Extract metadata using language-specific logic
            let metadata = entry.plugin.extract_metadata(&def, source, &kind)?;

            // Extract signature — the first line of the definition up to `{` or end of line
            let signature = extract_signature(&def, source);

            // Extract docstring — delegates to language plugin (Python overrides for string-based docstrings)
            let docstring = entry.plugin.extract_docstring(&def, source);

            // Determine parent_id for methods inside classes
            let parent_id = if kind == "method" || kind == "property" {
                find_parent_class(&def, source, file_path, entry.plugin.as_ref())
            } else {
                None
            };

            symbols.push(Symbol {
                id,
                name,
                kind,
                file_path: file_path.to_string(),
                line_start: def.start_position().row as u32 + 1,
                line_end: def.end_position().row as u32 + 1,
                signature,
                docstring,
                parent_id,
                language: lang.as_str().to_string(),
                metadata,
            });
        }

        Ok(symbols)
    }

    /// Extract edges (relationships) from a source file.
    pub fn extract_edges(
        &mut self,
        file_path: &str,
        source: &str,
        lang: SupportedLanguage,
    ) -> Result<Vec<Edge>> {
        let entry = self
            .find_plugin(lang)
            .ok_or_else(|| anyhow::anyhow!("Language {:?} not loaded", lang))?;

        let ts_lang = entry.plugin.ts_language();

        self.parser
            .set_language(&ts_lang)
            .context("Failed to set parser language")?;

        let tree = self
            .parser
            .parse(source, None)
            .ok_or_else(|| anyhow::anyhow!("Parse failed for {file_path}"))?;

        let mut cursor = QueryCursor::new();

        // Re-find plugin after mutable borrow of self.parser
        let entry = self
            .find_plugin(lang)
            .ok_or_else(|| anyhow::anyhow!("Language {:?} not loaded", lang))?;

        let matches = cursor.matches(&entry.edge_query, tree.root_node(), source.as_bytes());

        let mut edges = Vec::new();
        let capture_names = entry.edge_query.capture_names();

        for m in matches {
            let pattern = m.pattern_index;
            let mut captures_map: HashMap<String, (String, u32)> = HashMap::new();
            // Save a representative AST node from the first capture for scope resolution
            let mut representative_node: Option<tree_sitter::Node> = None;

            for capture in m.captures {
                let capture_name = capture_names[capture.index as usize].to_string();
                let text = capture
                    .node
                    .utf8_text(source.as_bytes())
                    .unwrap_or_default()
                    .to_string();
                let line = capture.node.start_position().row as u32 + 1;
                if representative_node.is_none() {
                    representative_node = Some(capture.node);
                }
                captures_map.insert(capture_name, (text, line));
            }

            // Resolve the enclosing scope for this match
            let enclosing_scope_id = representative_node
                .as_ref()
                .and_then(|n| find_enclosing_scope(n, source, file_path, entry.plugin.as_ref()));

            let extracted = entry.plugin.extract_edge(
                pattern,
                &captures_map,
                file_path,
                enclosing_scope_id.as_deref(),
            );
            edges.extend(extracted);
        }

        Ok(edges)
    }
}

/// Extract the signature — first line of the definition up to `{` or end of the line.
fn extract_signature(node: &tree_sitter::Node, source: &str) -> Option<String> {
    let start = node.start_byte();
    let end = node.end_byte();
    let text = &source[start..end];

    // Take up to the first `{` or newline, whichever comes first
    let sig = if let Some(brace_pos) = text.find('{') {
        text[..brace_pos].trim()
    } else if let Some(nl_pos) = text.find('\n') {
        text[..nl_pos].trim()
    } else {
        text.trim()
    };

    if sig.is_empty() {
        None
    } else {
        Some(sig.to_string())
    }
}

/// Walk up the AST from `node` to find the nearest enclosing scope (function, method, class).
/// Returns the symbol ID of that scope, or `None` if at module level.
fn find_enclosing_scope(
    node: &tree_sitter::Node,
    source: &str,
    file_path: &str,
    plugin: &dyn LanguagePlugin,
) -> Option<String> {
    let mut current = node.parent();

    let scope_types = plugin.scope_node_types();

    while let Some(parent) = current {
        if scope_types.contains(&parent.kind()) {
            // For arrow functions / function expressions assigned to variables,
            // walk up to the variable_declarator to get a meaningful name.
            if parent.kind() == "arrow_function" || parent.kind() == "function_expression" {
                if let Some(grandparent) = parent.parent() {
                    if grandparent.kind() == "variable_declarator" {
                        if let Some(name_node) = grandparent.child_by_field_name("name") {
                            if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
                                let line = grandparent.start_position().row as u32 + 1;
                                return Some(format!("{file_path}::{name}::function::{line}"));
                            }
                        }
                    }
                }
                // If we can't get a name from variable_declarator, keep walking up
                current = parent.parent();
                continue;
            }

            // Named scope — get its name and build the ID
            if let Some(name_node) = parent.child_by_field_name("name") {
                if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
                    let kind = plugin.infer_symbol_kind(parent.kind());
                    let line = parent.start_position().row as u32 + 1;
                    return Some(format!("{file_path}::{name}::{kind}::{line}"));
                }
            }
        }
        current = parent.parent();
    }

    None // Module level — no enclosing scope
}

/// Find the parent class for a method or property node.
fn find_parent_class(
    node: &tree_sitter::Node,
    source: &str,
    file_path: &str,
    plugin: &dyn LanguagePlugin,
) -> Option<String> {
    let class_body_nodes = plugin.class_body_node_types();
    let class_decl_nodes = plugin.class_decl_node_types();

    let mut current = node.parent();
    while let Some(parent) = current {
        if class_body_nodes.contains(&parent.kind()) {
            if let Some(class_node) = parent.parent() {
                if class_decl_nodes.contains(&class_node.kind()) {
                    if let Some(name_node) = class_node.child_by_field_name("name") {
                        let class_name = name_node.utf8_text(source.as_bytes()).ok()?;
                        let kind = plugin.infer_symbol_kind(class_node.kind());
                        let class_line = class_node.start_position().row as u32 + 1;
                        return Some(format!("{file_path}::{class_name}::{kind}::{class_line}"));
                    }
                }
            }
        }
        current = parent.parent();
    }
    None
}
