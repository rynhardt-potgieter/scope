/// C#-specific metadata extraction and language plugin.
///
/// Extracts access modifiers (public, private, protected, internal),
/// C#-specific modifiers (async, static, abstract, virtual, override, sealed, readonly),
/// return type, and parameters from C# AST nodes.
use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
use tree_sitter::Language;

use crate::core::graph::{Edge, Symbol};
use crate::core::parser::SupportedLanguage;
use crate::languages::{make_edge, resolve_scope_id, LanguagePlugin};

/// C# language plugin.
pub struct CSharpPlugin;

impl LanguagePlugin for CSharpPlugin {
    fn language(&self) -> SupportedLanguage {
        SupportedLanguage::CSharp
    }

    fn extensions(&self) -> &[&str] {
        &["cs"]
    }

    fn ts_language(&self) -> Language {
        tree_sitter_c_sharp::language()
    }

    fn symbol_query_source(&self) -> &str {
        include_str!("../queries/csharp/symbols.scm")
    }

    fn edge_query_source(&self) -> &str {
        include_str!("../queries/csharp/edges.scm")
    }

    fn infer_symbol_kind(&self, node_kind: &str) -> &str {
        match node_kind {
            "class_declaration" => "class",
            "method_declaration" => "method",
            "constructor_declaration" => "method",
            "property_declaration" => "property",
            "interface_declaration" => "interface",
            "enum_declaration" => "enum",
            "struct_declaration" => "struct",
            "record_declaration" => "class",
            "delegate_declaration" => "type",
            "enum_member_declaration" => "variant",
            _ => "function",
        }
    }

    fn scope_node_types(&self) -> &[&str] {
        &[
            "method_declaration",
            "constructor_declaration",
            "class_declaration",
            "struct_declaration",
            "interface_declaration",
            "record_declaration",
        ]
    }

    fn class_body_node_types(&self) -> &[&str] {
        &["declaration_list", "enum_member_declaration_list"]
    }

    fn class_decl_node_types(&self) -> &[&str] {
        &[
            "class_declaration",
            "struct_declaration",
            "interface_declaration",
            "record_declaration",
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
        extract_cs_edge(pattern_index, captures, file_path, enclosing_scope_id)
    }

    fn generic_name_stopwords(&self) -> &[&str] {
        &[
            "ToString",
            "GetHashCode",
            "Equals",
            "Dispose",
            "GetType",
            "Main",
        ]
    }
}

/// Structured metadata for a C# symbol.
#[derive(Debug, Clone, Serialize, Default)]
pub struct CSharpMetadata {
    /// Access modifier: public, private, protected, internal, or protected internal.
    /// C# defaults to private for class members, internal for top-level types.
    pub access: String,
    /// Whether the symbol is async.
    pub is_async: bool,
    /// Whether the symbol is static.
    pub is_static: bool,
    /// Whether the symbol is abstract.
    pub is_abstract: bool,
    /// Whether the symbol is virtual.
    pub is_virtual: bool,
    /// Whether the symbol is an override.
    pub is_override: bool,
    /// Whether the symbol is sealed.
    pub is_sealed: bool,
    /// Whether the symbol is readonly.
    pub is_readonly: bool,
    /// Whether the symbol is partial (relevant for partial classes).
    pub is_partial: bool,
    /// Return type, if present (for methods, properties).
    pub return_type: Option<String>,
    /// Parameter list with names, types, and optionality.
    pub parameters: Vec<CSharpParameterInfo>,
}

/// Information about a single C# method/constructor parameter.
#[derive(Debug, Clone, Serialize)]
pub struct CSharpParameterInfo {
    /// Parameter name.
    pub name: String,
    /// Type annotation.
    #[serde(rename = "type")]
    pub type_annotation: Option<String>,
    /// Whether the parameter has a default value (optional).
    pub optional: bool,
}

/// Extract metadata from a C# AST node.
///
/// Returns a JSON string suitable for the `metadata` column.
pub fn extract_metadata(node: &tree_sitter::Node, source: &str, kind: &str) -> Result<String> {
    let mut meta = CSharpMetadata::default();

    // Walk direct children to find modifiers
    let mut child_cursor = node.walk();
    for child in node.children(&mut child_cursor) {
        if child.kind() == "modifier" {
            if let Ok(text) = child.utf8_text(source.as_bytes()) {
                match text {
                    "public" => meta.access = "public".to_string(),
                    "private" => meta.access = "private".to_string(),
                    "protected" => {
                        // Could be "protected internal" — check if access already set
                        if meta.access == "internal" {
                            meta.access = "protected internal".to_string();
                        } else {
                            meta.access = "protected".to_string();
                        }
                    }
                    "internal" => {
                        if meta.access == "protected" {
                            meta.access = "protected internal".to_string();
                        } else {
                            meta.access = "internal".to_string();
                        }
                    }
                    "async" => meta.is_async = true,
                    "static" => meta.is_static = true,
                    "abstract" => meta.is_abstract = true,
                    "virtual" => meta.is_virtual = true,
                    "override" => meta.is_override = true,
                    "sealed" => meta.is_sealed = true,
                    "readonly" => meta.is_readonly = true,
                    "partial" => meta.is_partial = true,
                    _ => {}
                }
            }
        }
    }

    // Default access if none was set
    if meta.access.is_empty() {
        meta.access = match kind {
            "class" | "interface" | "struct" | "enum" => "internal".to_string(),
            _ => "private".to_string(),
        };
    }

    // Extract return type from the `returns` field (method_declaration)
    if let Some(returns_node) = node.child_by_field_name("returns") {
        if let Ok(text) = returns_node.utf8_text(source.as_bytes()) {
            meta.return_type = Some(text.trim().to_string());
        }
    }
    // For properties, the type is in the `type` field
    if kind == "property" {
        if let Some(type_node) = node.child_by_field_name("type") {
            if let Ok(text) = type_node.utf8_text(source.as_bytes()) {
                meta.return_type = Some(text.trim().to_string());
            }
        }
    }

    // Extract parameters
    if kind == "function" || kind == "method" || kind == "constructor" {
        if let Some(params_node) = node.child_by_field_name("parameters") {
            meta.parameters = extract_parameters(&params_node, source);
        }
    }

    let json = serde_json::to_string(&meta)?;
    Ok(json)
}

/// Extract parameter info from a parameter_list node.
fn extract_parameters(params_node: &tree_sitter::Node, source: &str) -> Vec<CSharpParameterInfo> {
    let mut params = Vec::new();
    let mut cursor = params_node.walk();

    for child in params_node.children(&mut cursor) {
        if child.kind() == "parameter" {
            let name = child
                .child_by_field_name("name")
                .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                .unwrap_or_default()
                .to_string();

            let type_annotation = child
                .child_by_field_name("type")
                .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                .map(|t| t.trim().to_string());

            // Check if parameter has a default value (= expression child)
            let mut param_cursor = child.walk();
            let has_default = child
                .children(&mut param_cursor)
                .any(|c| c.kind() == "equals_value_clause");
            let optional = has_default;

            if !name.is_empty() {
                params.push(CSharpParameterInfo {
                    name,
                    type_annotation,
                    optional,
                });
            }
        }
    }

    params
}

/// Merge partial C# classes that are split across multiple files.
///
/// Groups symbols by class name, keeps the first as primary, and re-parents
/// methods from secondary definitions to the primary class symbol.
/// Returns the IDs of symbols that should be removed (secondary class definitions).
///
/// Not yet wired into the indexing pipeline — will be called from `indexer.rs`
/// after C# files are parsed, once partial-class test fixtures are added.
#[allow(dead_code)]
pub fn merge_partial_classes(symbols: &mut [Symbol]) -> Vec<String> {
    use std::collections::HashMap;

    let mut removals = Vec::new();

    // Group class symbols by name
    let mut class_groups: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, symbol) in symbols.iter().enumerate() {
        if symbol.kind == "class" {
            // Only merge if the class is marked partial
            let is_partial = symbol.metadata.contains("\"is_partial\":true");
            if is_partial {
                class_groups.entry(symbol.name.clone()).or_default().push(i);
            }
        }
    }

    // For each class with multiple definitions, merge them
    for indices in class_groups.values() {
        if indices.len() <= 1 {
            continue;
        }

        let primary_id = symbols[indices[0]].id.clone();
        let secondary_ids: Vec<String> = indices[1..]
            .iter()
            .map(|&i| symbols[i].id.clone())
            .collect();

        // Re-parent methods from secondary classes to primary
        for symbol in symbols.iter_mut() {
            if let Some(ref parent) = symbol.parent_id {
                if secondary_ids.contains(parent) {
                    symbol.parent_id = Some(primary_id.clone());
                }
            }
        }

        // Mark secondary class symbols for removal
        removals.extend(secondary_ids);
    }

    removals
}

/// C# edge extraction by pattern index.
///
/// Pattern indices map to the order of patterns in `queries/csharp/edges.scm`:
/// 0 = using (identifier), 1 = using (qualified), 2 = member call,
/// 3 = direct call, 4 = new expression, 5 = this.Method() call,
/// 6 = base list (identifier), 7 = base list (qualified), 8 = base.Method() call,
/// 9 = switch case member access variant ref
fn extract_cs_edge(
    pattern: usize,
    captures: &HashMap<String, (String, u32)>,
    file_path: &str,
    enclosing_scope_id: Option<&str>,
) -> Vec<Edge> {
    let mut edges = Vec::new();

    let from_fn = resolve_scope_id(enclosing_scope_id, file_path, "function");
    let from_cls = resolve_scope_id(enclosing_scope_id, file_path, "class");
    let module_fn = || format!("{file_path}::__module__::function");

    match pattern {
        // Using directive with identifier — always module-level
        // Using directive with qualified name — always module-level
        0 | 1 => {
            if let Some((imported_name, line)) = captures.get("imported_name") {
                edges.push(make_edge(
                    module_fn(),
                    imported_name,
                    "imports",
                    file_path,
                    *line,
                ));
            }
        }
        // Member access call (e.g. _logger.Info(...))
        2 => {
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
        // Direct call (e.g. DoSomething(...))
        3 => {
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
        // Object creation (new ...)
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
        // this.Method() call — captures method name only
        // base.Method() call — captures method name only
        5 | 8 => {
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
        // Base list with identifier (implements/extends)
        // Base list with qualified name
        6 | 7 => {
            if let Some((base_type, line)) = captures.get("base_type") {
                edges.push(make_edge(
                    from_cls.clone(),
                    base_type,
                    "implements",
                    file_path,
                    *line,
                ));
            }
        }
        // Switch case with member access variant ref (e.g. case PaymentStatus.Pending:)
        9 => {
            if let Some((variant_ref, line)) = captures.get("variant_ref") {
                edges.push(make_edge(
                    from_fn.clone(),
                    variant_ref,
                    "references",
                    file_path,
                    *line,
                ));
            }
        }
        _ => {}
    }

    edges
}
