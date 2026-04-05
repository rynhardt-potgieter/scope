/// `scope setup` — one-command setup for agent integration.
///
/// Runs `scope init` + `scope index --full`, then writes the CLAUDE.md
/// snippet and optionally bakes `scope map` output into it (--preload).
///
/// Benchmarks show preloading saves 32% on agent cost (Phase 12 data).
///
/// Examples:
///   scope setup              — init + index + write CLAUDE.md snippet
///   scope setup --preload    — same, plus bake scope map into CLAUDE.md
///   scope setup --json       — machine-readable single JSON envelope
use anyhow::{Context, Result};
use clap::Args;
use std::path::Path;

use crate::output::json::JsonOutput;

/// Run a scope subcommand as a subprocess, suppressing its stdout.
/// Used by --json mode to prevent child commands from polluting the
/// JSON output stream. Stderr is inherited so warnings still appear.
fn run_subprocess(project_root: &Path, args: &[&str]) -> Result<()> {
    let scope_bin = std::env::current_exe()?;
    let status = std::process::Command::new(scope_bin)
        .args(args)
        .current_dir(project_root)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::inherit())
        .status()
        .with_context(|| format!("Failed to run scope {}", args[0]))?;
    if !status.success() {
        anyhow::bail!(
            "scope {} failed with exit code {:?}",
            args[0],
            status.code()
        );
    }
    Ok(())
}

/// Arguments for the `scope setup` command.
#[derive(Args, Debug)]
pub struct SetupArgs {
    /// Bake `scope map` output into CLAUDE.md for 32% agent cost savings.
    ///
    /// Phase 12 benchmarks showed preloaded agents save 35% on output
    /// tokens and 32% on cost because the agent already has the repo's
    /// architecture when the conversation starts.
    #[arg(long)]
    pub preload: bool,

    /// Output as JSON instead of human-readable format
    #[arg(long, short = 'j')]
    pub json: bool,
}

/// Build the preload architecture snippet from the index.
fn build_preload_snippet(scope_dir: &Path) -> Result<Option<String>> {
    let db_path = scope_dir.join("graph.db");
    if !db_path.exists() {
        return Ok(None);
    }
    let graph = crate::core::graph::Graph::open(&db_path)?;
    let stats_line = format!(
        "{} files, {} symbols, {} edges",
        graph.file_count()?,
        graph.symbol_count()?,
        graph.edge_count()?,
    );
    let core = graph.get_symbols_by_importance(10)?;
    let core_lines: Vec<String> = core
        .iter()
        .map(|(sym, count)| format!("  {} ({}) — {} callers", sym.name, sym.file_path, count))
        .collect();
    let dirs = graph.get_directory_stats()?;
    let arch_lines: Vec<String> = dirs
        .iter()
        .map(|(dir, files, syms)| format!("  {dir} — {files} files, {syms} symbols"))
        .collect();
    Ok(Some(format!(
        "\n### Preloaded Architecture (scope map)\n\n\
         Stats: {stats_line}\n\n\
         Core symbols:\n{}\n\n\
         Architecture:\n{}\n",
        core_lines.join("\n"),
        arch_lines.join("\n"),
    )))
}

/// Run the `scope setup` command.
pub fn run(args: &SetupArgs, project_root: &Path) -> Result<()> {
    let scope_dir = project_root.join(".scope");

    // Track what actually happened for accurate JSON output.
    let mut did_init = false;
    let mut did_claude_md = false;
    let mut did_skill = false;

    // Step 1: Init (skip if already done).
    if !scope_dir.exists() {
        if args.json {
            // Subprocess with stdout suppressed so JSON stream stays clean.
            run_subprocess(project_root, &["init"])?;
        } else {
            println!("Initialising scope...");
            let init_args = crate::commands::init::InitArgs { json: false };
            crate::commands::init::run(&init_args, project_root)?;
        }
        did_init = true;
    } else if !args.json {
        println!("scope already initialised, skipping init.");
    }

    // Step 2: Full index.
    if args.json {
        run_subprocess(project_root, &["index", "--full"])?;
    } else {
        println!("Building index...");
        let index_args = crate::commands::index::IndexArgs {
            full: true,
            json: false,
            watch: false,
        };
        crate::commands::index::run(&index_args, project_root)?;
    }

    // Step 3: Write CLAUDE.md snippet
    let claude_md_path = project_root.join("CLAUDE.md");
    let snippet_marker = "## Code Navigation";

    let existing = std::fs::read_to_string(&claude_md_path).unwrap_or_default();
    let has_section = existing.contains(snippet_marker);
    let has_preload = existing.contains("### Preloaded Architecture");
    let needs_preload_upgrade = has_section && args.preload && !has_preload;

    if has_section && !needs_preload_upgrade {
        if !args.json {
            println!("CLAUDE.md already has Code Navigation section, skipping.");
        }
    } else if needs_preload_upgrade {
        // Upgrade: section exists but preload is missing. Append preload block.
        if let Some(preload_snippet) = build_preload_snippet(&scope_dir)? {
            let mut content = existing;
            content.push_str(&preload_snippet);
            std::fs::write(&claude_md_path, content)?;
            did_claude_md = true;
            if !args.json {
                println!("Added preloaded architecture to existing CLAUDE.md");
            }
        }
    } else {
        // Fresh install: write full section + optional preload.
        let mut snippet = format!(
            "\n\n{snippet_marker}\n\n\
             This project has [Scope](https://github.com/rynhardt-potgieter/scope) CLI installed.\n\
             Run `scope status` to check availability and `scope map` for a repo overview.\n\n\
             When dispatching subagents that need to navigate, search, or understand code,\n\
             include the `code-navigation` skill or instruct them to read\n\
             `.claude/skills/code-navigation/SKILL.md` before starting.\n"
        );

        if args.preload {
            if let Some(preload_snippet) = build_preload_snippet(&scope_dir)? {
                snippet.push_str(&preload_snippet);
            }
        }

        let mut content = existing;
        content.push_str(&snippet);
        std::fs::write(&claude_md_path, content)?;
        did_claude_md = true;
        if !args.json {
            println!("Appended Code Navigation section to CLAUDE.md");
        }
    }

    // Step 5: Copy skill file
    let skill_dir = project_root.join(".claude/skills/code-navigation");
    if !skill_dir.exists() {
        std::fs::create_dir_all(&skill_dir)?;
        let skill_content = include_str!("../../skills/code-navigation/SKILL.md");
        std::fs::write(skill_dir.join("SKILL.md"), skill_content)?;
        did_skill = true;
        if !args.json {
            println!("Installed code-navigation skill to .claude/skills/");
        }
    } else if !args.json {
        println!("code-navigation skill already installed, skipping.");
    }

    if args.json {
        let data = serde_json::json!({
            "initialized": did_init,
            "indexed": true,
            "preloaded": args.preload,
            "claude_md_updated": did_claude_md,
            "skill_installed": did_skill,
            "scope_dir": ".scope/",
        });
        let envelope = JsonOutput {
            command: "setup",
            symbol: None,
            data: &data,
            truncated: false,
            total: 1,
        };
        println!("{}", serde_json::to_string_pretty(&envelope)?);
    } else if args.preload {
        println!(
            "\nSetup complete with preloading. Benchmark data shows this saves ~32% on agent cost."
        );
    } else {
        println!("\nSetup complete. Run with --preload to bake architecture into CLAUDE.md for 32% agent cost savings.");
    }

    Ok(())
}
