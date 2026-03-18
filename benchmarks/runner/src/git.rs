use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Reset a corpus directory to its clean state.
///
/// Runs `git reset --hard HEAD` and `git clean -fd` to undo all changes
/// the agent may have made during a benchmark run.
pub fn reset_corpus(corpus_path: &Path) -> Result<()> {
    let status = Command::new("git")
        .args(["reset", "--hard", "HEAD"])
        .current_dir(corpus_path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .with_context(|| format!("Failed to run git reset in {}", corpus_path.display()))?;

    if !status.success() {
        anyhow::bail!("git reset --hard HEAD failed in {}", corpus_path.display());
    }

    let status = Command::new("git")
        .args(["clean", "-fd"])
        .current_dir(corpus_path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .with_context(|| format!("Failed to run git clean in {}", corpus_path.display()))?;

    if !status.success() {
        anyhow::bail!("git clean -fd failed in {}", corpus_path.display());
    }

    Ok(())
}

/// Back up the `.scope/` index directory before a benchmark run.
///
/// Copies the pre-built Scope index to a temporary location so it can be
/// restored between runs without re-indexing.
///
/// Returns the path to the backup directory.
pub fn backup_scope_index(corpus_path: &Path) -> Result<PathBuf> {
    let scope_dir = corpus_path.join(".scope");
    if !scope_dir.is_dir() {
        anyhow::bail!(
            "No .scope/ directory found in {}. Build the index first with 'scope index'.",
            corpus_path.display()
        );
    }

    let backup_dir =
        std::env::temp_dir().join(format!("scope-benchmark-backup-{}", std::process::id()));

    if backup_dir.exists() {
        std::fs::remove_dir_all(&backup_dir)
            .with_context(|| format!("Failed to clean old backup at {}", backup_dir.display()))?;
    }

    copy_dir_recursive(&scope_dir, &backup_dir)
        .with_context(|| format!("Failed to back up .scope/ to {}", backup_dir.display()))?;

    Ok(backup_dir)
}

/// Restore the `.scope/` index directory from a backup.
///
/// Removes the current `.scope/` directory (which may have been modified
/// by the agent or by git clean) and replaces it with the backup.
pub fn restore_scope_index(corpus_path: &Path, backup: &Path) -> Result<()> {
    let scope_dir = corpus_path.join(".scope");

    // Remove current .scope/ if it exists
    if scope_dir.exists() {
        std::fs::remove_dir_all(&scope_dir)
            .with_context(|| format!("Failed to remove .scope/ at {}", scope_dir.display()))?;
    }

    copy_dir_recursive(backup, &scope_dir)
        .with_context(|| format!("Failed to restore .scope/ from {}", backup.display()))?;

    Ok(())
}

/// Recursively copy a directory and all its contents.
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;

    for entry in walkdir::WalkDir::new(src)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let relative = entry
            .path()
            .strip_prefix(src)
            .context("Failed to compute relative path during copy")?;
        let target = dst.join(relative);

        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(entry.path(), &target)?;
        }
    }

    Ok(())
}
