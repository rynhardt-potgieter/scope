/// TypeScript-specific metadata extraction and language plugin.
///
/// Extracts access modifiers, async, static, return type, and parameters
/// from TypeScript AST nodes. TypeScript defaults to public access.
use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
use tree_sitter::Language;

use crate::core::graph::Edge;
use crate::core::parser::SupportedLanguage;
use crate::languages::{make_edge, resolve_scope_id, LanguagePlugin};

/// TypeScript language plugin.
pub struct TypeScriptPlugin;

impl LanguagePlugin for TypeScriptPlugin {
    fn language(&self) -> SupportedLanguage {
        SupportedLanguage::TypeScript
    }

    fn extensions(&self) -> &[&str] {
        &["ts", "tsx"]
    }

    fn ts_language(&self) -> Language {
        tree_sitter_typescript::language_typescript()
    }

    fn symbol_query_source(&self) -> &str {
        include_str!("../queries/typescript/symbols.scm")
    }

    fn edge_query_source(&self) -> &str {
        include_str!("../queries/typescript/edges.scm")
    }

    fn infer_symbol_kind(&self, node_kind: &str) -> &str {
        match node_kind {
            "function_declaration" => "function",
            "class_declaration" => "class",
            "method_definition" => "method",
            "interface_declaration" => "interface",
            "enum_declaration" => "enum",
            "type_alias_declaration" => "type",
            "public_field_definition" => "property",
            "lexical_declaration" | "arrow_function" | "function_expression" => "function",
            "enum_assignment" | "property_identifier" => "variant",
            _ => "function",
        }
    }

    fn scope_node_types(&self) -> &[&str] {
        &[
            "function_declaration",
            "method_definition",
            "arrow_function",
            "function_expression",
            "class_declaration",
            "interface_declaration",
        ]
    }

    fn class_body_node_types(&self) -> &[&str] {
        &["class_body", "enum_body"]
    }

    fn class_decl_node_types(&self) -> &[&str] {
        &["class_declaration", "enum_declaration"]
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
        extract_ts_edge(pattern_index, captures, file_path, enclosing_scope_id)
    }

    fn generic_name_stopwords(&self) -> &[&str] {
        &["constructor", "toString", "valueOf", "render", "default"]
    }
}

/// Structured metadata for a TypeScript symbol.
#[derive(Debug, Clone, Serialize, Default)]
pub struct SymbolMetadata {
    /// Access modifier: public, private, or protected. TypeScript defaults to public.
    pub access: String,
    /// Whether the symbol is async.
    pub is_async: bool,
    /// Whether the symbol is static.
    pub is_static: bool,
    /// Whether the symbol is abstract.
    pub is_abstract: bool,
    /// Whether the symbol is readonly.
    pub is_readonly: bool,
    /// Return type annotation, if present.
    pub return_type: Option<String>,
    /// Parameter list with names, types, and optionality.
    pub parameters: Vec<ParameterInfo>,
}

/// Information about a single function/method parameter.
#[derive(Debug, Clone, Serialize)]
pub struct ParameterInfo {
    /// Parameter name.
    pub name: String,
    /// Type annotation, if present.
    #[serde(rename = "type")]
    pub type_annotation: Option<String>,
    /// Whether the parameter is optional.
    pub optional: bool,
}

/// Extract metadata from a TypeScript AST node.
///
/// Returns a JSON string suitable for the `metadata` column.
pub fn extract_metadata(node: &tree_sitter::Node, source: &str, kind: &str) -> Result<String> {
    let mut meta = SymbolMetadata {
        access: "public".to_string(),
        ..Default::default()
    };

    // Walk direct children to find modifiers
    let mut child_cursor = node.walk();
    for child in node.children(&mut child_cursor) {
        match child.kind() {
            "async" => meta.is_async = true,
            "static" => meta.is_static = true,
            "abstract" => meta.is_abstract = true,
            "readonly" => meta.is_readonly = true,
            "accessibility_modifier" => {
                if let Ok(text) = child.utf8_text(source.as_bytes()) {
                    meta.access = text.to_string();
                }
            }
            _ => {}
        }
    }

    // Extract return type from type_annotation field
    if let Some(return_type_node) = node.child_by_field_name("return_type") {
        if let Ok(text) = return_type_node.utf8_text(source.as_bytes()) {
            // Strip the leading `: ` from type annotations
            let clean = text.trim_start_matches(':').trim();
            meta.return_type = Some(clean.to_string());
        }
    }

    // Extract parameters
    if kind == "function" || kind == "method" {
        if let Some(params_node) = node.child_by_field_name("parameters") {
            meta.parameters = extract_parameters(&params_node, source);
        }
    }

    let json = serde_json::to_string(&meta)?;
    Ok(json)
}

/// Extract parameter info from a formal_parameters node.
fn extract_parameters(params_node: &tree_sitter::Node, source: &str) -> Vec<ParameterInfo> {
    let mut params = Vec::new();
    let mut cursor = params_node.walk();

    for child in params_node.children(&mut cursor) {
        if child.kind() == "required_parameter" || child.kind() == "optional_parameter" {
            let optional = child.kind() == "optional_parameter";

            let name = child
                .child_by_field_name("pattern")
                .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                .unwrap_or_default()
                .to_string();

            let type_annotation = child
                .child_by_field_name("type")
                .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                .map(|t| t.trim_start_matches(':').trim().to_string());

            if !name.is_empty() {
                params.push(ParameterInfo {
                    name,
                    type_annotation,
                    optional,
                });
            }
        }
    }

    params
}

/// TypeScript edge extraction by pattern index.
///
/// Pattern indices map to the order of patterns in `queries/typescript/edges.scm`:
/// 0 = import, 1 = direct call, 2 = member call, 3 = chained member call,
/// 4 = new expression, 5 = extends, 6 = implements, 7 = this.method() call,
/// 8 = type reference
fn extract_ts_edge(
    pattern: usize,
    captures: &HashMap<String, (String, u32)>,
    file_path: &str,
    enclosing_scope_id: Option<&str>,
) -> Vec<Edge> {
    let mut edges = Vec::new();

    let from_fn = resolve_scope_id(enclosing_scope_id, file_path, "function");
    let from_cls = resolve_scope_id(enclosing_scope_id, file_path, "class");

    match pattern {
        // Import statement — always module-level, use __module__ synthetic ID
        0 => {
            if let (Some((imported_name, line)), Some((source, _))) =
                (captures.get("imported_name"), captures.get("source"))
            {
                let source_clean = source.trim_matches(|c| c == '\'' || c == '"');
                edges.push(make_edge(
                    format!("{file_path}::__module__::function"),
                    format!("{source_clean}::{imported_name}"),
                    "imports",
                    file_path,
                    *line,
                ));
            }
        }
        // Direct call expression
        1 => {
            if let Some((callee, line)) = captures.get("callee") {
                edges.push(make_edge(
                    from_fn.clone(),
                    callee,
                    "calls",
                    file_path,
                    *line,
                ));
            }
        }
        // Member call expression / chained member access call (patterns 2 and 3)
        2 | 3 => {
            if let (Some((object, line)), Some((method, _))) =
                (captures.get("object"), captures.get("method"))
            {
                edges.push(make_edge(
                    from_fn.clone(),
                    format!("{object}.{method}"),
                    "calls",
                    file_path,
                    *line,
                ));
            }
        }
        // New expression (instantiation)
        4 => {
            if let Some((class_name, line)) = captures.get("class_name") {
                edges.push(make_edge(
                    from_fn.clone(),
                    class_name,
                    "instantiates",
                    file_path,
                    *line,
                ));
            }
        }
        // Extends clause
        5 => {
            if let Some((base_class, line)) = captures.get("base_class") {
                edges.push(make_edge(
                    from_cls.clone(),
                    base_class,
                    "extends",
                    file_path,
                    *line,
                ));
            }
        }
        // Implements clause
        6 => {
            if let Some((iface_name, line)) = captures.get("interface_name") {
                edges.push(make_edge(
                    from_cls.clone(),
                    iface_name,
                    "implements",
                    file_path,
                    *line,
                ));
            }
        }
        // this.method() call — captures method name only
        7 => {
            if let Some((method, line)) = captures.get("method") {
                edges.push(make_edge(
                    from_fn.clone(),
                    method,
                    "calls",
                    file_path,
                    *line,
                ));
            }
        }
        // Type reference
        8 => {
            if let Some((type_ref, line)) = captures.get("type_ref") {
                edges.push(make_edge(
                    from_fn.clone(),
                    type_ref,
                    "references_type",
                    file_path,
                    *line,
                ));
            }
        }
        _ => {}
    }

    edges
}
