/// `scope init` — initialise Scope for a project.
///
/// Creates the `.scope/` directory with a default `config.toml` and `.gitignore`.
/// Auto-detects languages from project markers (tsconfig.json, .csproj, etc.).
use anyhow::{bail, Result};
use clap::Args;
use std::path::Path;

use crate::config::ProjectConfig;

/// Arguments for the `scope init` command.
#[derive(Args, Debug)]
pub struct InitArgs {
    /// Output as JSON instead of human-readable format
    #[arg(long, short = 'j')]
    pub json: bool,
}

/// Run the `scope init` command.
pub fn run(args: &InitArgs, project_root: &Path) -> Result<()> {
    let scope_dir = project_root.join(".scope");

    // Check if already initialised
    if scope_dir.exists() {
        bail!(
            ".scope/ already exists in {}. Remove it first to re-initialise.",
            project_root.display()
        );
    }

    // Create .scope/ directory
    std::fs::create_dir_all(&scope_dir)?;

    // Detect languages
    let mut languages = Vec::new();

    if project_root.join("tsconfig.json").exists() || project_root.join("package.json").exists() {
        languages.push("typescript".to_string());
    }

    // Check for C# project files
    let has_csharp = std::fs::read_dir(project_root)?
        .filter_map(|e| e.ok())
        .any(|e| {
            let name = e.file_name();
            let name = name.to_string_lossy();
            name.ends_with(".csproj") || name.ends_with(".sln")
        });
    if has_csharp {
        languages.push("csharp".to_string());
    }

    // Check for Python project files
    if project_root.join("pyproject.toml").exists()
        || project_root.join("setup.py").exists()
        || project_root.join("requirements.txt").exists()
        || project_root.join("Pipfile").exists()
    {
        languages.push("python".to_string());
    }

    // Go detection
    if project_root.join("go.mod").exists() {
        languages.push("go".to_string());
    }

    // Rust detection
    if project_root.join("Cargo.toml").exists() {
        languages.push("rust".to_string());
    }

    // Derive project name from directory name
    let project_name = project_root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Write config.toml
    let config = ProjectConfig::default_for(&project_name, languages.clone());
    config.save(&scope_dir)?;

    // Write .gitignore inside .scope/
    std::fs::write(
        scope_dir.join(".gitignore"),
        "# Ignore all Scope index files\ngraph.db\nvectors/\nfile_hashes.db\nmodels/\n",
    )?;

    // Format language display names
    let lang_display: Vec<&str> = languages
        .iter()
        .map(|l| match l.as_str() {
            "typescript" => "TypeScript",
            "csharp" => "C#",
            "python" => "Python",
            "go" => "Go",
            "java" => "Java",
            "rust" => "Rust",
            other => other,
        })
        .collect();

    if args.json {
        let output = serde_json::json!({
            "command": "init",
            "project_name": project_name,
            "languages": languages,
            "scope_dir": ".scope/"
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("Initialised .scope/ for project: {project_name}");
        if lang_display.is_empty() {
            println!("Detected languages: (none)");
        } else {
            println!("Detected languages: {}", lang_display.join(", "));
        }
        println!("Run `scope index` to build the index.");
    }

    Ok(())
}
