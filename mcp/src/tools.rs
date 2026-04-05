//! MCP tool definitions for Scope CLI.
//!
//! Each tool maps to a scope command. Parameters are validated by the
//! MCP framework via `schemars::JsonSchema` derive.

use std::path::PathBuf;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use rmcp::{schemars, tool, tool_router};

use crate::runner;

// ---------------------------------------------------------------------------
// Parameter structs
// ---------------------------------------------------------------------------

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SymbolParam {
    #[schemars(description = "Symbol name (e.g. PaymentService, Graph.find_symbol)")]
    pub symbol: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FindParam {
    #[schemars(description = "Natural language search query")]
    pub query: String,
    #[schemars(description = "Filter by symbol kind: function, class, method, interface")]
    pub kind: Option<String>,
    #[schemars(description = "Maximum results (default: 10)")]
    pub limit: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RefsParam {
    #[schemars(description = "Symbol name")]
    pub symbol: String,
    #[schemars(description = "Filter by reference kind: calls, imports, extends, implements")]
    pub kind: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CallersParam {
    #[schemars(description = "Symbol name")]
    pub symbol: String,
    #[schemars(description = "Depth for transitive callers (default: 1, use 2+ for blast radius)")]
    pub depth: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DepsParam {
    #[schemars(description = "Symbol name")]
    pub symbol: String,
    #[schemars(description = "Depth for transitive deps (default: 1)")]
    pub depth: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DiffParam {
    #[schemars(description = "Git ref to compare against (default: HEAD)")]
    pub git_ref: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TraceParam {
    #[schemars(description = "Symbol name to trace entry paths to")]
    pub symbol: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FlowParam {
    #[schemars(description = "Source symbol — where the path starts")]
    pub start: String,
    #[schemars(description = "Target symbol — where the path ends")]
    pub end: String,
    #[schemars(description = "Maximum path length (default: 10)")]
    pub depth: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MapParam {
    #[schemars(description = "Maximum core symbols to show (default: 10)")]
    pub limit: Option<usize>,
}

// ---------------------------------------------------------------------------
// Server struct
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ScopeMcp {
    pub scope_bin: PathBuf,
    pub project_root: PathBuf,
    pub tool_router: ToolRouter<Self>,
}

impl ScopeMcp {
    pub fn new(scope_bin: PathBuf, project_root: PathBuf) -> Self {
        Self {
            scope_bin,
            project_root,
            tool_router: Self::tool_router(),
        }
    }
}

// ---------------------------------------------------------------------------
// Tool implementations
// ---------------------------------------------------------------------------

#[tool_router]
impl ScopeMcp {
    #[tool(
        description = "Check scope index status — symbol count, file count, freshness. Run first to verify scope is available."
    )]
    async fn scope_status(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        self.run(&["status", "--json"]).await
    }

    #[tool(
        description = "Repository overview — entry points, core symbols, architecture. Use at the start of complex tasks (~500 tokens)."
    )]
    async fn scope_map(
        &self,
        Parameters(p): Parameters<MapParam>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let mut args = vec!["map", "--compact"];
        let limit_str;
        if let Some(limit) = p.limit {
            limit_str = limit.to_string();
            args.extend(["--limit", &limit_str]);
        }
        self.run(&args).await
    }

    #[tool(
        description = "Structural overview of a symbol — methods, caller counts, deps, signatures (~200 tokens). Use before editing."
    )]
    async fn scope_sketch(
        &self,
        Parameters(p): Parameters<SymbolParam>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.run(&["sketch", "--compact", "--", &p.symbol]).await
    }

    #[tool(
        description = "One-line symbol summary — name, kind, file:line, callers, deps (~30 tokens). Quick 'what is this?' check."
    )]
    async fn scope_summary(
        &self,
        Parameters(p): Parameters<SymbolParam>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.run(&["summary", "--json", "--", &p.symbol]).await
    }

    #[tool(
        description = "Fetch full source code of a symbol. Use after scope_sketch when you need the actual implementation."
    )]
    async fn scope_source(
        &self,
        Parameters(p): Parameters<SymbolParam>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.run(&["source", "--json", "--", &p.symbol]).await
    }

    #[tool(
        description = "Search symbols by intent — finds code by what it does, not just what it's named. Returns ranked results."
    )]
    async fn scope_find(
        &self,
        Parameters(p): Parameters<FindParam>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let mut args = vec!["find", "--json"];
        let kind_str;
        let limit_str;
        if let Some(ref kind) = p.kind {
            kind_str = kind.clone();
            args.extend(["--kind", &kind_str]);
        }
        if let Some(limit) = p.limit {
            limit_str = limit.to_string();
            args.extend(["--limit", &limit_str]);
        }
        args.extend(["--", &p.query]);
        self.run(&args).await
    }

    #[tool(
        description = "Find all references to a symbol — call sites, imports, type annotations. Use before changing signatures."
    )]
    async fn scope_refs(
        &self,
        Parameters(p): Parameters<RefsParam>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let mut args = vec!["refs", "--json"];
        let kind_str;
        if let Some(ref kind) = p.kind {
            kind_str = kind.clone();
            args.extend(["--kind", &kind_str]);
        }
        args.extend(["--", &p.symbol]);
        self.run(&args).await
    }

    #[tool(
        description = "Show callers of a symbol. depth=1 for direct, depth=2+ for transitive blast radius analysis."
    )]
    async fn scope_callers(
        &self,
        Parameters(p): Parameters<CallersParam>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let mut args = vec!["callers", "--json"];
        let depth_str;
        if let Some(depth) = p.depth {
            depth_str = depth.to_string();
            args.extend(["--depth", &depth_str]);
        }
        args.extend(["--", &p.symbol]);
        self.run(&args).await
    }

    #[tool(
        description = "Show what a symbol depends on — imports, calls, type references. Understand prerequisites."
    )]
    async fn scope_deps(
        &self,
        Parameters(p): Parameters<DepsParam>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let mut args = vec!["deps", "--json"];
        let depth_str;
        if let Some(depth) = p.depth {
            depth_str = depth.to_string();
            args.extend(["--depth", &depth_str]);
        }
        args.extend(["--", &p.symbol]);
        self.run(&args).await
    }

    #[tool(
        description = "Show symbols in git-changed files — cross-references git diff with the index. Use for PR review."
    )]
    async fn scope_diff(
        &self,
        Parameters(p): Parameters<DiffParam>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let mut args = vec!["diff", "--json"];
        let ref_str;
        if let Some(ref git_ref) = p.git_ref {
            ref_str = git_ref.clone();
            args.extend(["--ref", &ref_str]);
        }
        self.run(&args).await
    }

    #[tool(
        description = "Trace call paths from entry points to a symbol. Shows how requests reach a function. Use for debugging."
    )]
    async fn scope_trace(
        &self,
        Parameters(p): Parameters<TraceParam>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.run(&["trace", "--json", "--", &p.symbol]).await
    }

    #[tool(
        description = "Find call paths between two symbols. Use to understand how A connects to B through the call graph."
    )]
    async fn scope_flow(
        &self,
        Parameters(p): Parameters<FlowParam>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let mut args = vec!["flow", "--json"];
        let depth_str;
        if let Some(depth) = p.depth {
            depth_str = depth.to_string();
            args.extend(["--depth", &depth_str]);
        }
        args.extend(["--", &p.start, &p.end]);
        self.run(&args).await
    }
}

// ---------------------------------------------------------------------------
// Shared runner helper
// ---------------------------------------------------------------------------

impl ScopeMcp {
    async fn run(&self, args: &[&str]) -> Result<CallToolResult, rmcp::ErrorData> {
        match runner::run_scope(&self.scope_bin, args, &self.project_root).await {
            Ok(data) => {
                // Value serialization is infallible (no non-string map keys possible).
                let text = serde_json::to_string_pretty(&data)
                    .expect("serde_json::Value serialization is infallible");
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(msg) => Ok(CallToolResult::error(vec![Content::text(msg)])),
        }
    }
}
