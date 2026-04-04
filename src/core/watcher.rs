//! File system watching for incremental re-indexing.
//!
//! Uses the `notify` crate with `notify-debouncer-mini` to watch for
//! file changes and emit batched events for re-indexing.

use anyhow::{bail, Context, Result};
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

/// Watches the file system for changes to supported source files.
///
/// Respects `.gitignore` and config ignore patterns. Filters events
/// to only supported file extensions. Debounces rapid changes into
/// batched events.
pub struct Watcher {
    /// Project root directory being watched.
    project_root: PathBuf,
    /// Glob patterns to ignore (from config + .gitignore).
    ignore_patterns: Vec<String>,
    /// Supported file extensions (without the dot, e.g. "ts", "tsx", "cs").
    supported_extensions: Vec<String>,
    /// Debounce duration for file events.
    debounce_duration: Duration,
}

impl Watcher {
    /// Create a new file watcher.
    ///
    /// # Arguments
    /// - `project_root` — the root directory to watch recursively
    /// - `ignore_patterns` — glob patterns to exclude (e.g. `node_modules`, `dist`)
    /// - `supported_extensions` — file extensions to include (without dot)
    /// - `debounce_duration` — how long to wait before emitting a batch
    pub fn new(
        project_root: PathBuf,
        ignore_patterns: Vec<String>,
        supported_extensions: Vec<String>,
        debounce_duration: Duration,
    ) -> Self {
        Self {
            project_root,
            ignore_patterns,
            supported_extensions,
            debounce_duration,
        }
    }

    /// Start watching and return a receiver that emits batched file change paths.
    ///
    /// The returned receiver will emit `Vec<PathBuf>` batches whenever files
    /// matching the supported extensions change. The watcher runs on a
    /// background thread managed by the `notify` crate.
    ///
    /// Returns the receiver and the debouncer handle (must be kept alive).
    pub fn start(
        &self,
    ) -> Result<(
        mpsc::Receiver<Vec<PathBuf>>,
        notify_debouncer_mini::Debouncer<notify::RecommendedWatcher>,
    )> {
        let (tx, rx) = mpsc::channel();

        let project_root = self.project_root.clone();
        let ignore_patterns = self.ignore_patterns.clone();
        let supported_extensions = self.supported_extensions.clone();

        let debouncer_tx = tx;
        let mut debouncer = new_debouncer(
            self.debounce_duration,
            move |result: notify_debouncer_mini::DebounceEventResult| match result {
                Ok(events) => {
                    let paths: Vec<PathBuf> = events
                        .into_iter()
                        .filter(|e| e.kind == DebouncedEventKind::Any)
                        .map(|e| e.path)
                        .filter(|p| {
                            is_supported_file(p, &supported_extensions)
                                && !is_ignored(p, &project_root, &ignore_patterns)
                        })
                        .collect::<HashSet<_>>()
                        .into_iter()
                        .collect();

                    if !paths.is_empty() {
                        if let Err(e) = debouncer_tx.send(paths) {
                            tracing::warn!("Failed to send watch event: {e}");
                        }
                    }
                }
                Err(error) => {
                    tracing::warn!("File watcher error: {error}");
                }
            },
        )
        .context("Failed to create file watcher")?;

        // Watch the project root recursively
        debouncer
            .watcher()
            .watch(&self.project_root, notify::RecursiveMode::Recursive)
            .with_context(|| {
                format!("Failed to watch directory: {}", self.project_root.display())
            })?;

        Ok((rx, debouncer))
    }
}

/// Check if a file has a supported extension.
pub fn is_supported_file(path: &Path, supported_extensions: &[String]) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| supported_extensions.iter().any(|s| s == ext))
        .unwrap_or(false)
}

/// Check if a file path should be ignored based on ignore patterns.
///
/// Uses simple directory-name matching: if any path component matches
/// an ignore pattern, the file is ignored. Also ignores the `.scope/` directory.
pub fn is_ignored(path: &Path, project_root: &Path, ignore_patterns: &[String]) -> bool {
    let rel_path = match path.strip_prefix(project_root) {
        Ok(p) => p,
        Err(_) => return false,
    };

    // Always ignore the .scope directory
    if rel_path.starts_with(".scope") {
        return true;
    }

    // Check each path component against ignore patterns
    for component in rel_path.components() {
        let component_str = component.as_os_str().to_string_lossy();
        for pattern in ignore_patterns {
            if component_str == *pattern {
                return true;
            }
        }
    }

    false
}

/// Lock file management for preventing concurrent watchers.
pub struct WatchLock {
    lock_path: PathBuf,
}

impl WatchLock {
    /// Create a new lock file manager for the given `.scope/` directory.
    pub fn new(scope_dir: &Path) -> Self {
        Self {
            lock_path: scope_dir.join(".watch.lock"),
        }
    }

    /// Acquire the watch lock. Fails if another watcher is running.
    pub fn acquire(&self) -> Result<()> {
        if self.lock_path.exists() {
            let content = match std::fs::read_to_string(&self.lock_path) {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!("Cannot read lock file: {e}. Treating as stale.");
                    String::new()
                }
            };
            let pid_str = content.trim();

            if !pid_str.is_empty() {
                match pid_str.parse::<u32>() {
                    Ok(pid) => {
                        if is_process_alive(pid) {
                            bail!(
                                "Another watcher is running (PID {pid}). \
                                 Stop it first or remove .scope/.watch.lock"
                            );
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Lock file contains invalid PID '{}': {e}. Treating as stale.",
                            pid_str
                        );
                    }
                }
            }
            // Stale lock file — remove it
            tracing::warn!("Removing stale watch lock file");
        }

        let current_pid = std::process::id();
        std::fs::write(&self.lock_path, current_pid.to_string())
            .with_context(|| format!("Failed to write lock file: {}", self.lock_path.display()))?;

        Ok(())
    }

    /// Release the watch lock by removing the lock file.
    pub fn release(&self) {
        if self.lock_path.exists() {
            if let Err(e) = std::fs::remove_file(&self.lock_path) {
                eprintln!("Warning: failed to remove watch lock file: {e}. You may need to delete .scope/.watch.lock manually.");
            }
        }
    }

    /// Get the path to the lock file (for testing).
    #[allow(dead_code)]
    pub fn lock_path(&self) -> &Path {
        &self.lock_path
    }
}

impl Drop for WatchLock {
    fn drop(&mut self) {
        self.release();
    }
}

/// Check if a process with the given PID is still alive.
#[cfg(windows)]
fn is_process_alive(pid: u32) -> bool {
    use std::process::Command;
    // On Windows, use tasklist to check if the PID exists
    Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}"), "/NH"])
        .output()
        .map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout);
            // tasklist returns the process info if it exists, or "INFO: No tasks..."
            !stdout.contains("No tasks") && stdout.contains(&pid.to_string())
        })
        .unwrap_or(false)
}

/// Check if a process with the given PID is still alive.
#[cfg(not(windows))]
fn is_process_alive(pid: u32) -> bool {
    // On Unix, check if /proc/<pid> exists (Linux) or use kill -0 via command
    use std::process::Command;
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn is_supported_file_matches_known_extensions() {
        let exts = vec!["ts".to_string(), "tsx".to_string(), "cs".to_string()];
        assert!(is_supported_file(Path::new("src/app.ts"), &exts));
        assert!(is_supported_file(Path::new("src/app.tsx"), &exts));
        assert!(is_supported_file(Path::new("src/Service.cs"), &exts));
        assert!(!is_supported_file(Path::new("src/app.js"), &exts));
        assert!(!is_supported_file(Path::new("README.md"), &exts));
        assert!(!is_supported_file(Path::new("Makefile"), &exts));
    }

    #[test]
    fn is_ignored_filters_scope_directory() {
        let root = Path::new("/project");
        let patterns: Vec<String> = vec![];
        assert!(is_ignored(
            Path::new("/project/.scope/graph.db"),
            root,
            &patterns
        ));
        assert!(is_ignored(
            Path::new("/project/.scope/config.toml"),
            root,
            &patterns
        ));
    }

    #[test]
    fn is_ignored_filters_configured_patterns() {
        let root = Path::new("/project");
        let patterns = vec!["node_modules".to_string(), "dist".to_string()];
        assert!(is_ignored(
            Path::new("/project/node_modules/pkg/index.ts"),
            root,
            &patterns
        ));
        assert!(is_ignored(
            Path::new("/project/dist/bundle.js"),
            root,
            &patterns
        ));
        assert!(!is_ignored(
            Path::new("/project/src/app.ts"),
            root,
            &patterns
        ));
    }

    #[test]
    fn watch_lock_acquire_and_release() {
        let dir = tempdir().unwrap();
        let lock = WatchLock::new(dir.path());

        // Acquire should succeed
        lock.acquire().unwrap();
        assert!(lock.lock_path().exists());

        // Lock file should contain current PID
        let content = std::fs::read_to_string(lock.lock_path()).unwrap();
        let pid: u32 = content.trim().parse().unwrap();
        assert_eq!(pid, std::process::id());

        // Release should remove the file
        lock.release();
        assert!(!lock.lock_path().exists());
    }

    #[test]
    fn watch_lock_detects_stale_lock() {
        let dir = tempdir().unwrap();
        let lock = WatchLock::new(dir.path());

        // Write a lock file with a bogus PID (very unlikely to be alive)
        std::fs::write(lock.lock_path(), "999999999").unwrap();

        // Acquire should succeed (stale lock)
        lock.acquire().unwrap();

        // Lock file should now contain our PID
        let content = std::fs::read_to_string(lock.lock_path()).unwrap();
        let pid: u32 = content.trim().parse().unwrap();
        assert_eq!(pid, std::process::id());
    }

    #[test]
    fn watch_lock_blocks_concurrent_watcher() {
        let dir = tempdir().unwrap();

        // Write a lock file with our own PID (definitely alive)
        let lock_path = dir.path().join(".watch.lock");
        std::fs::write(&lock_path, std::process::id().to_string()).unwrap();

        let lock = WatchLock::new(dir.path());
        let result = lock.acquire();
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Another watcher is running"));
    }
}
