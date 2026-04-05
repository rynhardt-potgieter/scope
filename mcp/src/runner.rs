//! Subprocess runner for the `scope` CLI binary.
//!
//! Spawns `scope <cmd> --json` and captures the JSON output.
//! The `scope` binary is located via the `SCOPE_BIN` environment variable
//! or by searching `PATH` with `which`.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::{Child, Command};

const TIMEOUT_SECS: u64 = 30;

/// Find the scope binary path.
///
/// Checks `SCOPE_BIN` env var first, then `PATH` via `which`,
/// then common install locations (cross-platform).
pub fn find_scope_bin() -> Result<PathBuf, String> {
    if let Ok(bin) = std::env::var("SCOPE_BIN") {
        let path = PathBuf::from(bin);
        if path.exists() {
            return Ok(path);
        }
    }

    // Try PATH first
    if let Ok(path) = which::which("scope") {
        return Ok(path);
    }

    // Check common install locations (cross-platform)
    if let Some(home) = dirs::home_dir() {
        for suffix in &["bin/scope", ".cargo/bin/scope"] {
            let path = home.join(suffix);
            if path.exists() {
                return Ok(path);
            }
        }
    }

    #[cfg(unix)]
    {
        let path = PathBuf::from("/usr/local/bin/scope");
        if path.exists() {
            return Ok(path);
        }
    }

    Err("Could not find `scope` binary. Install it or set SCOPE_BIN env var.".to_string())
}

/// Run a scope command and return the parsed JSON output.
///
/// Spawns `scope <args>` as a subprocess in the given working directory
/// with a 30-second timeout. Returns the `data` field from the standard
/// `JsonOutput` envelope, or the full JSON if no envelope is detected.
///
/// On failure (non-zero exit or timeout), returns the stderr content
/// as an error string.
pub async fn run_scope(
    scope_bin: &Path,
    args: &[&str],
    cwd: &Path,
) -> Result<serde_json::Value, String> {
    let child: Child = Command::new(scope_bin)
        .args(args)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn scope: {e}"))?;

    // wait_with_output takes ownership, so wrap in an Option for the timeout branch.
    let mut child_opt = Some(child);
    let output = tokio::select! {
        result = async { child_opt.take().unwrap().wait_with_output().await } => {
            result.map_err(|e| format!("Failed to wait for scope: {e}"))?
        }
        _ = tokio::time::sleep(Duration::from_secs(TIMEOUT_SECS)) => {
            if let Some(mut c) = child_opt.take() {
                c.kill().await.ok();
                c.wait().await.ok(); // reap to avoid zombie
            }
            return Err(format!("scope command timed out after {TIMEOUT_SECS}s"));
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let msg = stderr.trim();
        let clean = msg.strip_prefix("Error: ").unwrap_or(msg);
        return Err(clean.to_string());
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| format!("Invalid UTF-8 in scope output: {e}"))?;
    let json: serde_json::Value =
        serde_json::from_str(&stdout).map_err(|e| format!("Invalid JSON from scope: {e}"))?;

    // Strip the JsonOutput envelope — return just the `data` field.
    // If the output doesn't have a `data` field, return it as-is.
    if let Some(data) = json.get("data") {
        Ok(data.clone())
    } else {
        Ok(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Serialize tests that mutate SCOPE_BIN env var to avoid races.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_find_scope_bin_from_env() {
        let _guard = ENV_LOCK.lock().unwrap();
        let which_path = which::which("which")
            .or_else(|_| which::which("ls"))
            .expect("need some binary on PATH for this test");
        std::env::set_var("SCOPE_BIN", which_path.to_str().unwrap());
        let result = find_scope_bin();
        std::env::remove_var("SCOPE_BIN");
        assert!(result.is_ok());
    }

    #[test]
    fn test_find_scope_bin_nonexistent_env() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("SCOPE_BIN", "/nonexistent/path/scope");
        let result = find_scope_bin();
        std::env::remove_var("SCOPE_BIN");
        if let Ok(path) = &result {
            assert_ne!(path.to_str().unwrap(), "/nonexistent/path/scope");
        }
    }
}
