/// Rust-specific metadata extraction and language plugin.
///
/// Extracts visibility modifiers (pub, pub(crate), pub(super), private),
/// Rust-specific modifiers (async, const, unsafe), attributes, return type,
/// and parameters from Rust AST nodes.
use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
use tree_sitter::Language;

use crate::core::graph::Edge;
use crate::core::parser::SupportedLanguage;
use crate::languages::LanguagePlugin;

/// Rust language plugin.
pub struct RustPlugin;

impl LanguagePlugin for RustPlugin {
    fn language(&self) -> SupportedLanguage {
        SupportedLanguage::Rust
    }

    fn extensions(&self) -> &[&str] {
        &["rs"]
    }

    fn ts_language(&self) -> Language {
        tree_sitter_rust::language()
    }

    fn symbol_query_source(&self) -> &str {
        include_str!("../queries/rust/symbols.scm")
    }

    fn edge_query_source(&self) -> &str {
        include_str!("../queries/rust/edges.scm")
    }

    fn infer_symbol_kind(&self, node_kind: &str) -> &str {
        match node_kind {
            "function_item" => "function",
            "struct_item" => "struct",
            "enum_item" => "enum",
            "trait_item" => "interface",
            "type_item" => "type",
            "const_item" | "static_item" => "const",
            _ => "function",
        }
    }

    fn scope_node_types(&self) -> &[&str] {
        &["function_item", "impl_item", "trait_item", "mod_item"]
    }

    fn class_body_node_types(&self) -> &[&str] {
        // Rust impl blocks contain a `declaration_list` body, but `impl_item` is not
        // stored as a symbol. The standard `find_parent_class` would generate a
        // parent_id referencing a non-existent symbol, causing FK constraint errors.
        // Leave empty so methods don't get an incorrect parent_id.
        &[]
    }

    fn class_decl_node_types(&self) -> &[&str] {
        &[]
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
        extract_rust_edge(pattern_index, captures, file_path, enclosing_scope_id)
    }
}

/// Structured metadata for a Rust symbol.
#[derive(Debug, Clone, Serialize, Default)]
pub struct RustMetadata {
    /// Visibility: "pub", "pub(crate)", "pub(super)", or "private".
    pub visibility: String,
    /// Whether the symbol is async.
    pub is_async: bool,
    /// Whether the symbol is const.
    pub is_const: bool,
    /// Whether the symbol is unsafe.
    pub is_unsafe: bool,
    /// Attributes like #[test], #[derive(Debug)].
    pub attributes: Vec<String>,
    /// Return type, if present.
    pub return_type: Option<String>,
    /// Parameter list with names and types.
    pub parameters: Vec<RustParameterInfo>,
}

/// Information about a single Rust function/method parameter.
#[derive(Debug, Clone, Serialize)]
pub struct RustParameterInfo {
    /// Parameter name.
    pub name: String,
    /// Type annotation, if present.
    #[serde(rename = "type")]
    pub type_annotation: Option<String>,
    /// Whether the parameter binding is mutable.
    pub is_mutable: bool,
}

/// Extract metadata from a Rust AST node.
///
/// Returns a JSON string suitable for the `metadata` column.
pub fn extract_metadata(node: &tree_sitter::Node, source: &str, kind: &str) -> Result<String> {
    let mut meta = RustMetadata {
        visibility: "private".to_string(),
        ..Default::default()
    };

    // Walk direct children to find modifiers and attributes
    let mut child_cursor = node.walk();
    for child in node.children(&mut child_cursor) {
        match child.kind() {
            "visibility_modifier" => {
                if let Ok(text) = child.utf8_text(source.as_bytes()) {
                    meta.visibility = match text.trim() {
                        "pub" => "pub".to_string(),
                        s if s.starts_with("pub(crate)") => "pub(crate)".to_string(),
                        s if s.starts_with("pub(super)") => "pub(super)".to_string(),
                        s if s.starts_with("pub") => s.to_string(),
                        _ => "private".to_string(),
                    };
                }
            }
            "attribute_item" => {
                if let Ok(text) = child.utf8_text(source.as_bytes()) {
                    meta.attributes.push(text.trim().to_string());
                }
            }
            _ => {}
        }
    }

    // Check for async, const, unsafe keywords in function items
    if kind == "function" || kind == "method" {
        let mut fn_cursor = node.walk();
        for child in node.children(&mut fn_cursor) {
            if let Ok(text) = child.utf8_text(source.as_bytes()) {
                match text {
                    "async" => meta.is_async = true,
                    "const" => meta.is_const = true,
                    "unsafe" => meta.is_unsafe = true,
                    _ => {}
                }
            }
        }

        // Extract return type
        if let Some(return_node) = node.child_by_field_name("return_type") {
            if let Ok(text) = return_node.utf8_text(source.as_bytes()) {
                // Strip the leading `-> ` from return types
                let clean = text.trim_start_matches("->").trim();
                if !clean.is_empty() {
                    meta.return_type = Some(clean.to_string());
                }
            }
        }

        // Extract parameters
        if let Some(params_node) = node.child_by_field_name("parameters") {
            meta.parameters = extract_parameters(&params_node, source);
        }
    }

    // For const/static items, mark is_const
    if kind == "const" {
        meta.is_const = true;
    }

    let json = serde_json::to_string(&meta)?;
    Ok(json)
}

/// Extract parameter info from a parameters node.
fn extract_parameters(params_node: &tree_sitter::Node, source: &str) -> Vec<RustParameterInfo> {
    let mut params = Vec::new();
    let mut cursor = params_node.walk();

    for child in params_node.children(&mut cursor) {
        match child.kind() {
            "parameter" => {
                let mut name = String::new();
                let mut type_annotation = None;
                let mut is_mutable = false;

                // Extract pattern (name) and type
                if let Some(pattern_node) = child.child_by_field_name("pattern") {
                    if let Ok(text) = pattern_node.utf8_text(source.as_bytes()) {
                        let text = text.trim();
                        if let Some(stripped) = text.strip_prefix("mut ") {
                            name = stripped.to_string();
                            is_mutable = true;
                        } else {
                            name = text.to_string();
                        }
                    }
                }

                if let Some(type_node) = child.child_by_field_name("type") {
                    if let Ok(text) = type_node.utf8_text(source.as_bytes()) {
                        type_annotation = Some(text.trim().to_string());
                    }
                }

                if !name.is_empty() {
                    params.push(RustParameterInfo {
                        name,
                        type_annotation,
                        is_mutable,
                    });
                }
            }
            "self_parameter" => {
                if let Ok(text) = child.utf8_text(source.as_bytes()) {
                    let text = text.trim();
                    let is_mutable = text.contains("mut");
                    params.push(RustParameterInfo {
                        name: "self".to_string(),
                        type_annotation: None,
                        is_mutable,
                    });
                }
            }
            _ => {}
        }
    }

    params
}

/// Rust edge extraction by pattern index.
///
/// Pattern indices map to the order of patterns in `queries/rust/edges.scm`:
/// 0 = use scoped, 1 = use aliased, 2 = direct call, 3 = scoped call,
/// 4 = method call, 5 = macro invocation, 6 = scoped macro,
/// 7 = field type ref, 8 = param type ref, 9 = return type ref
fn extract_rust_edge(
    pattern: usize,
    captures: &HashMap<String, (String, u32)>,
    file_path: &str,
    enclosing_scope_id: Option<&str>,
) -> Vec<Edge> {
    let mut edges = Vec::new();

    let from_function = enclosing_scope_id
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{file_path}::__module__::function"));
    let _from_class = enclosing_scope_id
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{file_path}::__module__::class"));

    match pattern {
        // Use declaration — scoped identifier (e.g. use std::io)
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
        // Use declaration — aliased (use ... as ...)
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
        // Direct call expression (e.g. process_payment(...))
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
        // Scoped call expression (e.g. PaymentService::new(...))
        3 => {
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
        // Method call expression (e.g. self.client.charge(...))
        4 => {
            if let Some((method, line)) = captures.get("method") {
                edges.push(Edge {
                    from_id: from_function.clone(),
                    to_id: method.clone(),
                    kind: "calls".to_string(),
                    file_path: file_path.to_string(),
                    line: Some(*line),
                });
            }
        }
        // Macro invocation (e.g. println!(...))
        5 => {
            if let Some((macro_name, line)) = captures.get("macro_name") {
                edges.push(Edge {
                    from_id: from_function.clone(),
                    to_id: format!("{macro_name}!"),
                    kind: "calls".to_string(),
                    file_path: file_path.to_string(),
                    line: Some(*line),
                });
            }
        }
        // Scoped macro invocation (e.g. std::println!(...))
        6 => {
            if let Some((macro_name, line)) = captures.get("macro_name") {
                edges.push(Edge {
                    from_id: from_function.clone(),
                    to_id: format!("{macro_name}!"),
                    kind: "calls".to_string(),
                    file_path: file_path.to_string(),
                    line: Some(*line),
                });
            }
        }
        // Field type reference
        7 => {
            if let Some((type_ref, line)) = captures.get("type_ref") {
                edges.push(Edge {
                    from_id: from_function.clone(),
                    to_id: type_ref.clone(),
                    kind: "references_type".to_string(),
                    file_path: file_path.to_string(),
                    line: Some(*line),
                });
            }
        }
        // Parameter type reference
        8 => {
            if let Some((type_ref, line)) = captures.get("type_ref") {
                edges.push(Edge {
                    from_id: from_function.clone(),
                    to_id: type_ref.clone(),
                    kind: "references_type".to_string(),
                    file_path: file_path.to_string(),
                    line: Some(*line),
                });
            }
        }
        // Return type reference
        9 => {
            if let Some((type_ref, line)) = captures.get("type_ref") {
                edges.push(Edge {
                    from_id: from_function.clone(),
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
