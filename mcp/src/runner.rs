//! Subprocess runner for the `scope` CLI binary.
//!
//! Spawns `scope <cmd> --json` and captures the JSON output.
//! The `scope` binary is located via the `SCOPE_BIN` environment variable
//! or by searching `PATH` with `which`.

use std::path::{Path, PathBuf};
use tokio::process::Command;

/// Find the scope binary path.
///
/// Checks `SCOPE_BIN` env var first, then falls back to `which scope`.
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

    // Check common install locations
    let home = std::env::var("HOME").unwrap_or_default();
    for candidate in &[
        format!("{home}/bin/scope"),
        format!("{home}/.cargo/bin/scope"),
        "/usr/local/bin/scope".to_string(),
    ] {
        let path = PathBuf::from(candidate);
        if path.exists() {
            return Ok(path);
        }
    }

    Err("Could not find `scope` binary. Install it or set SCOPE_BIN env var.".to_string())
}

/// Run a scope command and return the parsed JSON output.
///
/// Spawns `scope <args> --json` as a subprocess in the given working directory.
/// Returns the `data` field from the standard `JsonOutput` envelope, or the
/// full JSON if no envelope is detected.
///
/// On failure (non-zero exit), returns the stderr content as an error string.
pub async fn run_scope(
    scope_bin: &Path,
    args: &[&str],
    cwd: &Path,
) -> Result<serde_json::Value, String> {
    let output = Command::new(scope_bin)
        .args(args)
        .current_dir(cwd)
        .output()
        .await
        .map_err(|e| format!("Failed to spawn scope: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let msg = stderr.trim();
        // Strip the "Error: " prefix that anyhow adds
        let clean = msg.strip_prefix("Error: ").unwrap_or(msg);
        return Err(clean.to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
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
