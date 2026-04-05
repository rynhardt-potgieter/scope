//! MCP server for Scope code intelligence CLI.
//!
//! Wraps `scope` commands as MCP tools over stdio transport.
//! Each tool spawns `scope <cmd> --json` and returns structured results.
//!
//! Usage:
//!   scope-mcp                      # stdio mode (for MCP clients)
//!   SCOPE_BIN=/path/to/scope scope-mcp  # custom scope binary location

mod runner;
mod tools;

use rmcp::model::*;
use rmcp::{tool_handler, ServerHandler, ServiceExt};
use tools::ScopeMcp;
use tracing_subscriber::EnvFilter;

#[tool_handler(router = self.tool_router)]
impl ServerHandler for ScopeMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Scope provides code intelligence for LLM agents. \
                 Use scope_status first to check index health. \
                 Use scope_map for repo overview, scope_summary for a quick \
                 'what is this?' (~30 tokens), scope_sketch for full structure \
                 before editing (~200 tokens), scope_find for search, \
                 scope_callers for blast radius."
                    .to_string(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Logging goes to stderr (stdout is the MCP transport).
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    // Find the scope binary.
    let scope_bin = runner::find_scope_bin().map_err(|e| anyhow::anyhow!(e))?;
    tracing::info!("Using scope binary: {}", scope_bin.display());

    // Resolve project root from CWD.
    let project_root = std::env::current_dir()?;
    tracing::info!("Project root: {}", project_root.display());

    // Create the MCP server and serve over stdio.
    let service = ScopeMcp::new(scope_bin, project_root);
    let server = service.serve(rmcp::transport::stdio()).await?;

    tracing::info!("scope-mcp server running on stdio");
    server.waiting().await?;

    Ok(())
}
