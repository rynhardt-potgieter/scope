/// TypeScript-specific metadata extraction.
///
/// Extracts access modifiers, async, static, return type, and parameters
/// from TypeScript AST nodes. TypeScript defaults to public access.
use anyhow::Result;
use serde::Serialize;

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
