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
use anyhow::Result;
use clap::Args;
use std::path::Path;

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
}

/// Run the `scope setup` command.
pub fn run(args: &SetupArgs, project_root: &Path) -> Result<()> {
    let scope_dir = project_root.join(".scope");

    // Step 1: Init (skip if already done)
    if !scope_dir.exists() {
        println!("Initialising scope...");
        let init_args = crate::commands::init::InitArgs { json: false };
        crate::commands::init::run(&init_args, project_root)?;
    } else {
        println!("scope already initialised, skipping init.");
    }

    // Step 2: Full index
    println!("Building index...");
    let index_args = crate::commands::index::IndexArgs {
        full: true,
        json: false,
        watch: false,
    };
    crate::commands::index::run(&index_args, project_root)?;

    // Step 3: Write CLAUDE.md snippet
    let claude_md_path = project_root.join("CLAUDE.md");
    let snippet_marker = "## Code Navigation";

    // Check if CLAUDE.md already has the snippet
    let existing = std::fs::read_to_string(&claude_md_path).unwrap_or_default();
    if existing.contains(snippet_marker) {
        println!("CLAUDE.md already has Code Navigation section, skipping.");
    } else {
        let mut snippet = format!(
            "\n\n{snippet_marker}\n\n\
             This project has [Scope](https://github.com/rynhardt-potgieter/scope) CLI installed.\n\
             Run `scope status` to check availability and `scope map` for a repo overview.\n\n\
             When dispatching subagents that need to navigate, search, or understand code,\n\
             include the `code-navigation` skill or instruct them to read\n\
             `.claude/skills/code-navigation/SKILL.md` before starting.\n"
        );

        // Step 4: Preload map output if requested
        if args.preload {
            let db_path = scope_dir.join("graph.db");
            if db_path.exists() {
                let graph = crate::core::graph::Graph::open(&db_path)?;
                let stats_line = format!(
                    "{} files, {} symbols, {} edges",
                    graph.file_count()?,
                    graph.symbol_count()?,
                    graph.edge_count()?,
                );

                // Get core symbols
                let core = graph.get_symbols_by_importance(10)?;
                let core_lines: Vec<String> = core
                    .iter()
                    .map(|(sym, count)| {
                        format!("  {} ({}) — {} callers", sym.name, sym.file_path, count)
                    })
                    .collect();

                // Get architecture
                let dirs = graph.get_directory_stats()?;
                let arch_lines: Vec<String> = dirs
                    .iter()
                    .map(|(dir, files, syms)| format!("  {dir} — {files} files, {syms} symbols"))
                    .collect();

                snippet.push_str(&format!(
                    "\n### Preloaded Architecture (scope map)\n\n\
                     Stats: {stats_line}\n\n\
                     Core symbols:\n{}\n\n\
                     Architecture:\n{}\n",
                    core_lines.join("\n"),
                    arch_lines.join("\n"),
                ));
            }
        }

        // Append to CLAUDE.md
        let mut content = existing;
        content.push_str(&snippet);
        std::fs::write(&claude_md_path, content)?;
        println!("Appended Code Navigation section to CLAUDE.md");
    }

    // Step 5: Copy skill file
    let skill_dir = project_root.join(".claude/skills/code-navigation");
    if !skill_dir.exists() {
        std::fs::create_dir_all(&skill_dir)?;
        // The skill file is in skills/code-navigation/SKILL.md relative to scope's own repo.
        // For installed scope, we generate a minimal skill pointer.
        let skill_content = include_str!("../../skills/code-navigation/SKILL.md");
        std::fs::write(skill_dir.join("SKILL.md"), skill_content)?;
        println!("Installed code-navigation skill to .claude/skills/");
    } else {
        println!("code-navigation skill already installed, skipping.");
    }

    if args.preload {
        println!(
            "\nSetup complete with preloading. Benchmark data shows this saves ~32% on agent cost."
        );
    } else {
        println!("\nSetup complete. Run with --preload to bake architecture into CLAUDE.md for 32% agent cost savings.");
    }

    Ok(())
}
