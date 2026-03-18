//! Human-readable output formatting for all Scope commands.
//!
//! Rules:
//! - Separator line uses `─` (U+2500), never `-` or `=`
//! - File paths always use forward slashes, even on Windows
//! - Line ranges formatted as `start-end`
//! - Caller counts in square brackets: `[11 callers]`, `[internal]`
//! - Similarity scores always 2 decimal places: `0.91`

/// The separator line used between header and body in all command output.
#[allow(dead_code)]
pub const SEPARATOR: &str = "─────────────────────────────────────────────────────────────────";

/// Normalize a file path to always use forward slashes in output.
#[allow(dead_code)]
pub fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// Format a line range as `start-end`.
#[allow(dead_code)]
pub fn format_line_range(start: u32, end: u32) -> String {
    if start == end {
        format!("{start}")
    } else {
        format!("{start}-{end}")
    }
}
