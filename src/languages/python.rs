/// Python-specific metadata extraction and language plugin.
///
/// Extracts access level (public, private, name_mangled from naming conventions),
/// async, decorator-based modifiers (staticmethod, classmethod, abstractmethod, property),
/// return type annotations, and parameters from Python AST nodes.
///
/// Python docstrings are NOT comment nodes — they are the first `expression_statement`
/// child of the function/class body containing a `string` node.
use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
use tree_sitter::Language;

use crate::core::graph::Edge;
use crate::core::parser::SupportedLanguage;
use crate::languages::LanguagePlugin;

/// Python language plugin.
pub struct PythonPlugin;

impl LanguagePlugin for PythonPlugin {
    fn language(&self) -> SupportedLanguage {
        SupportedLanguage::Python
    }

    fn extensions(&self) -> &[&str] {
        &["py"]
    }

    fn ts_language(&self) -> Language {
        tree_sitter_python::language()
    }

    fn symbol_query_source(&self) -> &str {
        include_str!("../queries/python/symbols.scm")
    }

    fn edge_query_source(&self) -> &str {
        include_str!("../queries/python/edges.scm")
    }

    fn infer_symbol_kind(&self, node_kind: &str) -> &str {
        match node_kind {
            // Python uses `function_definition` for both top-level functions and
            // class methods. We map to "function" here. Note: parser.rs only sets
            // parent_id for kind == "method" || kind == "property", so Python
            // methods won't have parent_id set automatically. This is a known
            // limitation of the current LanguagePlugin trait contract.
            "function_definition" => "function",
            "class_definition" => "class",
            _ => "function",
        }
    }

    fn scope_node_types(&self) -> &[&str] {
        &[
            "function_definition",
            "class_definition",
            "decorated_definition",
            "module",
        ]
    }

    fn class_body_node_types(&self) -> &[&str] {
        &["block"]
    }

    fn class_decl_node_types(&self) -> &[&str] {
        &["class_definition"]
    }

    fn extract_metadata(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        kind: &str,
    ) -> Result<String> {
        extract_metadata(node, source, kind)
    }

    fn extract_edge(
        &self,
        pattern_index: usize,
        captures: &HashMap<String, (String, u32)>,
        file_path: &str,
        enclosing_scope_id: Option<&str>,
    ) -> Vec<Edge> {
        extract_py_edge(pattern_index, captures, file_path, enclosing_scope_id)
    }

    fn extract_docstring(&self, node: &tree_sitter::Node, source: &str) -> Option<String> {
        extract_docstring(node, source)
    }
}

/// Structured metadata for a Python symbol.
#[derive(Debug, Clone, Serialize, Default)]
pub struct PythonMetadata {
    /// Access level: "public", "private" (single `_` prefix), or "name_mangled" (double `__` prefix).
    pub access: String,
    /// Whether the function is async (`async def`).
    pub is_async: bool,
    /// Whether the function has a `@staticmethod` decorator.
    pub is_static: bool,
    /// Whether the function has a `@classmethod` decorator.
    pub is_classmethod: bool,
    /// Whether the function has a `@abstractmethod` decorator.
    pub is_abstract: bool,
    /// Whether the function has a `@property` decorator.
    pub is_property: bool,
    /// All decorator names on this symbol.
    pub decorators: Vec<String>,
    /// Return type annotation, if present.
    pub return_type: Option<String>,
    /// Parameter list with names, type annotations, and default status.
    pub parameters: Vec<PythonParameterInfo>,
}

/// Information about a single Python function/method parameter.
#[derive(Debug, Clone, Serialize)]
pub struct PythonParameterInfo {
    /// Parameter name.
    pub name: String,
    /// Type annotation, if present.
    #[serde(rename = "type")]
    pub type_annotation: Option<String>,
    /// Whether the parameter has a default value.
    pub has_default: bool,
}

/// Extract metadata from a Python AST node.
///
/// The `node` is the `@definition` capture from the symbol query — either a
/// `function_definition` or `class_definition` node. If it is wrapped in a
/// `decorated_definition`, we walk up to the parent to extract decorators.
///
/// Returns a JSON string suitable for the `metadata` column.
pub fn extract_metadata(node: &tree_sitter::Node, source: &str, kind: &str) -> Result<String> {
    let mut meta = PythonMetadata {
        access: "public".to_string(),
        ..Default::default()
    };

    // Check if the parent is a decorated_definition and extract decorators from it.
    if let Some(parent) = node.parent() {
        if parent.kind() == "decorated_definition" {
            let mut cursor = parent.walk();
            for child in parent.children(&mut cursor) {
                if child.kind() == "decorator" {
                    if let Ok(text) = child.utf8_text(source.as_bytes()) {
                        // Strip leading `@` and any arguments (e.g., `@decorator(args)` -> `decorator`)
                        let dec_name = text
                            .trim_start_matches('@')
                            .split('(')
                            .next()
                            .unwrap_or("")
                            .trim()
                            .to_string();
                        if !dec_name.is_empty() {
                            match dec_name.as_str() {
                                "staticmethod" => meta.is_static = true,
                                "classmethod" => meta.is_classmethod = true,
                                "abstractmethod" | "abc.abstractmethod" => meta.is_abstract = true,
                                "property" => meta.is_property = true,
                                _ => {}
                            }
                            meta.decorators.push(dec_name);
                        }
                    }
                }
            }
        }
    }

    // Check for async keyword (async def)
    if node.kind() == "function_definition" {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "async" {
                meta.is_async = true;
                break;
            }
        }
    }

    // Infer access from the symbol name
    if let Some(name_node) = node.child_by_field_name("name") {
        if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
            meta.access = infer_access(name);
        }
    }

    // Extract return type annotation
    if let Some(return_type_node) = node.child_by_field_name("return_type") {
        if let Ok(text) = return_type_node.utf8_text(source.as_bytes()) {
            // Strip leading `-> ` from return type annotations
            let clean = text.trim_start_matches("->").trim();
            if !clean.is_empty() {
                meta.return_type = Some(clean.to_string());
            }
        }
    }

    // Extract parameters
    if kind == "function" || kind == "method" || kind == "class" {
        if let Some(params_node) = node.child_by_field_name("parameters") {
            meta.parameters = extract_parameters(&params_node, source);
        }
    }

    let json = serde_json::to_string(&meta)?;
    Ok(json)
}

/// Infer Python access level from naming conventions.
///
/// - Name starts with `__` and does NOT end with `__` -> "name_mangled"
/// - Name starts with `_` -> "private"
/// - Otherwise -> "public"
fn infer_access(name: &str) -> String {
    if name.starts_with("__") && !name.ends_with("__") {
        "name_mangled".to_string()
    } else if name.starts_with('_') {
        "private".to_string()
    } else {
        "public".to_string()
    }
}

/// Extract parameter info from a `parameters` node.
fn extract_parameters(params_node: &tree_sitter::Node, source: &str) -> Vec<PythonParameterInfo> {
    let mut params = Vec::new();
    let mut cursor = params_node.walk();

    for child in params_node.children(&mut cursor) {
        match child.kind() {
            // Regular parameter: `name` or `name: type` or `name: type = default`
            "identifier" => {
                if let Ok(name) = child.utf8_text(source.as_bytes()) {
                    let name = name.to_string();
                    // Skip `self` and `cls`
                    if name != "self" && name != "cls" {
                        params.push(PythonParameterInfo {
                            name,
                            type_annotation: None,
                            has_default: false,
                        });
                    }
                }
            }
            // Typed parameter: `name: type`
            "typed_parameter" => {
                let name_node = child.child_by_field_name("name").or_else(|| {
                    // Sometimes the name is the first identifier child
                    let mut c = child.walk();
                    let found = child.children(&mut c).find(|n| n.kind() == "identifier");
                    found
                });
                let name = name_node
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                    .unwrap_or_default()
                    .to_string();

                // Skip `self` and `cls`
                if name == "self" || name == "cls" {
                    continue;
                }

                let type_annotation = child
                    .child_by_field_name("type")
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                    .map(|t| t.trim().to_string());

                params.push(PythonParameterInfo {
                    name,
                    type_annotation,
                    has_default: false,
                });
            }
            // Default parameter: `name = value`
            "default_parameter" => {
                let name = child
                    .child_by_field_name("name")
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                    .unwrap_or_default()
                    .to_string();

                if name == "self" || name == "cls" {
                    continue;
                }

                params.push(PythonParameterInfo {
                    name,
                    type_annotation: None,
                    has_default: true,
                });
            }
            // Typed default parameter: `name: type = value`
            "typed_default_parameter" => {
                let name = child
                    .child_by_field_name("name")
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                    .unwrap_or_default()
                    .to_string();

                if name == "self" || name == "cls" {
                    continue;
                }

                let type_annotation = child
                    .child_by_field_name("type")
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                    .map(|t| t.trim().to_string());

                params.push(PythonParameterInfo {
                    name,
                    type_annotation,
                    has_default: true,
                });
            }
            _ => {}
        }
    }

    params
}

/// Extract Python docstring from the first statement in a function/class body.
///
/// Python docstrings are the first `expression_statement` child of the body `block`
/// containing a `string` node. They are NOT comment nodes.
pub fn extract_docstring(node: &tree_sitter::Node, source: &str) -> Option<String> {
    // The node is the inner function_definition or class_definition.
    // Find the body block.
    let body = node.child_by_field_name("body")?;

    // Check the first child — docstrings must be the very first statement.
    let mut cursor = body.walk();
    let first_child = body.children(&mut cursor).next()?;

    if first_child.kind() != "expression_statement" {
        return None;
    }

    // Look for a string node inside the expression_statement
    let mut inner_cursor = first_child.walk();
    for inner in first_child.children(&mut inner_cursor) {
        if inner.kind() == "string" {
            if let Ok(text) = inner.utf8_text(source.as_bytes()) {
                // Strip triple-quote delimiters
                let cleaned = text
                    .trim_start_matches("\"\"\"")
                    .trim_start_matches("'''")
                    .trim_end_matches("\"\"\"")
                    .trim_end_matches("'''")
                    .trim();
                if !cleaned.is_empty() {
                    return Some(cleaned.to_string());
                }
            }
        }
    }

    None
}

/// Python edge extraction by pattern index.
///
/// Pattern indices map to the order of patterns in `queries/python/edges.scm`:
/// 0 = import statement, 1 = from-import statement, 2 = direct call,
/// 3 = attribute/method call, 4 = class inheritance
fn extract_py_edge(
    pattern: usize,
    captures: &HashMap<String, (String, u32)>,
    file_path: &str,
    enclosing_scope_id: Option<&str>,
) -> Vec<Edge> {
    let mut edges = Vec::new();

    // Resolve from_id: use enclosing scope when available, fall back to __module__ synthetic ID.
    let from_function = enclosing_scope_id
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{file_path}::__module__::function"));
    let from_class = enclosing_scope_id
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{file_path}::__module__::class"));

    match pattern {
        // import statement (e.g. `import os`) — always module-level
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
        // from-import statement (e.g. `from os.path import join`)
        1 => {
            if let (Some((imported_name, line)), Some((source_mod, _))) =
                (captures.get("imported_name"), captures.get("source"))
            {
                edges.push(Edge {
                    from_id: format!("{file_path}::__module__::function"),
                    to_id: format!("{source_mod}::{imported_name}"),
                    kind: "imports".to_string(),
                    file_path: file_path.to_string(),
                    line: Some(*line),
                });
            }
        }
        // Direct function call (e.g. `foo()`)
        2 => {
            if let Some((callee, line)) = captures.get("callee") {
                edges.push(Edge {
                    from_id: from_function.clone(),
                    to_id: callee.clone(),
                    kind: "calls".to_string(),
                    file_path: file_path.to_string(),
                    line: Some(*line),
                });
            }
        }
        // Attribute/method call (e.g. `self.foo()`, `obj.bar()`)
        3 => {
            if let (Some((object, line)), Some((method, _))) =
                (captures.get("object"), captures.get("method"))
            {
                edges.push(Edge {
                    from_id: from_function.clone(),
                    to_id: format!("{object}.{method}"),
                    kind: "calls".to_string(),
                    file_path: file_path.to_string(),
                    line: Some(*line),
                });
            }
        }
        // Class inheritance (e.g. `class Foo(Bar):`)
        4 => {
            if let Some((base_class, line)) = captures.get("base_class") {
                edges.push(Edge {
                    from_id: from_class.clone(),
                    to_id: base_class.clone(),
                    kind: "extends".to_string(),
                    file_path: file_path.to_string(),
                    line: Some(*line),
                });
            }
        }
        _ => {}
    }

    edges
}
