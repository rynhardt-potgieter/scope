//! Human-readable output formatting for all Scope commands.
//!
//! Rules:
//! - Separator line uses `─` (U+2500), never `-` or `=`
//! - File paths always use forward slashes, even on Windows
//! - Line ranges formatted as `start-end`
//! - Caller counts in square brackets: `[11 callers]`, `[internal]`
//! - Similarity scores always 2 decimal places: `0.91`

use std::collections::HashMap;

use crate::core::graph::{CallerInfo, ClassRelationships, Dependency, Reference, Symbol};

/// The separator line used between header and body in all command output.
pub const SEPARATOR: &str =
    "──────────────────────────────────────────────────────────────────────────────";

/// Normalize a file path to always use forward slashes in output.
pub fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// Format a line range as `start-end`, or just `start` if start == end.
pub fn format_line_range(start: u32, end: u32) -> String {
    if start == end {
        format!("{start}")
    } else {
        format!("{start}-{end}")
    }
}

/// Format the header line: `name  kind  file:line_range`
fn print_header(symbol: &Symbol) {
    let path = normalize_path(&symbol.file_path);
    let line_range = format_line_range(symbol.line_start, symbol.line_end);
    println!(
        "{:<50}{}  {}:{}",
        symbol.name, symbol.kind, path, line_range
    );
    println!("{SEPARATOR}");
}

/// Print a class sketch.
///
/// Format:
/// ```text
/// PaymentService                                    class  src/payments/service.ts:12
/// ──────────────────────────────────────────────────────────────────────────────
/// deps:     StripeClient, UserRepository, Logger
/// extends:  BaseService
/// implements: IPaymentService
///
/// methods:
///   processPayment(amount: Decimal, userId: string) → PaymentResult       [11 callers]
///   validateCard(card: CardDetails) → ValidationResult                     [internal]
///
/// fields:
///   private client: StripeClient
/// ```
pub fn print_class_sketch(
    symbol: &Symbol,
    methods: &[Symbol],
    caller_counts: &HashMap<String, usize>,
    relationships: &ClassRelationships,
    limit: usize,
) {
    print_header(symbol);

    // Dependencies line
    if !relationships.dependencies.is_empty() {
        println!("deps:     {}", relationships.dependencies.join(", "));
    }

    // Extends line
    if !relationships.extends.is_empty() {
        println!("extends:  {}", relationships.extends.join(", "));
    }

    // Implements line
    if !relationships.implements.is_empty() {
        println!("implements: {}", relationships.implements.join(", "));
    }

    // Separate sections: methods and fields
    let (method_syms, field_syms): (Vec<&Symbol>, Vec<&Symbol>) = methods
        .iter()
        .partition(|m| m.kind == "method" || m.kind == "function");

    // Methods section
    if !method_syms.is_empty() {
        println!();
        println!("methods:");
        let display_methods = if method_syms.len() > limit {
            &method_syms[..limit]
        } else {
            &method_syms
        };

        for method in display_methods {
            let sig = method_display_line(method);
            let count = caller_counts.get(&method.id).copied().unwrap_or(0);
            let count_label = if count > 0 {
                format!("[{count} caller{}]", if count == 1 { "" } else { "s" })
            } else {
                "[internal]".to_string()
            };
            // Right-align the caller count
            let padding = SEPARATOR
                .chars()
                .count()
                .saturating_sub(2 + sig.chars().count() + count_label.chars().count());
            println!(
                "  {sig}{:>width$}",
                count_label,
                width = padding + count_label.len()
            );
        }

        if method_syms.len() > limit {
            println!(
                "  ... {} more (use --limit to show more)",
                method_syms.len() - limit
            );
        }
    }

    // Fields section (properties)
    let field_syms: Vec<&Symbol> = field_syms
        .into_iter()
        .filter(|s| s.kind == "property")
        .collect();

    if !field_syms.is_empty() {
        println!();
        println!("fields:");
        for field in &field_syms {
            let sig = field.signature.as_deref().unwrap_or(&field.name);
            println!("  {sig}");
        }
    }
}

/// Print a method/function sketch.
///
/// Format:
/// ```text
/// processPayment                        method  src/payments/service.ts:34-67
/// ──────────────────────────────────────────────────────────────────────────────
/// signature:  (amount: Decimal, userId: string) → PaymentResult
/// calls:      validateCard, repo.findUser
/// called by:  OrderController.checkout [x3]
/// ```
pub fn print_method_sketch(
    symbol: &Symbol,
    outgoing_calls: &[String],
    incoming_callers: &[CallerInfo],
) {
    print_header(symbol);

    // Signature line
    if let Some(sig) = &symbol.signature {
        println!("signature:  {sig}");
    }

    // Calls line
    if !outgoing_calls.is_empty() {
        println!("calls:      {}", outgoing_calls.join(", "));
    }

    // Called by line
    if !incoming_callers.is_empty() {
        let caller_parts: Vec<String> = incoming_callers
            .iter()
            .map(|c| {
                if c.count > 1 {
                    format!("{} [x{}]", c.name, c.count)
                } else {
                    c.name.clone()
                }
            })
            .collect();
        println!("called by:  {}", caller_parts.join(", "));
    }
}

/// Print an interface sketch.
///
/// Format:
/// ```text
/// IPaymentService                          interface  src/types/payment.ts:4
/// ──────────────────────────────────────────────────────────────────────────────
/// implemented by:  PaymentService
///
/// methods:
///   processPayment(amount: Decimal, userId: string) → Promise<PaymentResult>
/// ```
pub fn print_interface_sketch(
    symbol: &Symbol,
    methods: &[Symbol],
    implementors: &[String],
    limit: usize,
) {
    print_header(symbol);

    // Implemented by
    if !implementors.is_empty() {
        println!("implemented by:  {}", implementors.join(", "));
    }

    // Methods section
    if !methods.is_empty() {
        println!();
        println!("methods:");
        let display_methods = if methods.len() > limit {
            &methods[..limit]
        } else {
            methods
        };

        for method in display_methods {
            let sig = method_display_line(method);
            println!("  {sig}");
        }

        if methods.len() > limit {
            println!(
                "  ... {} more (use --limit to show more)",
                methods.len() - limit
            );
        }
    }
}

/// Print a file-level sketch.
///
/// Format:
/// ```text
/// src/payments/service.ts
/// ──────────────────────────────────────────────────────────────────────────────
///   PaymentService          class     12-89    [11 callers]
///   processPayment          method    34-67    [11 callers]
/// ```
pub fn print_file_sketch(
    file_path: &str,
    symbols: &[Symbol],
    caller_counts: &HashMap<String, usize>,
) {
    let path = normalize_path(file_path);
    println!("{path}");
    println!("{SEPARATOR}");

    for sym in symbols {
        let line_range = format_line_range(sym.line_start, sym.line_end);
        let count = caller_counts.get(&sym.id).copied().unwrap_or(0);
        let count_label = if count > 0 {
            format!("[{count} caller{}]", if count == 1 { "" } else { "s" })
        } else {
            "[internal]".to_string()
        };
        println!(
            "  {:<24}{:<10}{:<9}{}",
            sym.name, sym.kind, line_range, count_label
        );
    }
}

/// Print a generic symbol sketch (enum, const, type, struct).
///
/// Falls back to a simple header + signature.
pub fn print_generic_sketch(symbol: &Symbol) {
    print_header(symbol);

    if let Some(sig) = &symbol.signature {
        println!("signature:  {sig}");
    }
}

/// Build the display string for a method in a class/interface listing.
///
/// Uses the signature if available, otherwise just the name.
fn method_display_line(method: &Symbol) -> String {
    method
        .signature
        .as_deref()
        .unwrap_or(&method.name)
        .to_string()
}

/// Print references to a function or method (flat list).
///
/// Format:
/// ```text
/// processPayment — 11 references
/// ──────────────────────────────────────────────────────────────────────────────
/// src/controllers/order.ts:89       OrderController.checkout
/// src/controllers/order.ts:134      OrderController.retryPayment
/// ... 8 more (use --limit to show more)
/// ```
pub fn print_refs(symbol_name: &str, refs: &[Reference], total: usize) {
    println!(
        "{} \u{2014} {} reference{}",
        symbol_name,
        total,
        if total == 1 { "" } else { "s" }
    );
    println!("{SEPARATOR}");

    for r in refs {
        let path = normalize_path(&r.file_path);
        let location = if let Some(line) = r.line {
            format!("{path}:{line}")
        } else {
            path
        };
        println!("{:<40}{}", location, r.context);
    }

    if refs.len() < total {
        println!("... {} more (use --limit to show more)", total - refs.len());
    }
}

/// Print references to a class symbol, grouped by kind.
///
/// Format:
/// ```text
/// PaymentService — 18 references
/// ──────────────────────────────────────────────────────────────────────────────
/// instantiated (4):
///   src/controllers/order.ts:23       new PaymentService(config)
///   ...
///
/// extended (1):
///   src/payments/stripe-service.ts:4  class StripeService extends PaymentService
/// ```
pub fn print_refs_grouped(symbol_name: &str, groups: &[(String, Vec<Reference>)], total: usize) {
    println!(
        "{} \u{2014} {} reference{}",
        symbol_name,
        total,
        if total == 1 { "" } else { "s" }
    );
    println!("{SEPARATOR}");

    let mut shown = 0;
    for (kind, refs) in groups {
        let kind_label = humanize_edge_kind(kind);
        println!("{kind_label} ({}):", refs.len());
        for r in refs {
            let path = normalize_path(&r.file_path);
            let location = if let Some(line) = r.line {
                format!("{path}:{line}")
            } else {
                path
            };
            println!("  {:<38}{}", location, r.context);
        }
        shown += refs.len();
        println!();
    }

    if shown < total {
        println!("... {} more (use --limit to show more)", total - shown);
    }
}

/// Print file-level references.
///
/// Same as `print_refs` but with the file path as header.
pub fn print_file_refs(file_path: &str, refs: &[Reference], total: usize) {
    let path = normalize_path(file_path);
    println!(
        "{} \u{2014} {} reference{}",
        path,
        total,
        if total == 1 { "" } else { "s" }
    );
    println!("{SEPARATOR}");

    for r in refs {
        let rpath = normalize_path(&r.file_path);
        let location = if let Some(line) = r.line {
            format!("{rpath}:{line}")
        } else {
            rpath
        };
        println!("{:<40}{}", location, r.context);
    }

    if refs.len() < total {
        println!("... {} more (use --limit to show more)", total - refs.len());
    }
}

/// Print dependencies of a symbol.
///
/// Format:
/// ```text
/// PaymentService — direct dependencies
/// ──────────────────────────────────────────────────────────────────────────────
/// imports:
///   StripeClient            src/clients/stripe.ts
///   Decimal                 (external)
///
/// calls:
///   stripe.charges.create   (external)
/// ```
pub fn print_deps(symbol_name: &str, deps: &[Dependency], max_depth: usize) {
    let depth_label = if max_depth <= 1 {
        "direct dependencies".to_string()
    } else {
        format!("transitive dependencies (depth {max_depth})")
    };

    println!("{} \u{2014} {}", symbol_name, depth_label);
    println!("{SEPARATOR}");

    if deps.is_empty() {
        println!("(no dependencies found)");
        return;
    }

    // Group by kind
    let mut groups: Vec<(String, Vec<&Dependency>)> = Vec::new();
    for dep in deps {
        if let Some(group) = groups.iter_mut().find(|(k, _)| *k == dep.kind) {
            group.1.push(dep);
        } else {
            let kind = dep.kind.clone();
            groups.push((kind, vec![dep]));
        }
    }

    for (kind, group_deps) in &groups {
        // Check if all deps in this group are external
        let all_external = group_deps.iter().all(|d| d.is_external);
        let kind_label = if all_external {
            format!("{kind} (external):")
        } else {
            format!("{kind}:")
        };
        println!("{kind_label}");

        for dep in group_deps {
            if dep.is_external {
                println!("  {:<24}(external)", dep.name);
            } else if let Some(fp) = &dep.file_path {
                let path = normalize_path(fp);
                println!("  {:<24}{}", dep.name, path);
            } else {
                println!("  {}", dep.name);
            }
        }

        println!();
    }
}

/// Print file-level dependencies.
pub fn print_file_deps(file_path: &str, deps: &[Dependency], max_depth: usize) {
    let path = normalize_path(file_path);
    let depth_label = if max_depth <= 1 {
        "direct dependencies".to_string()
    } else {
        format!("transitive dependencies (depth {max_depth})")
    };

    println!("{} \u{2014} {}", path, depth_label);
    println!("{SEPARATOR}");

    if deps.is_empty() {
        println!("(no dependencies found)");
        return;
    }

    // Group by kind
    let mut groups: Vec<(String, Vec<&Dependency>)> = Vec::new();
    for dep in deps {
        if let Some(group) = groups.iter_mut().find(|(k, _)| *k == dep.kind) {
            group.1.push(dep);
        } else {
            let kind = dep.kind.clone();
            groups.push((kind, vec![dep]));
        }
    }

    for (kind, group_deps) in &groups {
        let all_external = group_deps.iter().all(|d| d.is_external);
        let kind_label = if all_external {
            format!("{kind} (external):")
        } else {
            format!("{kind}:")
        };
        println!("{kind_label}");

        for dep in group_deps {
            if dep.is_external {
                println!("  {:<24}(external)", dep.name);
            } else if let Some(fp) = &dep.file_path {
                let fpath = normalize_path(fp);
                println!("  {:<24}{}", dep.name, fpath);
            } else {
                println!("  {}", dep.name);
            }
        }

        println!();
    }
}

/// Convert an edge kind string to a human-readable label for grouped output.
fn humanize_edge_kind(kind: &str) -> &str {
    match kind {
        "instantiates" => "instantiated",
        "extends" => "extended",
        "implements" => "implemented",
        "references_type" => "used as type",
        "imports" => "imported",
        "calls" => "called",
        "references" => "referenced",
        _ => kind,
    }
}
