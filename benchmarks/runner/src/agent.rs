use anyhow::Result;
use serde::Serialize;

use crate::task::TaskDef;

/// Captured data from a single agent run against a task.
#[derive(Debug, Serialize)]
pub struct AgentRun {
    /// The task ID this run was for
    pub task_id: String,
    /// Whether Scope was enabled for this run
    pub scope_enabled: bool,
    /// Total input tokens consumed by the agent
    pub input_tokens: u64,
    /// Total output tokens produced by the agent
    pub output_tokens: u64,
    /// Number of full source file reads during the task
    pub file_reads: u32,
    /// Which scope commands the agent called (e.g. ["scope refs", "scope sketch"])
    pub scope_commands_called: Vec<String>,
    /// Wall clock duration of the agent run in milliseconds
    pub duration_ms: u64,
    /// Exit code of the agent process
    pub exit_code: i32,
}

/// Run a coding agent against a task.
///
/// This is a stub implementation. In production, this will:
/// 1. Write the appropriate CLAUDE.md (with or without Scope)
/// 2. Invoke Claude Code with the task prompt
/// 3. Parse token usage from Claude Code's JSON output
/// 4. Count file reads and sc commands from the tool call log
///
/// Requires `ANTHROPIC_API_KEY` to be set and the `claude` CLI to be installed.
pub fn run_agent(task: &TaskDef, scope_enabled: bool) -> Result<AgentRun> {
    // TODO: Implement actual Claude Code invocation
    //
    // Production implementation will:
    //   1. Copy CLAUDE.md.with-scope or CLAUDE.md.without-scope to CLAUDE.md
    //   2. Run: claude --print "<prompt>" --output-format json
    //   3. Parse the JSON output for token counts
    //   4. Extract file reads and sc commands from tool calls
    //
    // For now, bail with a clear message about what's needed.
    let _ = (task, scope_enabled);
    anyhow::bail!(
        "Agent invocation not yet implemented. \
         Configure ANTHROPIC_API_KEY and install the claude CLI to run benchmarks."
    );
}
