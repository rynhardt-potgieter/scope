/// `sc find <query>` — find code by intent using semantic search.
///
/// Uses embeddings to find symbols by what they do, not what they are called.
/// Returns ranked results with similarity scores.
use anyhow::Result;
use clap::Args;

/// Arguments for the `sc find` command.
#[derive(Args, Debug)]
pub struct FindArgs {
    /// Natural language search query.
    ///
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

/// Run the `sc find` command (stub).
pub fn run(args: &FindArgs) -> Result<()> {
    eprintln!("find command not yet implemented. Query: {}", args.query);
    Ok(())
}
