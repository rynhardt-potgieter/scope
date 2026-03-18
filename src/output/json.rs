//! JSON output envelope for all Scope commands.
//!
//! Every `--json` output uses `JsonOutput<T>` as the wrapper, ensuring
//! a consistent schema across all commands.
use serde::Serialize;

/// The standard JSON envelope for all command output.
///
/// Example:
/// ```json
/// {
///   "command": "refs",
///   "symbol": "processPayment",
///   "data": [...],
///   "truncated": false,
///   "total": 11
/// }
/// ```
#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct JsonOutput<T: Serialize> {
    /// The command that produced this output (e.g. "sketch", "refs").
    pub command: &'static str,
    /// The symbol name that was queried, if applicable.
    pub symbol: Option<String>,
    /// The command-specific data payload.
    pub data: T,
    /// Whether the output was truncated due to a limit.
    pub truncated: bool,
    /// The total count of results before truncation.
    pub total: usize,
}
