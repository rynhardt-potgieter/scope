/// Java-specific metadata extraction and language plugin.
///
/// Extracts access modifiers (public, protected, private, package-private),
/// Java-specific modifiers (static, final, abstract, synchronized),
/// annotations, return type, parameters, and throws declarations from
/// Java AST nodes.
use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
use tree_sitter::Language;

use crate::core::graph::Edge;
use crate::core::parser::SupportedLanguage;
use crate::languages::LanguagePlugin;

/// Java language plugin.
pub struct JavaPlugin;

impl LanguagePlugin for JavaPlugin {
    fn language(&self) -> SupportedLanguage {
        SupportedLanguage::Java
    }

    fn extensions(&self) -> &[&str] {
        &["java"]
    }

    fn ts_language(&self) -> Language {
        tree_sitter_java::language()
    }

    fn symbol_query_source(&self) -> &str {
        include_str!("../queries/java/symbols.scm")
    }

    fn edge_query_source(&self) -> &str {
        include_str!("../queries/java/edges.scm")
    }

    fn infer_symbol_kind(&self, node_kind: &str) -> &str {
        match node_kind {
            "class_declaration" => "class",
            "interface_declaration" => "interface",
            "enum_declaration" => "enum",
            "record_declaration" => "class",
            "method_declaration" => "method",
            "constructor_declaration" => "method",
            "field_declaration" => "property",
            "annotation_type_declaration" => "type",
            "enum_constant" => "variant",
            _ => "function",
        }
    }

    fn scope_node_types(&self) -> &[&str] {
        &[
            "class_declaration",
            "interface_declaration",
            "enum_declaration",
            "method_declaration",
            "constructor_declaration",
            "lambda_expression",
        ]
    }

    fn class_body_node_types(&self) -> &[&str] {
        &["class_body", "interface_body", "enum_body"]
    }

    fn class_decl_node_types(&self) -> &[&str] {
        &[
            "class_declaration",
            "interface_declaration",
            "enum_declaration",
        ]
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
        extract_java_edge(pattern_index, captures, file_path, enclosing_scope_id)
    }

    fn extract_docstring(&self, node: &tree_sitter::Node, source: &str) -> Option<String> {
        // Java uses block comments (/** ... */) as Javadoc, which tree-sitter
        // represents as `block_comment` or `line_comment` preceding siblings.
        let prev = node.prev_sibling()?;
        match prev.kind() {
            "block_comment" | "line_comment" => {
                let text = prev.utf8_text(source.as_bytes()).ok()?;
                Some(text.trim().to_string())
            }
            _ => None,
        }
    }

    fn generic_name_stopwords(&self) -> &[&str] {
        &[
            "toString", "hashCode", "equals", "get", "set", "of", "main", "run", "close",
        ]
    }
}

/// Structured metadata for a Java symbol.
#[derive(Debug, Clone, Serialize, Default)]
pub struct JavaMetadata {
    /// Access modifier: "public", "protected", "private", or "package".
    pub access: String,
    /// Whether the symbol is static.
    pub is_static: bool,
    /// Whether the symbol is final.
    pub is_final: bool,
    /// Whether the symbol is abstract.
    pub is_abstract: bool,
    /// Whether the symbol is synchronized.
    pub is_synchronized: bool,
    /// Annotations on this symbol (e.g., "Override", "Deprecated", "Autowired").
    pub annotations: Vec<String>,
    /// Return type, if present (for methods).
    pub return_type: Option<String>,
    /// Parameter list with names and types.
    pub parameters: Vec<JavaParameterInfo>,
    /// Checked exceptions declared in throws clause.
    pub throws: Vec<String>,
}

/// Information about a single Java method/constructor parameter.
#[derive(Debug, Clone, Serialize)]
pub struct JavaParameterInfo {
    /// Parameter name.
    pub name: String,
    /// Type annotation, if present.
    #[serde(rename = "type")]
    pub type_annotation: Option<String>,
    /// Whether the parameter is declared final.
    pub is_final: bool,
}

/// Extract metadata from a Java AST node.
///
/// Returns a JSON string suitable for the `metadata` column.
pub fn extract_metadata(node: &tree_sitter::Node, source: &str, kind: &str) -> Result<String> {
    let mut meta = JavaMetadata::default();

    // Walk direct children to find modifiers
    let mut child_cursor = node.walk();
    for child in node.children(&mut child_cursor) {
        if child.kind() == "modifiers" {
            let mut mod_cursor = child.walk();
            for mod_child in child.children(&mut mod_cursor) {
                match mod_child.kind() {
                    "public" => meta.access = "public".to_string(),
                    "protected" => meta.access = "protected".to_string(),
                    "private" => meta.access = "private".to_string(),
                    "static" => meta.is_static = true,
                    "final" => meta.is_final = true,
                    "abstract" => meta.is_abstract = true,
                    "synchronized" => meta.is_synchronized = true,
                    "marker_annotation" | "annotation" => {
                        if let Ok(text) = mod_child.utf8_text(source.as_bytes()) {
                            // Strip leading `@` and any arguments
                            let ann_name = text
                                .trim_start_matches('@')
                                .split('(')
                                .next()
                                .unwrap_or("")
                                .trim()
                                .to_string();
                            if !ann_name.is_empty() {
                                meta.annotations.push(ann_name);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Default access if none was set — Java defaults to package-private
    if meta.access.is_empty() {
        meta.access = "package".to_string();
    }

    // Extract return type (for method_declaration)
    if kind == "method" {
        if let Some(type_node) = node.child_by_field_name("type") {
            if let Ok(text) = type_node.utf8_text(source.as_bytes()) {
                meta.return_type = Some(text.trim().to_string());
            }
        }
    }

    // Extract parameters
    if kind == "method" {
        if let Some(params_node) = node.child_by_field_name("parameters") {
            meta.parameters = extract_parameters(&params_node, source);
        }
    }

    // Extract throws clause
    let mut throws_cursor = node.walk();
    for child in node.children(&mut throws_cursor) {
        if child.kind() == "throws" {
            let mut tc = child.walk();
            for throw_child in child.children(&mut tc) {
                if throw_child.kind() == "type_identifier" {
                    if let Ok(text) = throw_child.utf8_text(source.as_bytes()) {
                        meta.throws.push(text.trim().to_string());
                    }
                }
            }
        }
    }

    let json = serde_json::to_string(&meta)?;
    Ok(json)
}

/// Extract parameter info from a formal_parameters node.
fn extract_parameters(params_node: &tree_sitter::Node, source: &str) -> Vec<JavaParameterInfo> {
    let mut params = Vec::new();
    let mut cursor = params_node.walk();

    for child in params_node.children(&mut cursor) {
        if child.kind() == "formal_parameter" || child.kind() == "spread_parameter" {
            let name = child
                .child_by_field_name("name")
                .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                .unwrap_or_default()
                .to_string();

            let type_annotation = child
                .child_by_field_name("type")
                .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                .map(|t| t.trim().to_string());

            // Check for final modifier on the parameter
            let mut is_final = false;
            let mut param_cursor = child.walk();
            for param_child in child.children(&mut param_cursor) {
                if param_child.kind() == "modifiers" {
                    let mut mc = param_child.walk();
                    for m in param_child.children(&mut mc) {
                        if m.kind() == "final" {
                            is_final = true;
                        }
                    }
                }
            }

            if !name.is_empty() {
                params.push(JavaParameterInfo {
                    name,
                    type_annotation,
                    is_final,
                });
            }
        }
    }

    params
}

/// Java edge extraction by pattern index.
///
/// Pattern indices map to the order of patterns in `queries/java/edges.scm`:
/// 0 = import declaration, 1 = member method call, 2 = direct method call,
/// 3 = this.method() call, 4 = object creation (new), 5 = extends (superclass),
/// 6 = class implements, 7 = interface extends, 8 = field type ref, 9 = param type ref
fn extract_java_edge(
    pattern: usize,
    captures: &HashMap<String, (String, u32)>,
    file_path: &str,
    enclosing_scope_id: Option<&str>,
) -> Vec<Edge> {
    let mut edges = Vec::new();

    let from_function = enclosing_scope_id
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{file_path}::__module__::function"));
    let from_class = enclosing_scope_id
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{file_path}::__module__::class"));

    match pattern {
        // Import declaration
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
        // Member method invocation (e.g. service.processPayment())
        1 => {
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
        // Direct method invocation (e.g. processPayment())
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
        // this.method() call — captures method name only
        3 => {
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
        // Object creation (new Foo())
        4 => {
            if let Some((class_name, line)) = captures.get("class_name") {
                edges.push(Edge {
                    from_id: from_function.clone(),
                    to_id: class_name.clone(),
                    kind: "instantiates".to_string(),
                    file_path: file_path.to_string(),
                    line: Some(*line),
                });
            }
        }
        // Superclass (extends)
        5 => {
            if let Some((base_type, line)) = captures.get("base_type") {
                edges.push(Edge {
                    from_id: from_class.clone(),
                    to_id: base_type.clone(),
                    kind: "extends".to_string(),
                    file_path: file_path.to_string(),
                    line: Some(*line),
                });
            }
        }
        // Class implements
        6 => {
            if let Some((base_type, line)) = captures.get("base_type") {
                edges.push(Edge {
                    from_id: from_class.clone(),
                    to_id: base_type.clone(),
                    kind: "implements".to_string(),
                    file_path: file_path.to_string(),
                    line: Some(*line),
                });
            }
        }
        // Interface extends
        7 => {
            if let Some((base_type, line)) = captures.get("base_type") {
                edges.push(Edge {
                    from_id: from_class.clone(),
                    to_id: base_type.clone(),
                    kind: "extends".to_string(),
                    file_path: file_path.to_string(),
                    line: Some(*line),
                });
            }
        }
        // Field type reference
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
        // Parameter type reference
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
