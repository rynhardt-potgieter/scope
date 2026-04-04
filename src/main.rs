//! Scope — Code intelligence CLI for LLM coding agents.
//!
//! Builds a local code intelligence index and lets you query it efficiently.
//! Use it before editing any non-trivial code to understand structure,
//! dependencies, and blast radius.
use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

mod commands;
mod config;
mod core;
mod languages;
mod output;

/// Code intelligence CLI for LLM coding agents.
///
/// Scope builds a local code intelligence index and lets you query
/// it efficiently. Use it before editing any non-trivial code to
/// understand structure, dependencies, and blast radius.
#[derive(Parser, Debug)]
#[command(
    name = "scope",
    about = "Code intelligence CLI for LLM coding agents",
    long_about = "Scope builds a local code intelligence index and lets you query \
                  it efficiently. Use it before editing any non-trivial code to \
                  understand structure, dependencies, and blast radius.",
    version,
    propagate_version = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose logging
    #[arg(long, global = true)]
    pub verbose: bool,

    /// Query across all workspace members (requires scope-workspace.toml).
    ///
    /// When set, commands like map, refs, find, entrypoints, and status
    /// fan out to all projects in the workspace and merge results.
    /// Requires a scope-workspace.toml manifest in the current directory
    /// or a parent directory.
    #[arg(long, global = true)]
    pub workspace: bool,

    /// Target a specific workspace member by name.
    ///
    /// In workspace context, restricts queries to the named project.
    /// Use `scope workspace list` to see available member names.
    #[arg(long, global = true)]
    pub project: Option<String>,
}

/// All available subcommands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialise Scope for this project.
    ///
    /// Creates a .scope/ directory with default configuration.
    /// Auto-detects languages from project markers (tsconfig.json, .csproj).
    /// Run this once per project before running `scope index`.
    Init(commands::init::InitArgs),

    /// Build or refresh the code index.
    ///
    /// Walks all source files, parses them with tree-sitter, and stores
    /// symbols and edges in the local SQLite graph database. First run
    /// is always a full index. Subsequent runs can be incremental.
    Index(commands::index::IndexArgs),

    /// Show structural overview of a symbol without reading full source.
    ///
    /// Returns the class/function signature, dependencies, methods with caller
    /// counts, and type information. Use this before `scope source` to understand
    /// structure first.
    ///
    /// Examples:
    ///   scope sketch PaymentService              — sketch a class
    ///   scope sketch PaymentService.processPayment  — sketch a method
    ///   scope sketch src/payments/service.ts     — sketch a whole file
    Sketch(commands::sketch::SketchArgs),

    /// Find all references to a symbol across the codebase.
    ///
    /// Returns call sites, imports, type annotations, and other references.
    /// Use before changing a function signature to find all callers.
    ///
    /// Examples:
    ///   scope refs processPayment              — all references
    ///   scope refs PaymentService --kind calls  — only call sites
    Refs(commands::refs::RefsArgs),

    /// Show all callers of a function or method.
    ///
    /// At depth 1 (default): equivalent to `scope refs <symbol> --kind calls`.
    /// At depth 2+: performs transitive impact analysis showing callers of callers.
    ///
    /// Examples:
    ///   scope callers processPayment              — direct callers only
    ///   scope callers processPayment --depth 2    — callers + callers-of-callers
    ///   scope callers processPayment --context 2  — with surrounding code (depth 1)
    Callers(commands::refs::CallersArgs),

    /// Show what a symbol depends on.
    ///
    /// Lists direct imports, calls, and type references. Use --depth 2
    /// for transitive dependencies.
    ///
    /// Examples:
    ///   scope deps PaymentService               — direct dependencies
    ///   scope deps PaymentService --depth 2     — transitive dependencies
    Deps(commands::deps::DepsArgs),

    /// Show what depends on a symbol (reverse dependencies).
    ///
    /// Critical before any refactor or deletion. Shows all symbols
    /// and files that depend on the given symbol.
    ///
    /// Examples:
    ///   scope rdeps PaymentService              — what uses this class
    ///   scope rdeps PaymentConfig --depth 2     — transitive reverse deps
    Rdeps(commands::rdeps::RdepsArgs),

    /// Analyse blast radius if a symbol changes.
    ///
    /// Performs transitive reverse dependency traversal. Shows direct
    /// callers, second-degree dependents, and affected test files.
    ///
    /// Examples:
    ///   scope impact processPayment             — who breaks if this changes
    ///   scope impact PaymentConfig              — blast radius of config change
    Impact(commands::impact::ImpactArgs),

    /// Show which symbols changed since a git ref.
    ///
    /// Cross-references `git diff --name-only` with the index to show
    /// exactly which symbols were added, modified, or deleted.
    /// Designed for code review and PR triage.
    ///
    /// Examples:
    ///   scope diff                     — changes since last commit
    ///   scope diff --ref main          — changes vs main branch
    ///   scope diff --ref HEAD~3 --json — last 3 commits, JSON output
    Diff(commands::diff::DiffArgs),

    /// Find call paths between two symbols.
    ///
    /// Shows how <start> reaches <end> through the call graph.
    /// Use this when you need to understand how data or control flows
    /// between two specific points in the codebase.
    ///
    /// Unlike `scope trace` (entry points → target), this finds paths
    /// between any two symbols.
    ///
    /// Examples:
    ///   scope flow PaymentService OrderController
    ///   scope flow processPayment handleWebhook --depth 5
    ///   scope flow "src/auth.ts::validate" "src/api.ts::respond" --json
    Flow(commands::flow::FlowArgs),

    /// Find code by intent using semantic search.
    ///
    /// Uses embeddings to find symbols by what they do, not what they
    /// are called. Returns ranked results with similarity scores.
    ///
    /// Examples:
    ///   scope find "handles authentication errors"
    ///   scope find "sends email notifications" --kind function
    Find(commands::find::FindArgs),

    /// Find structurally similar symbols.
    ///
    /// Uses embeddings to find symbols with similar structure or semantics.
    /// Useful for discovering existing implementations before writing new code.
    ///
    /// Examples:
    ///   scope similar processPayment            — find similar functions
    ///   scope similar PaymentService --kind class — find similar classes
    Similar(commands::similar::SimilarArgs),

    /// One-line summary of a symbol (~30 tokens).
    ///
    /// Returns name, kind, location, signature, caller count, and dependency
    /// count on a single line. Use when an agent just needs "what is this?"
    /// without the full sketch output.
    ///
    /// Examples:
    ///   scope summary PaymentService
    ///   scope summary Graph.find_symbol
    Summary(commands::summary::SummaryArgs),

    /// Fetch full source of a specific symbol.
    ///
    /// Returns the exact source code of the symbol. Only call this when
    /// ready to read or edit the implementation — use `scope sketch` first.
    ///
    /// Examples:
    ///   scope source processPayment
    ///   scope source PaymentService.validateCard
    Source(commands::source::SourceArgs),

    /// Trace call paths from entry points to a symbol.
    ///
    /// Shows how API endpoints, workers, and event handlers reach the
    /// target method through the call graph. Useful for understanding
    /// how a bug is triggered or what code paths exercise a function.
    ///
    /// Examples:
    ///   scope trace processPayment
    ///   scope trace SubscriptionService.processRenewal
    Trace(commands::trace::TraceArgs),

    /// List entry points — API controllers, workers, and event handlers.
    ///
    /// Shows symbols with no incoming call edges, grouped by type.
    /// These are the starting points for request flows: HTTP endpoints,
    /// background workers, event handlers, and standalone functions.
    ///
    /// Examples:
    ///   scope entrypoints
    ///   scope entrypoints --json
    Entrypoints(commands::entrypoints::EntrypointsArgs),

    /// Show a structural overview of the repository.
    ///
    /// Displays entry points, core symbols ranked by importance,
    /// architecture layers, and key statistics. Gives an LLM agent
    /// a complete mental model of the codebase in ~500-1000 tokens,
    /// replacing multiple scope sketch calls.
    ///
    /// Examples:
    ///   scope map
    ///   scope map --limit 5
    ///   scope map --json
    Map(commands::map::MapArgs),

    /// Show index status and freshness.
    ///
    /// Quick health check: is the index built? How many symbols and files?
    /// Are there stale or unindexed files?
    Status(commands::status::StatusArgs),

    /// One-command agent integration setup.
    ///
    /// Runs init + index + writes CLAUDE.md snippet + installs skill file.
    /// With --preload, bakes `scope map` into CLAUDE.md for 32% agent cost savings.
    ///
    /// Examples:
    ///   scope setup              — full setup
    ///   scope setup --preload    — setup with architecture preloading
    Setup(commands::setup::SetupArgs),

    /// Manage multi-project workspaces.
    ///
    /// A workspace groups multiple Scope projects and enables federated
    /// queries across all members. Use `scope workspace init` to discover
    /// projects and create a scope-workspace.toml manifest.
    ///
    /// Examples:
    ///   scope workspace init              — discover and create manifest
    ///   scope workspace list              — show all members and status
    ///   scope workspace list --json       — machine-readable output
    Workspace(commands::workspace::WorkspaceArgs),
}

/// The resolved execution context: single project or workspace.
pub enum Context {
    /// Standard single-project mode. CWD has a .scope/ directory (or will create one).
    SingleProject {
        /// Absolute path to the project root.
        root: PathBuf,
    },
    /// Workspace mode. A scope-workspace.toml was found.
    Workspace {
        /// Path to the scope-workspace.toml file.
        manifest_path: PathBuf,
        /// Workspace root directory (parent of the manifest).
        workspace_root: PathBuf,
        /// Parsed workspace configuration.
        config: config::workspace::WorkspaceConfig,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialise tracing
    let level = if cli.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::WARN
    };

    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_writer(std::io::stderr)
        .init();

    // Resolve context based on flags
    let ctx = resolve_context(cli.workspace, cli.project.as_deref())?;

    match &cli.command {
        // --- Commands that SUPPORT workspace mode ---
        Commands::Status(args) => commands::status::run(args, &ctx),
        Commands::Map(args) => commands::map::run(args, &ctx),
        Commands::Refs(args) => commands::refs::run(args, &ctx),
        Commands::Find(args) => commands::find::run(args, &ctx),
        Commands::Entrypoints(args) => commands::entrypoints::run(args, &ctx),

        // --- Commands that operate on a single project only ---
        Commands::Init(args) => {
            let root = project_root_from_context(&ctx)?;
            commands::init::run(args, root)
        }
        Commands::Index(args) => {
            let root = project_root_from_context(&ctx)?;
            commands::index::run(args, root)
        }
        Commands::Sketch(args) => {
            let root = project_root_from_context(&ctx)?;
            commands::sketch::run(args, root)
        }
        Commands::Callers(args) => {
            let root = project_root_from_context(&ctx)?;
            commands::refs::run_callers(args, root)
        }
        Commands::Deps(args) => {
            let root = project_root_from_context(&ctx)?;
            commands::deps::run(args, root)
        }
        Commands::Rdeps(args) => {
            let root = project_root_from_context(&ctx)?;
            commands::rdeps::run(args, root)
        }
        Commands::Impact(args) => {
            let root = project_root_from_context(&ctx)?;
            commands::impact::run(args, root)
        }
        Commands::Similar(args) => {
            let root = project_root_from_context(&ctx)?;
            commands::similar::run(args, root)
        }
        Commands::Summary(args) => {
            let root = project_root_from_context(&ctx)?;
            commands::summary::run(args, root)
        }
        Commands::Source(args) => {
            let root = project_root_from_context(&ctx)?;
            commands::source::run(args, root)
        }
        Commands::Trace(args) => {
            let root = project_root_from_context(&ctx)?;
            commands::trace::run(args, root)
        }
        Commands::Diff(args) => {
            let root = project_root_from_context(&ctx)?;
            commands::diff::run(args, root)
        }
        Commands::Flow(args) => {
            let root = project_root_from_context(&ctx)?;
            commands::flow::run(args, root)
        }

        Commands::Setup(args) => {
            let root = project_root_from_context(&ctx)?;
            commands::setup::run(args, root)
        }

        // --- Workspace management subcommands ---
        Commands::Workspace(args) => {
            let root = cwd()?;
            commands::workspace::run(args, &root)
        }
    }
}

/// Resolve the execution context based on CLI flags.
///
/// - No flags: returns `SingleProject` with CWD as root.
/// - `--workspace`: finds scope-workspace.toml upward from CWD.
/// - `--project <name>`: finds workspace manifest, then targets that member.
fn resolve_context(workspace_flag: bool, project_flag: Option<&str>) -> Result<Context> {
    let cwd = cwd()?;

    if let Some(project_name) = project_flag {
        // --project implies workspace context; find the manifest and resolve member
        let manifest_path = commands::workspace::find_workspace_manifest(&cwd)?;
        let workspace_root = manifest_path.parent().unwrap_or(&cwd).to_path_buf();
        let config = config::workspace::WorkspaceConfig::load(&manifest_path)?;
        config.validate(&workspace_root)?;

        // Find the named member
        let member = config
            .workspace
            .members
            .iter()
            .find(|m| config::workspace::WorkspaceConfig::resolve_member_name(m) == project_name)
            .ok_or_else(|| {
                let available: Vec<String> = config
                    .workspace
                    .members
                    .iter()
                    .map(config::workspace::WorkspaceConfig::resolve_member_name)
                    .collect();
                anyhow::anyhow!(
                    "Project '{}' not found in workspace. Available: {}",
                    project_name,
                    available.join(", ")
                )
            })?;

        let member_root = workspace_root.join(&member.path);
        return Ok(Context::SingleProject { root: member_root });
    }

    if workspace_flag {
        let manifest_path = commands::workspace::find_workspace_manifest(&cwd)?;
        let workspace_root = manifest_path.parent().unwrap_or(&cwd).to_path_buf();
        let config = config::workspace::WorkspaceConfig::load(&manifest_path)?;
        config.validate(&workspace_root)?;

        return Ok(Context::Workspace {
            manifest_path,
            workspace_root,
            config,
        });
    }

    // Default: single project with CWD
    Ok(Context::SingleProject { root: cwd })
}

/// Get the current working directory.
fn cwd() -> Result<PathBuf> {
    std::env::current_dir().map_err(|e| anyhow::anyhow!("Failed to get current directory: {e}"))
}

/// Extract a project root from the context.
///
/// For single-project mode, returns the root directly.
/// For workspace mode, errors with guidance to use `--project`.
fn project_root_from_context(ctx: &Context) -> Result<&Path> {
    match ctx {
        Context::SingleProject { root } => Ok(root),
        Context::Workspace { .. } => {
            bail!(
                "This command operates on a single project. \
                 Use --project <name> to target a workspace member."
            )
        }
    }
}
