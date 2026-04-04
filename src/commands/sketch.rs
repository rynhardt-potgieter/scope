/// `scope sketch <symbol>` — show structural overview of a symbol.
///
/// Returns the class/function signature, dependencies, methods with caller counts,
/// and type information. Use this before `scope source` to understand structure first.
///
/// Examples:
///   scope sketch PaymentService              — sketch a class
///   scope sketch PaymentService.processPayment  — sketch a method
///   scope sketch src/payments/service.ts     — sketch a whole file
use anyhow::{bail, Result};
use clap::Args;
use std::path::Path;

use crate::commands::warn_if_stale;
use crate::core::graph::{Graph, Symbol};
use crate::output::formatter;
use crate::output::json::JsonOutput;

/// Strip internal fields from a Symbol for compact JSON output.
/// Keeps only what agents need: name, kind, signature, line_start, line_end.
fn compact_symbol(s: &Symbol) -> serde_json::Value {
    serde_json::json!({
        "name": s.name,
        "kind": s.kind,
        "signature": s.signature,
        "file_path": s.file_path,
        "line_start": s.line_start,
        "line_end": s.line_end,
    })
}

/// Compact a Vec of Symbol references.
fn compact_symbols(syms: &[&Symbol]) -> Vec<serde_json::Value> {
    syms.iter().map(|s| compact_symbol(s)).collect()
}

/// Arguments for the `scope sketch` command.
#[derive(Args, Debug)]
pub struct SketchArgs {
    /// Symbol name or file path to sketch.
    ///
    /// Pass a class name to see its methods, deps, and inheritance.
    /// Pass a method name to see its signature, callers, and callees.
    /// Pass Class.method for qualified lookup.
    /// Pass a file path to see all symbols in that file.
    ///
    /// Examples: PaymentService, PaymentService.processPayment, src/payments/service.ts
    pub symbol: String,

    /// Output as JSON instead of human-readable format
    #[arg(long, short = 'j')]
    pub json: bool,

    /// Maximum number of methods to show (default: all)
    #[arg(long, default_value = "50")]
    pub limit: usize,

    /// Suppress docstring display in sketch output
    #[arg(long)]
    pub no_docs: bool,

    /// Treat the argument as a file path (sketch all symbols in the file).
    ///
    /// Useful when the path doesn't contain `/` and would otherwise be
    /// treated as a symbol name.
    #[arg(long)]
    pub file: bool,

    /// Emit compact JSON (strips internal IDs, raw metadata, language).
    ///
    /// Reduces token cost by ~70% for LLM agents that only need
    /// name, kind, signature, and line numbers. Implies --json.
    #[arg(long)]
    pub compact: bool,
}

/// Returns true if the input looks like a file path rather than a symbol name.
use super::looks_like_file_path;

/// Run the `scope sketch` command.
pub fn run(args: &SketchArgs, project_root: &Path) -> Result<()> {
    let scope_dir = project_root.join(".scope");

    if !scope_dir.exists() {
        bail!("No .scope/ directory found. Run 'scope init' first.");
    }

    let db_path = scope_dir.join("graph.db");
    if !db_path.exists() {
        bail!("No index found. Run 'scope index' to build one first.");
    }

    let graph = Graph::open(&db_path)?;
    warn_if_stale(&graph, project_root);

    if args.file || looks_like_file_path(&args.symbol) {
        return run_file_sketch(args, &graph);
    }

    run_symbol_sketch(args, &graph)
}

/// Sketch a single symbol (class, method, interface, etc.).
fn run_symbol_sketch(args: &SketchArgs, graph: &Graph) -> Result<()> {
    let symbol = graph.find_symbol(&args.symbol)?.ok_or_else(|| {
        anyhow::anyhow!(
            "Symbol '{}' not found in index.\n\
             Tip: Check spelling, or use 'scope find \"{}\"' for semantic search.",
            args.symbol,
            args.symbol
        )
    })?;

    match symbol.kind.as_str() {
        "class" | "struct" => sketch_class(args, graph, &symbol),
        "method" | "function" => sketch_method(args, graph, &symbol),
        "interface" => sketch_interface(args, graph, &symbol),
        "enum" => sketch_enum(args, graph, &symbol),
        _ => sketch_generic(args, &symbol),
    }
}

/// Sketch a class or struct.
fn sketch_class(
    args: &SketchArgs,
    graph: &Graph,
    symbol: &crate::core::graph::Symbol,
) -> Result<()> {
    let methods = graph.get_methods(&symbol.id)?;
    let relationships = graph.get_class_relationships(&symbol.id)?;

    // Batch-fetch caller counts for all methods
    let method_ids: Vec<&str> = methods.iter().map(|m| m.id.as_str()).collect();
    let caller_counts = graph.get_caller_counts(&method_ids)?;

    if args.json || args.compact {
        let (fields, actual_methods): (Vec<_>, Vec<_>) =
            methods.iter().partition(|m| m.kind == "property");
        let field_data: Vec<serde_json::Value> = fields
            .iter()
            .map(|f| {
                serde_json::json!({
                    "name": f.name,
                    "signature": f.signature,
                    "line_start": f.line_start,
                })
            })
            .collect();

        let (sym_data, method_data) = if args.compact {
            (
                compact_symbol(symbol),
                serde_json::json!(compact_symbols(&actual_methods)),
            )
        } else {
            (
                serde_json::json!(symbol),
                serde_json::json!(actual_methods),
            )
        };

        let data = serde_json::json!({
            "symbol": sym_data,
            "methods": method_data,
            "fields": field_data,
            "caller_counts": caller_counts,
            "relationships": relationships,
        });
        let output = JsonOutput {
            command: "sketch",
            symbol: Some(symbol.name.clone()),
            data,
            truncated: methods.len() > args.limit,
            total: methods.len(),
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        formatter::print_class_sketch(
            symbol,
            &methods,
            &caller_counts,
            &relationships,
            args.limit,
            !args.no_docs,
        );
    }

    Ok(())
}

/// Sketch a method or function.
fn sketch_method(
    args: &SketchArgs,
    graph: &Graph,
    symbol: &crate::core::graph::Symbol,
) -> Result<()> {
    let outgoing_calls = graph.get_outgoing_calls(&symbol.id)?;
    let incoming_callers = graph.get_incoming_callers(&symbol.id)?;

    if args.json || args.compact {
        let sym = if args.compact { compact_symbol(symbol) } else { serde_json::json!(symbol) };
        let data = serde_json::json!({
            "symbol": sym,
            "calls": outgoing_calls,
            "called_by": incoming_callers,
        });
        let output = JsonOutput {
            command: "sketch",
            symbol: Some(symbol.name.clone()),
            data,
            truncated: false,
            total: 1,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        formatter::print_method_sketch(symbol, &outgoing_calls, &incoming_callers);
    }

    Ok(())
}

/// Sketch an interface.
fn sketch_interface(
    args: &SketchArgs,
    graph: &Graph,
    symbol: &crate::core::graph::Symbol,
) -> Result<()> {
    let methods = graph.get_methods(&symbol.id)?;
    let implementors = graph.get_implementors(&symbol.id)?;

    if args.json || args.compact {
        let sym = if args.compact { compact_symbol(symbol) } else { serde_json::json!(symbol) };
        let meths = if args.compact { serde_json::json!(compact_symbols(&methods.iter().collect::<Vec<_>>())) } else { serde_json::json!(methods) };
        let data = serde_json::json!({
            "symbol": sym,
            "methods": meths,
            "implementors": implementors,
        });
        let output = JsonOutput {
            command: "sketch",
            symbol: Some(symbol.name.clone()),
            data,
            truncated: methods.len() > args.limit,
            total: methods.len(),
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        formatter::print_interface_sketch(symbol, &methods, &implementors, args.limit);
    }

    Ok(())
}

/// Sketch an enum — shows variants and caller count.
fn sketch_enum(
    args: &SketchArgs,
    graph: &Graph,
    symbol: &crate::core::graph::Symbol,
) -> Result<()> {
    // get_methods returns all children by parent_id; filter for variants
    let children = graph.get_methods(&symbol.id)?;
    let variants: Vec<&crate::core::graph::Symbol> =
        children.iter().filter(|c| c.kind == "variant").collect();
    let caller_count = graph.get_caller_count(&symbol.id)?;

    if args.json || args.compact {
        let variant_data: Vec<serde_json::Value> = variants
            .iter()
            .map(|v| {
                serde_json::json!({
                    "name": v.name,
                    "signature": v.signature,
                    "line_start": v.line_start,
                    "line_end": v.line_end,
                })
            })
            .collect();
        let sym = if args.compact { compact_symbol(symbol) } else { serde_json::json!(symbol) };
        let data = serde_json::json!({
            "symbol": sym,
            "variants": variant_data,
            "caller_count": caller_count,
        });
        let output = JsonOutput {
            command: "sketch",
            symbol: Some(symbol.name.clone()),
            data,
            truncated: false,
            total: 1,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        formatter::print_enum_sketch(symbol, &variants, caller_count);
    }

    Ok(())
}

/// Sketch a generic symbol (const, type).
fn sketch_generic(args: &SketchArgs, symbol: &crate::core::graph::Symbol) -> Result<()> {
    if args.json || args.compact {
        let sym = if args.compact { compact_symbol(symbol) } else { serde_json::json!(symbol) };
        let data = serde_json::json!({
            "symbol": sym,
        });
        let output = JsonOutput {
            command: "sketch",
            symbol: Some(symbol.name.clone()),
            data,
            truncated: false,
            total: 1,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        formatter::print_generic_sketch(symbol);
    }

    Ok(())
}

/// Sketch all symbols in a file.
fn run_file_sketch(args: &SketchArgs, graph: &Graph) -> Result<()> {
    let file_path = formatter::normalize_path(&args.symbol);
    let symbols = graph.get_file_symbols(&file_path)?;

    if symbols.is_empty() {
        bail!(
            "No symbols found for file '{}'.\n\
             Tip: Check the path is relative to the project root. Run 'scope index' if the file is new.",
            file_path
        );
    }

    // Batch-fetch caller counts for all symbols in the file
    let symbol_ids: Vec<&str> = symbols.iter().map(|s| s.id.as_str()).collect();
    let caller_counts = graph.get_caller_counts(&symbol_ids)?;

    if args.json || args.compact {
        let sym_data = if args.compact {
            serde_json::json!(symbols.iter().map(|s| compact_symbol(s)).collect::<Vec<_>>())
        } else {
            serde_json::json!(symbols)
        };
        let data = serde_json::json!({
            "file_path": file_path,
            "symbols": sym_data,
            "caller_counts": caller_counts,
        });
        let output = JsonOutput {
            command: "sketch",
            symbol: Some(file_path.clone()),
            data,
            truncated: false,
            total: symbols.len(),
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        formatter::print_file_sketch(&file_path, &symbols, &caller_counts);
    }

    Ok(())
}
