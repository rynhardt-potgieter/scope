/// `scope similar <symbol>` — find structurally similar symbols.
///
/// Looks up the target symbol, builds a search query from its structural
/// properties (kind, name, signature, parent context), then runs FTS5
/// search to find symbols with similar characteristics. Filters out the
/// source symbol itself from results.
///
/// Examples:
///   scope similar processPayment            — find similar functions
///   scope similar PaymentService --kind class — find similar classes
use anyhow::{bail, Result};
use clap::Args;
use std::path::Path;

use crate::config::ProjectConfig;
use crate::core::graph::Graph;
use crate::core::searcher::Searcher;
use crate::output::formatter;
use crate::output::json::JsonOutput;

/// Arguments for the `scope similar` command.
#[derive(Args, Debug)]
pub struct SimilarArgs {
    /// Symbol name to find similar symbols for.
    pub symbol: String,

    /// Filter by symbol kind: function, class, method
    #[arg(long)]
    pub kind: Option<String>,

    /// Maximum number of results to show
    #[arg(long, default_value = "10")]
    pub limit: usize,

    /// Output as JSON instead of human-readable format
    #[arg(long, short = 'j')]
    pub json: bool,
}

/// Run the `scope similar` command.
pub fn run(args: &SimilarArgs, project_root: &Path) -> Result<()> {
    let scope_dir = project_root.join(".scope");

    if !scope_dir.exists() {
        bail!("No .scope/ directory found. Run 'scope init' first.");
    }

    let db_path = scope_dir.join("graph.db");
    if !db_path.exists() {
        bail!("No index found. Run 'scope index' to build one first.");
    }

    let graph = Graph::open(&db_path)?;
    let searcher = Searcher::open(&db_path)?;

    // Look up the source symbol
    let symbol = graph.find_symbol(&args.symbol)?.ok_or_else(|| {
        anyhow::anyhow!(
            "Symbol '{}' not found in index.\n\
             Tip: Check spelling, or use 'scope find \"{}\"' for semantic search.",
            args.symbol,
            args.symbol
        )
    })?;

    // Build a search query from the symbol's structural properties.
    // Include kind, name parts, signature, and parent context so FTS5
    // finds symbols with similar shape.
    let mut query_parts = Vec::new();
    query_parts.push(format!("{} {}", symbol.kind, symbol.name));

    // Split camelCase/snake_case for broader matching
    let split = crate::core::embedder::split_camel_case(&symbol.name);
    if split != symbol.name {
        query_parts.push(split);
    }
    let snake = crate::core::embedder::split_snake_case(&symbol.name);
    if snake != symbol.name {
        query_parts.push(snake);
    }

    if let Some(sig) = &symbol.signature {
        query_parts.push(sig.clone());
    }

    let query = query_parts.join(" ");

    // Use kind filter: prefer the user's --kind flag, otherwise match the source symbol's kind
    let kind_filter = args.kind.as_deref().or(Some(symbol.kind.as_str()));

    // Load vendor patterns for de-ranking
    let vendor_patterns = ProjectConfig::load(&scope_dir)
        .map(|c| c.index.vendor_patterns)
        .unwrap_or_default();

    // Fetch one extra result to account for filtering out the source symbol
    let mut results = searcher.search_with_vendor_derank(
        &query,
        args.limit + 1,
        kind_filter,
        &vendor_patterns,
    )?;

    // Remove the source symbol itself from results
    results.retain(|r| r.id != symbol.id);
    results.truncate(args.limit);

    if args.json {
        let total = results.len();
        let output = JsonOutput {
            command: "similar",
            symbol: Some(args.symbol.clone()),
            data: &results,
            truncated: false,
            total,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        formatter::print_find_results(&format!("similar to {}", args.symbol), &results);
    }

    Ok(())
}
