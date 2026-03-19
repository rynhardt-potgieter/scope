/// tree-sitter parsing and symbol/edge extraction.
///
/// Uses tree-sitter queries stored in `src/queries/<language>/` to extract
/// symbol definitions and relationships from source code.
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use tree_sitter::{Language, Parser, Query, QueryCursor};

use crate::core::graph::{Edge, Symbol};
use crate::languages::csharp;
use crate::languages::typescript;

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

/// Configuration for a single language: grammar, queries, and file extensions.
struct LanguageConfig {
    /// tree-sitter language grammar.
    language: Language,
    /// Query for extracting symbol definitions.
    symbol_query: Query,
    /// Query for extracting edges (calls, imports, etc.).
    edge_query: Query,
    /// File extensions this language handles (without the dot).
    extensions: Vec<&'static str>,
}

/// The code parser that uses tree-sitter to extract symbols and edges.
pub struct CodeParser {
    parser: Parser,
    languages: HashMap<SupportedLanguage, LanguageConfig>,
}

impl CodeParser {
    /// Create a new parser with TypeScript support initialised.
    pub fn new() -> Result<Self> {
        let parser = Parser::new();
        let mut languages = HashMap::new();

        // TypeScript
        let ts_lang = tree_sitter_typescript::language_typescript();
        let symbol_query = Query::new(&ts_lang, include_str!("../queries/typescript/symbols.scm"))
            .context("Failed to compile TypeScript symbol query")?;
        let edge_query = Query::new(&ts_lang, include_str!("../queries/typescript/edges.scm"))
            .context("Failed to compile TypeScript edge query")?;
        let ts_config = LanguageConfig {
            language: ts_lang,
            symbol_query,
            edge_query,
            extensions: vec!["ts", "tsx"],
        };
        languages.insert(SupportedLanguage::TypeScript, ts_config);

        // C#
        let cs_lang = tree_sitter_c_sharp::language();
        let cs_symbol_query = Query::new(&cs_lang, include_str!("../queries/csharp/symbols.scm"))
            .context("Failed to compile C# symbol query")?;
        let cs_edge_query = Query::new(&cs_lang, include_str!("../queries/csharp/edges.scm"))
            .context("Failed to compile C# edge query")?;
        let cs_config = LanguageConfig {
            language: cs_lang,
            symbol_query: cs_symbol_query,
            edge_query: cs_edge_query,
            extensions: vec!["cs"],
        };
        languages.insert(SupportedLanguage::CSharp, cs_config);

        Ok(Self { parser, languages })
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
        self.languages
            .values()
            .any(|config| config.extensions.contains(&ext))
    }

    /// Get the file extensions supported by a language.
    pub fn extensions_for(&self, lang: SupportedLanguage) -> &[&'static str] {
        self.languages
            .get(&lang)
            .map(|c| c.extensions.as_slice())
            .unwrap_or(&[])
    }

    /// Extract symbol definitions from a source file.
    pub fn extract_symbols(
        &mut self,
        file_path: &str,
        source: &str,
        lang: SupportedLanguage,
    ) -> Result<Vec<Symbol>> {
        let config = self
            .languages
            .get(&lang)
            .ok_or_else(|| anyhow::anyhow!("Language {:?} not loaded", lang))?;

        self.parser
            .set_language(&config.language)
            .context("Failed to set parser language")?;

        let tree = self
            .parser
            .parse(source, None)
            .ok_or_else(|| anyhow::anyhow!("Parse failed for {file_path}"))?;

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&config.symbol_query, tree.root_node(), source.as_bytes());

        let mut symbols = Vec::new();
        let capture_names = config.symbol_query.capture_names();

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

            let kind = infer_symbol_kind(def.kind());
            let line = def.start_position().row as u32 + 1;
            let id = format!("{file_path}::{name}::{kind}::{line}");

            // Extract metadata using language-specific logic
            let metadata = match lang {
                SupportedLanguage::TypeScript => typescript::extract_metadata(&def, source, &kind)?,
                SupportedLanguage::CSharp => csharp::extract_metadata(&def, source, &kind)?,
                _ => "{}".to_string(),
            };

            // Extract signature — the first line of the definition up to `{` or end of line
            let signature = extract_signature(&def, source);

            // Extract docstring — look for comment nodes immediately before the definition
            let docstring = extract_docstring(&def, source);

            // Determine parent_id for methods inside classes
            let parent_id = if kind == "method" || kind == "property" {
                find_parent_class(&def, source, file_path)
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
        let config = self
            .languages
            .get(&lang)
            .ok_or_else(|| anyhow::anyhow!("Language {:?} not loaded", lang))?;

        self.parser
            .set_language(&config.language)
            .context("Failed to set parser language")?;

        let tree = self
            .parser
            .parse(source, None)
            .ok_or_else(|| anyhow::anyhow!("Parse failed for {file_path}"))?;

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&config.edge_query, tree.root_node(), source.as_bytes());

        let mut edges = Vec::new();
        let capture_names = config.edge_query.capture_names();

        for m in matches {
            let pattern = m.pattern_index;
            let mut captures_map: HashMap<String, (String, u32)> = HashMap::new();

            for capture in m.captures {
                let capture_name = capture_names[capture.index as usize].to_string();
                let text = capture
                    .node
                    .utf8_text(source.as_bytes())
                    .unwrap_or_default()
                    .to_string();
                let line = capture.node.start_position().row as u32 + 1;
                captures_map.insert(capture_name, (text, line));
            }

            let extracted = extract_edge_from_pattern(lang, pattern, &captures_map, file_path);
            edges.extend(extracted);
        }

        Ok(edges)
    }
}

/// Infer the symbol kind from the tree-sitter node type.
fn infer_symbol_kind(node_kind: &str) -> String {
    match node_kind {
        // TypeScript / JavaScript
        "function_declaration" => "function".to_string(),
        "class_declaration" => "class".to_string(),
        "method_definition" => "method".to_string(),
        "interface_declaration" => "interface".to_string(),
        "enum_declaration" => "enum".to_string(),
        "type_alias_declaration" => "type".to_string(),
        "public_field_definition" => "property".to_string(),
        "lexical_declaration" | "arrow_function" | "function_expression" => "function".to_string(),
        // C#
        "method_declaration" => "method".to_string(),
        "constructor_declaration" => "method".to_string(),
        "property_declaration" => "property".to_string(),
        "struct_declaration" => "struct".to_string(),
        "record_declaration" => "class".to_string(),
        "delegate_declaration" => "type".to_string(),
        _ => "function".to_string(),
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

/// Extract a docstring from a comment node immediately preceding the definition.
fn extract_docstring(node: &tree_sitter::Node, source: &str) -> Option<String> {
    let prev = node.prev_sibling()?;
    if prev.kind() == "comment" {
        let text = prev.utf8_text(source.as_bytes()).ok()?;
        Some(text.trim().to_string())
    } else {
        None
    }
}

/// Parent container node types that hold methods/properties.
const CLASS_BODY_NODES: &[&str] = &["class_body", "declaration_list"];

/// Parent declaration node types that define classes/structs/interfaces.
const CLASS_DECL_NODES: &[&str] = &[
    "class_declaration",
    "struct_declaration",
    "interface_declaration",
    "record_declaration",
];

/// Find the parent class for a method or property node.
fn find_parent_class(node: &tree_sitter::Node, source: &str, file_path: &str) -> Option<String> {
    let mut current = node.parent();
    while let Some(parent) = current {
        if CLASS_BODY_NODES.contains(&parent.kind()) {
            if let Some(class_node) = parent.parent() {
                if CLASS_DECL_NODES.contains(&class_node.kind()) {
                    if let Some(name_node) = class_node.child_by_field_name("name") {
                        let class_name = name_node.utf8_text(source.as_bytes()).ok()?;
                        let kind = infer_symbol_kind(class_node.kind());
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

/// Extract edges from a query match pattern.
///
/// Pattern indices correspond to the order of patterns in the `.scm` query file,
/// so they differ per language.
fn extract_edge_from_pattern(
    lang: SupportedLanguage,
    pattern: usize,
    captures: &HashMap<String, (String, u32)>,
    file_path: &str,
) -> Vec<Edge> {
    match lang {
        SupportedLanguage::TypeScript => extract_ts_edge(pattern, captures, file_path),
        SupportedLanguage::CSharp => extract_cs_edge(pattern, captures, file_path),
        _ => Vec::new(),
    }
}

/// TypeScript edge extraction by pattern index.
fn extract_ts_edge(
    pattern: usize,
    captures: &HashMap<String, (String, u32)>,
    file_path: &str,
) -> Vec<Edge> {
    let mut edges = Vec::new();

    match pattern {
        // Import statement
        0 => {
            if let (Some((imported_name, line)), Some((source, _))) =
                (captures.get("imported_name"), captures.get("source"))
            {
                let source_clean = source.trim_matches(|c| c == '\'' || c == '"');
                edges.push(Edge {
                    from_id: format!("{file_path}::__module__::function"),
                    to_id: format!("{source_clean}::{imported_name}"),
                    kind: "imports".to_string(),
                    file_path: file_path.to_string(),
                    line: Some(*line),
                });
            }
        }
        // Direct call expression
        1 => {
            if let Some((callee, line)) = captures.get("callee") {
                edges.push(Edge {
                    from_id: format!("{file_path}::__module__::function"),
                    to_id: callee.clone(),
                    kind: "calls".to_string(),
                    file_path: file_path.to_string(),
                    line: Some(*line),
                });
            }
        }
        // Member call expression
        2 => {
            if let (Some((object, line)), Some((method, _))) =
                (captures.get("object"), captures.get("method"))
            {
                edges.push(Edge {
                    from_id: format!("{file_path}::__module__::function"),
                    to_id: format!("{object}.{method}"),
                    kind: "calls".to_string(),
                    file_path: file_path.to_string(),
                    line: Some(*line),
                });
            }
        }
        // New expression (instantiation)
        3 => {
            if let Some((class_name, line)) = captures.get("class_name") {
                edges.push(Edge {
                    from_id: format!("{file_path}::__module__::function"),
                    to_id: class_name.clone(),
                    kind: "instantiates".to_string(),
                    file_path: file_path.to_string(),
                    line: Some(*line),
                });
            }
        }
        // Extends clause
        4 => {
            if let Some((base_class, line)) = captures.get("base_class") {
                edges.push(Edge {
                    from_id: format!("{file_path}::__module__::class"),
                    to_id: base_class.clone(),
                    kind: "extends".to_string(),
                    file_path: file_path.to_string(),
                    line: Some(*line),
                });
            }
        }
        // Implements clause
        5 => {
            if let Some((iface_name, line)) = captures.get("interface_name") {
                edges.push(Edge {
                    from_id: format!("{file_path}::__module__::class"),
                    to_id: iface_name.clone(),
                    kind: "implements".to_string(),
                    file_path: file_path.to_string(),
                    line: Some(*line),
                });
            }
        }
        // Type reference
        6 => {
            if let Some((type_ref, line)) = captures.get("type_ref") {
                edges.push(Edge {
                    from_id: format!("{file_path}::__module__::function"),
                    to_id: type_ref.clone(),
                    kind: "references_type".to_string(),
                    file_path: file_path.to_string(),
                    line: Some(*line),
                });
            }
        }
        _ => {}
    }

    edges
}

/// C# edge extraction by pattern index.
fn extract_cs_edge(
    pattern: usize,
    captures: &HashMap<String, (String, u32)>,
    file_path: &str,
) -> Vec<Edge> {
    let mut edges = Vec::new();

    match pattern {
        // Using directive with identifier
        0 => {
            if let Some((imported_name, line)) = captures.get("imported_name") {
                edges.push(Edge {
                    from_id: format!("{file_path}::__module__::function"),
                    to_id: imported_name.clone(),
                    kind: "imports".to_string(),
                    file_path: file_path.to_string(),
                    line: Some(*line),
                });
            }
        }
        // Using directive with qualified name
        1 => {
            if let Some((imported_name, line)) = captures.get("imported_name") {
                edges.push(Edge {
                    from_id: format!("{file_path}::__module__::function"),
                    to_id: imported_name.clone(),
                    kind: "imports".to_string(),
                    file_path: file_path.to_string(),
                    line: Some(*line),
                });
            }
        }
        // Member access call (e.g. _logger.Info(...))
        2 => {
            if let (Some((object, line)), Some((method, _))) =
                (captures.get("object"), captures.get("method"))
            {
                edges.push(Edge {
                    from_id: format!("{file_path}::__module__::function"),
                    to_id: format!("{object}.{method}"),
                    kind: "calls".to_string(),
                    file_path: file_path.to_string(),
                    line: Some(*line),
                });
            }
        }
        // Direct call (e.g. DoSomething(...))
        3 => {
            if let Some((callee, line)) = captures.get("callee") {
                edges.push(Edge {
                    from_id: format!("{file_path}::__module__::function"),
                    to_id: callee.clone(),
                    kind: "calls".to_string(),
                    file_path: file_path.to_string(),
                    line: Some(*line),
                });
            }
        }
        // Object creation (new ...)
        4 => {
            if let Some((class_name, line)) = captures.get("class_name") {
                edges.push(Edge {
                    from_id: format!("{file_path}::__module__::function"),
                    to_id: class_name.clone(),
                    kind: "instantiates".to_string(),
                    file_path: file_path.to_string(),
                    line: Some(*line),
                });
            }
        }
        // Base list with identifier (implements/extends)
        5 => {
            if let Some((base_type, line)) = captures.get("base_type") {
                edges.push(Edge {
                    from_id: format!("{file_path}::__module__::class"),
                    to_id: base_type.clone(),
                    kind: "implements".to_string(),
                    file_path: file_path.to_string(),
                    line: Some(*line),
                });
            }
        }
        // Base list with qualified name
        6 => {
            if let Some((base_type, line)) = captures.get("base_type") {
                edges.push(Edge {
                    from_id: format!("{file_path}::__module__::class"),
                    to_id: base_type.clone(),
                    kind: "implements".to_string(),
                    file_path: file_path.to_string(),
                    line: Some(*line),
                });
            }
        }
        _ => {}
    }

    edges
}
