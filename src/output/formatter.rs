//! Human-readable output formatting for all Scope commands.
//!
//! Rules:
//! - Separator line uses `─` (U+2500), never `-` or `=`
//! - File paths always use forward slashes, even on Windows
//! - Line ranges formatted as `start-end`
//! - Caller counts in square brackets: `[11 callers]`, `[internal]`
//! - Similarity scores always 2 decimal places: `0.91`

use std::collections::HashMap;

use crate::commands::entrypoints::EntrypointInfo;
use crate::commands::map::{CoreSymbol, DirStats, MapStats};
use crate::core::graph::{
    CallerInfo, ClassRelationships, Dependency, ImpactResult, Reference, Symbol, TraceResult,
};
use crate::core::searcher::SearchResult;

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
    show_docs: bool,
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
            // Show first line of docstring if available and docs are enabled
            if show_docs {
                if let Some(ref doc) = method.docstring {
                    let first_line = doc.lines().next().unwrap_or("").trim();
                    let clean = first_line
                        .trim_start_matches("///")
                        .trim_start_matches("//")
                        .trim_start_matches("/**")
                        .trim_start_matches("*")
                        .trim_start_matches("*/")
                        .trim();
                    if !clean.is_empty() {
                        println!("  /// {clean}");
                    }
                }
            }

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

    // Modifiers line (only if any non-default modifiers exist)
    let modifiers = extract_modifiers(&symbol.metadata);
    if !modifiers.is_empty() {
        println!("{}", modifiers.join(" "));
    }

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
/// Prepends any modifiers from metadata that are not already present in the signature.
fn method_display_line(method: &Symbol) -> String {
    let sig = method.signature.as_deref().unwrap_or(&method.name);

    let modifiers = extract_modifiers(&method.metadata);
    if modifiers.is_empty() {
        return sig.to_string();
    }

    // Only prepend modifiers not already present in the signature text
    let missing: Vec<&str> = modifiers
        .iter()
        .filter(|m| !sig.contains(m.as_str()))
        .map(|m| m.as_str())
        .collect();

    if missing.is_empty() {
        sig.to_string()
    } else {
        format!("{} {}", missing.join(" "), sig)
    }
}

/// Extract display-worthy modifiers from a symbol's metadata JSON.
///
/// Returns modifiers that differ from defaults (public is default, so omit it).
/// Example output: `vec!["async", "private", "static"]`
fn extract_modifiers(metadata_json: &str) -> Vec<String> {
    let parsed: serde_json::Value = match serde_json::from_str(metadata_json) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let mut mods = Vec::new();

    // Access modifier (only show non-public)
    if let Some(access) = parsed.get("access").and_then(|v| v.as_str()) {
        match access {
            "private" | "protected" | "internal" | "protected internal" => {
                mods.push(access.to_string());
            }
            _ => {} // "public" is default, don't show
        }
    }

    if parsed
        .get("is_async")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        mods.push("async".to_string());
    }

    if parsed
        .get("is_static")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        mods.push("static".to_string());
    }

    if parsed
        .get("is_abstract")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        mods.push("abstract".to_string());
    }

    if parsed
        .get("is_virtual")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        mods.push("virtual".to_string());
    }

    if parsed
        .get("is_override")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        mods.push("override".to_string());
    }

    mods
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
        let display_text = r.snippet_line.as_deref().unwrap_or(&r.context);
        let truncated_text = truncate_str(display_text.trim(), 80);
        println!("{:<40}{}", location, truncated_text);

        // Show multi-line context if available
        if let Some(ref snippet) = r.snippet {
            print_snippet_context(snippet, r.line);
        }
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
            let display_text = r.snippet_line.as_deref().unwrap_or(&r.context);
            let truncated_text = truncate_str(display_text.trim(), 80);
            println!("  {:<38}{}", location, truncated_text);

            // Show multi-line context if available
            if let Some(ref snippet) = r.snippet {
                print_snippet_context(snippet, r.line);
            }
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
        let display_text = r.snippet_line.as_deref().unwrap_or(&r.context);
        let truncated_text = truncate_str(display_text.trim(), 80);
        println!("{:<40}{}", location, truncated_text);

        // Show multi-line context if available
        if let Some(ref snippet) = r.snippet {
            print_snippet_context(snippet, r.line);
        }
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

/// Print an impact analysis result.
///
/// Format:
/// ```text
/// Impact analysis: processPayment
/// ──────────────────────────────────────────────────────────────────────────────
/// Direct callers (11):
///   OrderController.checkout          src/controllers/order.ts
///   SubscriptionService.renew         src/services/subscription.ts
///   ... (9 more)
///
/// Second-degree (3):
///   src/api/routes/checkout.ts        → imports OrderController
///
/// Test files affected: 6
///   tests/unit/payment.test.ts
///   tests/unit/order.test.ts
///   ... (4 more)
/// ```
pub fn print_impact(symbol_name: &str, result: &ImpactResult) {
    println!("Impact analysis: {symbol_name}");
    println!("{SEPARATOR}");

    if result.nodes_by_depth.is_empty() && result.test_files.is_empty() {
        println!("(no impact detected)");
        return;
    }

    for (depth, nodes) in &result.nodes_by_depth {
        let depth_label = impact_depth_label(*depth);
        println!("{depth_label} ({}):", nodes.len());

        let max_display = 10;
        let display_nodes = if nodes.len() > max_display {
            &nodes[..max_display]
        } else {
            nodes
        };

        for node in display_nodes {
            let path = normalize_path(&node.file_path);
            println!("  {:<40}{}", node.name, path);
        }

        if nodes.len() > max_display {
            println!("  ... ({} more)", nodes.len() - max_display);
        }

        println!();
    }

    if !result.test_files.is_empty() {
        println!("Test files affected: {}", result.test_files.len());

        let max_display = 10;
        let display_tests = if result.test_files.len() > max_display {
            &result.test_files[..max_display]
        } else {
            &result.test_files
        };

        for node in display_tests {
            let path = normalize_path(&node.file_path);
            println!("  {path}");
        }

        if result.test_files.len() > max_display {
            println!("  ... ({} more)", result.test_files.len() - max_display);
        }
    }
}

/// Print trace results showing call paths from entry points to the target.
///
/// Format:
/// ```text
/// processRenewal — 2 entry paths
/// ──────────────────────────────────────────────────────────────────────────────
/// Path 1: SubscriptionController.renewSubscription
///   └─→ SubscriptionService.processRenewal          src/services/sub.ts:72
///
/// Path 2: SubscriptionRenewalWorker.autoRenewDue
///   └─→ SubscriptionService.processRenewal          src/services/sub.ts:72
/// ```
pub fn print_trace(symbol_name: &str, result: &TraceResult, total: usize, truncated: bool) {
    let path_count = result.paths.len();
    let path_word = if path_count == 1 { "path" } else { "paths" };

    let display_count = if truncated { total } else { path_count };
    println!(
        "{} \u{2014} {} entry {}",
        symbol_name, display_count, path_word
    );
    println!("{SEPARATOR}");

    if result.paths.is_empty() {
        println!("(no entry paths found)");
        return;
    }

    for (i, call_path) in result.paths.iter().enumerate() {
        if call_path.steps.is_empty() {
            continue;
        }

        // First step: the entry point (no arrow prefix)
        let entry = &call_path.steps[0];
        let entry_name = entry.symbol_name.clone();
        println!("Path {}: {}", i + 1, entry_name);

        // Subsequent steps: indented with └─→
        for (step_idx, step) in call_path.steps.iter().enumerate().skip(1) {
            let indent = "  ".repeat(step_idx);
            let step_name = step.symbol_name.clone();
            let path = normalize_path(&step.file_path);
            let location = format!("{path}:{}", step.line);
            println!(
                "{indent}\u{2514}\u{2500}\u{2192} {:<40}{}",
                step_name, location
            );
        }

        // Blank line between paths (but not after the last one)
        if i < path_count - 1 {
            println!();
        }
    }

    if truncated {
        println!(
            "... {} more paths (use --limit to show more)",
            total - path_count
        );
    }
}

/// Print entry points grouped by type.
///
/// Format:
/// ```text
/// Entrypoints — 8 across 6 files
/// ──────────────────────────────────────────────────────────────────────────────
/// API Controllers:
///   PaymentController              src/Api/Controllers/PaymentController.cs       → 3 methods
///   SubscriptionController         src/Api/Controllers/SubscriptionController.cs  → 2 methods
///
/// Background Workers:
///   PaymentRetryWorker             src/Infrastructure/Workers/PaymentRetryWorker.cs
/// ```
pub fn print_entrypoints(
    groups: &[(String, Vec<EntrypointInfo>)],
    total: usize,
    file_count: usize,
) {
    let file_word = if file_count == 1 { "file" } else { "files" };
    println!(
        "Entrypoints \u{2014} {} across {} {}",
        total, file_count, file_word
    );
    println!("{SEPARATOR}");

    if groups.is_empty() {
        println!("(no entry points found)");
        return;
    }

    for (i, (group_name, entries)) in groups.iter().enumerate() {
        println!("{group_name}:");

        // Calculate max name width for alignment within this group.
        let max_name_len = entries
            .iter()
            .map(|e| e.name.chars().count())
            .max()
            .unwrap_or(0);
        let name_width = max_name_len.max(20) + 2; // Minimum 20 chars + padding

        for entry in entries {
            let path = normalize_path(&entry.file_path);
            let suffix = if entry.method_count > 0 {
                format!(
                    "   \u{2192} {} method{}",
                    entry.method_count,
                    if entry.method_count == 1 { "" } else { "s" }
                )
            } else {
                String::new()
            };
            println!(
                "  {:<width$}{}{}",
                entry.name,
                path,
                suffix,
                width = name_width
            );
        }

        // Blank line between groups (but not after the last one).
        if i < groups.len() - 1 {
            println!();
        }
    }
}

/// Print a structural map of the entire repository.
///
/// Format:
/// ```text
/// project-name — 181 files, 1,147 symbols, 1,409 edges
/// ──────────────────────────────────────────────────────────────────────────────
/// Languages: C#
///
/// Entry points:
///   PaymentController              Api/Controllers/                → 3 methods
///   SubscriptionController         Api/Controllers/                → 2 methods
///
/// Core symbols (by caller count):
///   ProcessPayment                 7 callers    Infrastructure/Services/PaymentService.cs
///
/// Architecture:
///   Api/                    7 files    62 symbols
///   Application/            22 files   145 symbols
/// ```
pub fn print_map(
    project_name: &str,
    stats: &MapStats,
    entrypoints: &[(String, Vec<EntrypointInfo>)],
    core_symbols: &[CoreSymbol],
    directories: &[DirStats],
) {
    // Header line: project-name — N files, N symbols, N edges
    println!(
        "{} \u{2014} {} files, {} symbols, {} edges",
        project_name,
        format_number(stats.file_count),
        format_number(stats.symbol_count),
        format_number(stats.edge_count),
    );
    println!("{SEPARATOR}");

    // Languages line.
    if !stats.languages.is_empty() {
        println!("Languages: {}", stats.languages.join(", "));
    }

    // Entry points section.
    let mut ep_count = 0usize;
    let mut ep_lines: Vec<String> = Vec::new();

    for (_group_name, entries) in entrypoints {
        for entry in entries {
            let path = normalize_path(&entry.file_path);
            // Extract directory portion of the path.
            let dir = if let Some(pos) = path.rfind('/') {
                format!("{}/", &path[..pos])
            } else {
                String::new()
            };

            // Strip leading "src/" for brevity.
            let display_dir = dir.strip_prefix("src/").unwrap_or(&dir).to_string();

            let suffix = if entry.method_count > 0 {
                format!(
                    "   \u{2192} {} method{}",
                    entry.method_count,
                    if entry.method_count == 1 { "" } else { "s" }
                )
            } else {
                String::new()
            };

            ep_lines.push(format!("  {:<32}{:<32}{}", entry.name, display_dir, suffix));
            ep_count += 1;
        }
    }

    if !ep_lines.is_empty() {
        println!();
        println!("Entry points:");
        let max_display = 8;
        for line in ep_lines.iter().take(max_display) {
            println!("{line}");
        }
        if ep_count > max_display {
            println!("  ... {} more", ep_count - max_display);
        }
    }

    // Core symbols section.
    if !core_symbols.is_empty() {
        println!();
        println!("Core symbols (by caller count):");
        for sym in core_symbols {
            let path = normalize_path(&sym.file_path);
            // Strip leading "src/" for brevity.
            let display_path = path.strip_prefix("src/").unwrap_or(&path).to_string();

            let caller_label = format!(
                "{} caller{}",
                sym.caller_count,
                if sym.caller_count == 1 { "" } else { "s" }
            );
            println!("  {:<32}{:<14}{}", sym.name, caller_label, display_path);
        }
    }

    // Architecture section.
    if !directories.is_empty() {
        println!();
        println!("Architecture:");
        for dir in directories {
            let file_label = format!(
                "{} file{}",
                dir.file_count,
                if dir.file_count == 1 { "" } else { "s" }
            );
            let sym_label = format!(
                "{} symbol{}",
                dir.symbol_count,
                if dir.symbol_count == 1 { "" } else { "s" }
            );
            println!("  {:<24}{:<14}{}", dir.directory, file_label, sym_label);
        }
    }
}

/// Human-readable label for an impact depth level.
fn impact_depth_label(depth: usize) -> &'static str {
    match depth {
        1 => "Direct callers",
        2 => "Second-degree",
        3 => "Third-degree",
        _ => "Further impact",
    }
}

/// Print incremental indexing results.
///
/// Format:
/// ```text
/// 3 files changed. Re-indexing...
///   Modified: src/payments/processor.ts
///   Added:    src/payments/refund.ts
/// Updated in 0.3s.
/// ```
pub fn print_incremental_result(
    modified: &[String],
    added: &[String],
    deleted: &[String],
    duration_secs: f64,
) {
    let total = modified.len() + added.len() + deleted.len();
    eprintln!(
        "{} file{} changed. Re-indexing...",
        total,
        if total == 1 { "" } else { "s" }
    );

    for path in modified {
        eprintln!("  Modified: {}", normalize_path(path));
    }
    for path in added {
        eprintln!("  Added:    {}", normalize_path(path));
    }
    for path in deleted {
        eprintln!("  Deleted:  {}", normalize_path(path));
    }

    eprintln!("Updated in {duration_secs:.1}s.");
}

/// Print search results from `scope find`.
///
/// Format:
/// ```text
/// Results for: "handles authentication errors"
/// ──────────────────────────────────────────────────────────────────────────────
/// 0.91  AuthMiddleware.handleUnauthorized    src/middleware/auth.ts:34      method
/// 0.88  errorHandler (auth branch)           src/api/middleware/errors.ts:67  function
/// 0.85  TokenValidator.onExpired             src/auth/token.ts:112          method
/// ```
pub fn print_find_results(query: &str, results: &[SearchResult]) {
    println!("Results for: \"{query}\"");
    println!("{SEPARATOR}");

    if results.is_empty() {
        println!("(no results found)");
        return;
    }

    for result in results {
        let path = normalize_path(&result.file_path);
        let location = format!("{path}:{}", result.line_start);
        println!(
            "{:.2}  {:<40}{:<36}  {}",
            result.score, result.name, location, result.kind
        );
    }
}

/// Print index status.
///
/// Format:
/// ```text
/// Index status: up to date
///   Symbols:    6,764
///   Files:      847
///   Edges:      12,340
///   Last index: 2 minutes ago
/// ```
pub fn print_status(
    status_label: &str,
    symbol_count: usize,
    file_count: usize,
    edge_count: usize,
    last_indexed: Option<&str>,
) {
    println!("Index status: {status_label}");
    println!("  Symbols:    {}", format_number(symbol_count));
    println!("  Files:      {}", format_number(file_count));
    println!("  Edges:      {}", format_number(edge_count));
    if let Some(relative) = last_indexed {
        println!("  Last index: {relative}");
    } else {
        println!("  Last index: never");
    }
}

/// Format a number with comma separators (e.g. 6764 -> "6,764").
fn format_number(n: usize) -> String {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let len = bytes.len();
    if len <= 3 {
        return s;
    }

    let mut result = String::with_capacity(len + len / 3);
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(ch);
    }
    result
}

/// Truncate a string to a maximum character width, adding "..." if truncated.
fn truncate_str(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars.saturating_sub(3)).collect();
        format!("{truncated}...")
    }
}

/// Print multi-line snippet context with line numbers.
///
/// Marks the reference line with `>` and other lines with a space.
fn print_snippet_context(snippet: &[String], ref_line: Option<i64>) {
    // We need to figure out what line number the first snippet line corresponds to.
    // The snippet is centered around ref_line, so first line = ref_line - (snippet.len()-1)/2 approx.
    // But we need the actual start line. We can compute it from ref_line and snippet length.
    let Some(line_num) = ref_line else { return };
    let ref_idx_in_snippet = snippet.len() / 2; // approximate center
    let start_line = (line_num as usize).saturating_sub(ref_idx_in_snippet);

    for (i, code) in snippet.iter().enumerate() {
        let current_line = start_line + i;
        let marker = if current_line == line_num as usize {
            ">"
        } else {
            " "
        };
        println!("  {marker} {current_line:>4} | {code}");
    }
}

/// Print workspace member list in human-readable format.
///
/// Shows each member's name, path, index status, file count, and symbol count.
pub fn print_workspace_list(
    workspace_name: &str,
    members: &[crate::commands::workspace::MemberStatus],
) {
    println!("Workspace: {workspace_name}");
    println!("{SEPARATOR}");

    if members.is_empty() {
        println!("  (no members)");
        return;
    }

    // Find column widths
    let max_name = members
        .iter()
        .map(|m| m.name.len())
        .max()
        .unwrap_or(4)
        .max(4);
    let max_path = members
        .iter()
        .map(|m| m.path.len())
        .max()
        .unwrap_or(4)
        .max(4);

    // Header
    println!(
        "  {:<name_w$}  {:<path_w$}  {:<15}  {:>5}  {:>7}",
        "Name",
        "Path",
        "Status",
        "Files",
        "Symbols",
        name_w = max_name,
        path_w = max_path,
    );

    for member in members {
        println!(
            "  {:<name_w$}  {:<path_w$}  {:<15}  {:>5}  {:>7}",
            member.name,
            normalize_path(&member.path),
            member.status,
            if member.file_count > 0 {
                format_number(member.file_count)
            } else {
                "─".to_string()
            },
            if member.symbol_count > 0 {
                format_number(member.symbol_count)
            } else {
                "─".to_string()
            },
            name_w = max_name,
            path_w = max_path,
        );
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
