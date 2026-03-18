/// `sc find <query>` — find code by intent using full-text search.
///
/// Searches the symbol index for symbols matching a natural-language query.
/// Uses FTS5 with BM25 ranking to return the most relevant results.
/// Returns ranked results with similarity scores.
///
/// Examples:
///   sc find "handles authentication errors"
///   sc find "payment processing" --kind method
///   sc find "validates user input" --limit 5 --json
use anyhow::{bail, Result};
use clap::Args;
use std::path::Path;

use crate::core::searcher::Searcher;
use crate::output::formatter;
use crate::output::json::JsonOutput;

/// Arguments for the `sc find` command.
#[derive(Args, Debug)]
pub struct FindArgs {
    /// Natural language search query.
    ///
    /// Searches symbol names, signatures, and docstrings.
    /// Examples: "handles authentication errors", "sends email notifications"
    pub query: String,

    /// Filter by symbol kind: function, class, method, interface
    #[arg(long)]
    pub kind: Option<String>,

    /// Filter by language: typescript, csharp, python
    #[arg(long)]
    pub lang: Option<String>,

    /// Maximum number of results to show
    #[arg(long, default_value = "10")]
    pub limit: usize,

    /// Output as JSON instead of human-readable format
    #[arg(long, short = 'j')]
    pub json: bool,
}

/// Run the `sc find` command.
///
/// Opens the search index, runs the query, and prints results.
pub fn run(args: &FindArgs, project_root: &Path) -> Result<()> {
    let scope_dir = project_root.join(".scope");

    if !scope_dir.exists() {
        bail!("No .scope/ directory found. Run 'sc init' first.");
    }

    let db_path = scope_dir.join("graph.db");
    if !db_path.exists() {
        bail!("No index found. Run 'sc index' to build one first.");
    }

    let searcher = Searcher::open(&db_path)?;

    let results = searcher.search(&args.query, args.limit, args.kind.as_deref())?;

    if args.json {
        let total = results.len();
        let output = JsonOutput {
            command: "find",
            symbol: None,
            data: &results,
            truncated: false,
            total,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        formatter::print_find_results(&args.query, &results);
    }

    Ok(())
}
