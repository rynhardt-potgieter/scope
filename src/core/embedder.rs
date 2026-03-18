//! Text construction for symbol search indexing.
//!
//! Builds rich text representations of symbols for full-text search.
//! The text combines symbol kind, name, signature, docstring, and parent
//! context, ordered by importance. This is what gets indexed in the FTS5
//! table and searched against user queries.

use crate::core::graph::Symbol;

/// Build the searchable text representation of a symbol.
///
/// Combines kind, name, signature, docstring, and parent context into a
/// single string optimised for full-text search. More important fields
/// appear first so that FTS5's BM25 ranking naturally weights them higher.
///
/// # Examples
///
/// A method with signature and docstring:
/// ```text
/// "method processPayment | (amount: Decimal, userId: string) -> PaymentResult | Processes a payment charge"
/// ```
///
/// A class with only a name:
/// ```text
/// "class PaymentService"
/// ```
pub fn build_embedding_text(symbol: &Symbol) -> String {
    let mut parts = Vec::new();

    // Kind and name: "method processPayment"
    parts.push(format!("{} {}", symbol.kind, symbol.name));

    // Split camelCase/PascalCase name into separate words for better matching.
    // e.g. "processPayment" -> "process Payment" so searching "payment" works.
    let split_name = split_camel_case(&symbol.name);
    if split_name != symbol.name {
        parts.push(split_name);
    }

    // Signature if available
    if let Some(sig) = &symbol.signature {
        parts.push(sig.clone());
    }

    // Docstring if available
    if let Some(doc) = &symbol.docstring {
        parts.push(doc.clone());
    }

    // Parent context from parent_id (extract class name from ID format "file::ClassName::class")
    if let Some(parent_id) = &symbol.parent_id {
        if let Some(parent_name) = extract_name_from_id(parent_id) {
            parts.push(format!("in {parent_name}"));
            // Also split parent name for search
            let split_parent = split_camel_case(parent_name);
            if split_parent != parent_name {
                parts.push(format!("in {split_parent}"));
            }
        }
    }

    parts.join(" | ")
}

/// Split a camelCase or PascalCase identifier into space-separated words.
///
/// Examples:
/// - `"processPayment"` -> `"process Payment"`
/// - `"PaymentService"` -> `"Payment Service"`
/// - `"getHTTPResponse"` -> `"get HTTP Response"`
/// - `"login"` -> `"login"` (no change)
fn split_camel_case(name: &str) -> String {
    let mut result = String::with_capacity(name.len() + 4);
    let chars: Vec<char> = name.chars().collect();

    for (i, &ch) in chars.iter().enumerate() {
        if i > 0 && ch.is_uppercase() {
            // Insert space before uppercase letter if preceded by lowercase
            // or if it starts a new word in an acronym sequence (e.g. HTTPResponse)
            let prev_lower = chars[i - 1].is_lowercase();
            let next_lower = chars.get(i + 1).is_some_and(|c| c.is_lowercase());
            if prev_lower || (next_lower && chars[i - 1].is_uppercase()) {
                result.push(' ');
            }
        }
        result.push(ch);
    }

    result
}

/// Extract the symbol name portion from a symbol ID.
///
/// Symbol IDs follow the format `"file_path::name::kind"`. This extracts
/// the `name` part. Returns `None` if the ID doesn't match the expected format.
fn extract_name_from_id(id: &str) -> Option<&str> {
    let parts: Vec<&str> = id.split("::").collect();
    if parts.len() >= 2 {
        Some(parts[parts.len() - 2])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::graph::Symbol;

    fn make_symbol(name: &str, kind: &str) -> Symbol {
        Symbol {
            id: format!("test.ts::{name}::{kind}"),
            name: name.to_string(),
            kind: kind.to_string(),
            file_path: "test.ts".to_string(),
            line_start: 1,
            line_end: 10,
            signature: None,
            docstring: None,
            parent_id: None,
            language: "typescript".to_string(),
            metadata: "{}".to_string(),
        }
    }

    #[test]
    fn test_basic_embedding_text() {
        let sym = make_symbol("PaymentService", "class");
        assert_eq!(
            build_embedding_text(&sym),
            "class PaymentService | Payment Service"
        );
    }

    #[test]
    fn test_basic_embedding_text_no_split_needed() {
        let sym = make_symbol("login", "function");
        // "login" has no camelCase to split, so no extra part
        assert_eq!(build_embedding_text(&sym), "function login");
    }

    #[test]
    fn test_embedding_text_with_signature() {
        let mut sym = make_symbol("processPayment", "method");
        sym.signature = Some("(amount: number) => boolean".to_string());
        assert_eq!(
            build_embedding_text(&sym),
            "method processPayment | process Payment | (amount: number) => boolean"
        );
    }

    #[test]
    fn test_embedding_text_with_docstring() {
        let mut sym = make_symbol("login", "function");
        sym.docstring = Some("Authenticates a user".to_string());
        assert_eq!(
            build_embedding_text(&sym),
            "function login | Authenticates a user"
        );
    }

    #[test]
    fn test_embedding_text_with_parent() {
        let mut sym = make_symbol("processPayment", "method");
        sym.parent_id = Some("src/pay.ts::PaymentService::class".to_string());
        sym.signature = Some("(amount: number) => boolean".to_string());
        sym.docstring = Some("Process a payment".to_string());
        assert_eq!(
            build_embedding_text(&sym),
            "method processPayment | process Payment | (amount: number) => boolean | Process a payment | in PaymentService | in Payment Service"
        );
    }

    #[test]
    fn test_split_camel_case() {
        assert_eq!(split_camel_case("processPayment"), "process Payment");
        assert_eq!(split_camel_case("PaymentService"), "Payment Service");
        assert_eq!(split_camel_case("login"), "login");
        assert_eq!(split_camel_case("getHTTPResponse"), "get HTTP Response");
        assert_eq!(split_camel_case("URL"), "URL");
    }

    #[test]
    fn test_extract_name_from_id() {
        assert_eq!(
            extract_name_from_id("src/pay.ts::PaymentService::class"),
            Some("PaymentService")
        );
        assert_eq!(extract_name_from_id("foo"), None);
    }
}
