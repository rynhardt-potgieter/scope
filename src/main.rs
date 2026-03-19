//! Scope — Code intelligence CLI for LLM coding agents.
//!
//! Builds a local code intelligence index and lets you query it efficiently.
//! Use it before editing any non-trivial code to understand structure,
//! dependencies, and blast radius.
#![allow(dead_code)]

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;
mod config;
mod core;
mod error;
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

    /// Show all callers of a function or method (shorthand for refs --kind calls).
    ///
    /// Equivalent to `scope refs <symbol> --kind calls`. Use when you want
    /// a quick list of everything that calls a given function.
    ///
    /// Examples:
    ///   scope callers processPayment           — who calls this function
    ///   scope callers processPayment --context 2 — with surrounding code
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

    /// Fetch full source of a specific symbol.
    ///
    /// Returns the exact source code of the symbol. Only call this when
    /// ready to read or edit the implementation — use `scope sketch` first.
    ///
    /// Examples:
    ///   scope source processPayment
    ///   scope source PaymentService.validateCard
    Source(commands::source::SourceArgs),

    /// Show index status and freshness.
    ///
    /// Quick health check: is the index built? How many symbols and files?
    /// Are there stale or unindexed files?
    Status(commands::status::StatusArgs),
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

    // Resolve project root (current directory)
    let project_root = resolve_project_root()?;

    match &cli.command {
        Commands::Init(args) => commands::init::run(args, &project_root),
        Commands::Index(args) => commands::index::run(args, &project_root),
        Commands::Sketch(args) => commands::sketch::run(args, &project_root),
        Commands::Refs(args) => commands::refs::run(args, &project_root),
        Commands::Callers(args) => commands::refs::run_callers(args, &project_root),
        Commands::Deps(args) => commands::deps::run(args, &project_root),
        Commands::Rdeps(args) => commands::rdeps::run(args),
        Commands::Impact(args) => commands::impact::run(args, &project_root),
        Commands::Find(args) => commands::find::run(args, &project_root),
        Commands::Similar(args) => commands::similar::run(args),
        Commands::Source(args) => commands::source::run(args),
        Commands::Status(args) => commands::status::run(args, &project_root),
    }
}

/// Resolve the project root directory.
///
/// Uses the current working directory. In the future, this could walk up
/// to find a `.scope/` directory.
fn resolve_project_root() -> Result<PathBuf> {
    std::env::current_dir().map_err(|e| anyhow::anyhow!("Failed to get current directory: {e}"))
}
