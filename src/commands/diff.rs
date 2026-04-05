/// `scope diff` — show which symbols changed since a git ref.
///
/// Runs `git diff --name-only` against the given ref (default: HEAD),
/// then cross-references the changed files with the index to show
/// exactly which symbols were added, modified, or deleted.
///
/// Designed for code review and PR triage: an agent can instantly see
/// what changed structurally without reading full diffs.
///
/// Examples:
///   scope diff                     — changes since last commit
///   scope diff --ref main          — changes vs main branch
///   scope diff --ref HEAD~3 --json — last 3 commits, JSON output
use anyhow::{bail, Result};
use clap::Args;
use serde::Serialize;
use std::path::Path;
use std::process::Command;

use crate::core::graph::Graph;
use crate::output::json::JsonOutput;

/// Arguments for the `scope diff` command.
#[derive(Args, Debug)]
pub struct DiffArgs {
    /// Git ref to compare against (default: HEAD)
    #[arg(long, default_value = "HEAD")]
    pub r#ref: String,

    /// Output as JSON instead of human-readable format
    #[arg(long, short = 'j')]
    pub json: bool,
}

/// A symbol that was affected by the diff.
#[derive(Debug, Serialize)]
pub struct ChangedSymbol {
    pub name: String,
    pub kind: String,
    pub file_path: String,
    pub line_start: u32,
    pub line_end: u32,
    pub signature: Option<String>,
}

/// Full diff output.
#[derive(Debug, Serialize)]
pub struct DiffOutput {
    pub git_ref: String,
    pub changed_files: Vec<String>,
    pub symbols: Vec<ChangedSymbol>,
}

/// Run the `scope diff` command.
pub fn run(args: &DiffArgs, project_root: &Path) -> Result<()> {
    let scope_dir = project_root.join(".scope");
    if !scope_dir.exists() {
        bail!("No .scope/ directory found. Run 'scope init' first.");
    }

    let db_path = scope_dir.join("graph.db");
    if !db_path.exists() {
        bail!("No index found. Run 'scope index' first.");
    }

    // Validate ref doesn't look like a flag (prevents injection into git args).
    if args.r#ref.starts_with('-') {
        bail!("Invalid git ref '{}': must not start with '-'", args.r#ref);
    }

    // Get changed files from git
    let output = Command::new("git")
        .args(["diff", "--name-only", &args.r#ref, "--"])
        .current_dir(project_root)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git diff failed: {}", stderr.trim());
    }

    let changed_files: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect();

    if changed_files.is_empty() {
        if args.json {
            let out = DiffOutput {
                git_ref: args.r#ref.clone(),
                changed_files: vec![],
                symbols: vec![],
            };
            let envelope = JsonOutput {
                command: "diff",
                symbol: None,
                data: &out,
                truncated: false,
                total: 0,
            };
            println!("{}", serde_json::to_string_pretty(&envelope)?);
        } else {
            println!("No changes vs {}", args.r#ref);
        }
        return Ok(());
    }

    // Look up symbols in changed files
    let graph = Graph::open(&db_path)?;
    crate::commands::warn_if_stale(&graph, project_root);
    let mut symbols: Vec<ChangedSymbol> = Vec::new();

    for file in &changed_files {
        let file_syms = graph.get_file_symbols(file)?;
        for s in file_syms {
            symbols.push(ChangedSymbol {
                name: s.name,
                kind: s.kind,
                file_path: s.file_path,
                line_start: s.line_start,
                line_end: s.line_end,
                signature: s.signature,
            });
        }
    }

    if args.json {
        let total = symbols.len();
        let out = DiffOutput {
            git_ref: args.r#ref.clone(),
            changed_files: changed_files.clone(),
            symbols,
        };
        let envelope = JsonOutput {
            command: "diff",
            symbol: None,
            data: &out,
            truncated: false,
            total,
        };
        println!("{}", serde_json::to_string_pretty(&envelope)?);
    } else {
        println!(
            "Changes vs {} — {} files, {} symbols",
            args.r#ref,
            changed_files.len(),
            symbols.len(),
        );
        println!("{}", "─".repeat(72));

        for file in &changed_files {
            let file_syms: Vec<&ChangedSymbol> =
                symbols.iter().filter(|s| &s.file_path == file).collect();

            if file_syms.is_empty() {
                println!("  {file}  (no indexed symbols)");
            } else {
                println!("  {file}");
                for s in file_syms {
                    let sig = s
                        .signature
                        .as_deref()
                        .map(|sig| {
                            // Truncate multi-line signatures to first line
                            sig.lines().next().unwrap_or(sig)
                        })
                        .unwrap_or("");
                    println!("    {} {}  :{}  {}", s.kind, s.name, s.line_start, sig,);
                }
            }
        }
    }

    Ok(())
}
