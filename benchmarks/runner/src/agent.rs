use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::BufRead;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::task::TaskDef;

/// A single tool call made by the agent during a benchmark run.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentAction {
    /// Sequential position of this action in the run (1-based)
    pub sequence: u32,
    /// The tool name (e.g. "Read", "Edit", "Bash")
    pub tool_name: String,
    /// Short summary of the tool arguments
    pub arguments_summary: String,
    /// Whether this is a navigation/exploration action (Read, Grep, Glob, Bash)
    pub is_navigation: bool,
    /// Whether this is a Bash call that invokes a scope command
    pub is_scope_command: bool,
    /// Whether this is a file-modifying action (Edit, Write)
    pub is_edit: bool,
}

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
    /// Cache creation input tokens (tokens computed fresh for caching)
    pub cache_creation_input_tokens: u64,
    /// Cache read input tokens (tokens read from prompt cache)
    pub cache_read_input_tokens: u64,
    /// Number of full source file reads during the task
    pub file_reads: u32,
    /// Number of file edit/write operations
    pub file_edits: u32,
    /// Number of Grep tool calls
    pub grep_calls: u32,
    /// Number of Glob tool calls
    pub glob_calls: u32,
    /// Number of Bash tool calls
    pub bash_calls: u32,
    /// Which scope commands the agent called (e.g. ["scope sketch", "scope refs"])
    pub scope_commands_called: Vec<String>,
    /// Wall clock duration of the agent run in milliseconds
    pub duration_ms: u64,
    /// Exit code of the agent process
    pub exit_code: i32,
    /// Ordered list of all tool actions taken by the agent
    pub actions: Vec<AgentAction>,
}

/// Parsed data from a saved NDJSON stream.
#[derive(Debug)]
#[allow(dead_code)]
pub struct NdjsonParseResult {
    /// Ordered list of agent tool actions
    pub actions: Vec<AgentAction>,
    /// Total input tokens from usage events
    pub input_tokens: u64,
    /// Total output tokens from usage events
    pub output_tokens: u64,
    /// Cache creation input tokens
    pub cache_creation_input_tokens: u64,
    /// Cache read input tokens
    pub cache_read_input_tokens: u64,
    /// Scope commands detected in Bash calls
    pub scope_commands_called: Vec<String>,
    /// Count of file Read operations
    pub file_reads: u32,
}

/// Parse a saved NDJSON stream into structured action data.
///
/// Reuses the same extraction logic as `run_agent()` but operates on saved
/// text rather than a live process stream. Use this when importing manually
/// captured benchmark results that include raw Claude CLI output.
pub fn parse_ndjson_actions(ndjson_text: &str) -> NdjsonParseResult {
    let mut input_tokens: u64 = 0;
    let mut output_tokens: u64 = 0;
    let mut cache_creation_input_tokens: u64 = 0;
    let mut cache_read_input_tokens: u64 = 0;
    let mut file_reads: u32 = 0;
    let mut scope_commands_called: Vec<String> = Vec::new();
    let mut actions: Vec<AgentAction> = Vec::new();
    let mut sequence: u32 = 0;
    let mut skipped_lines: u32 = 0;

    for line in ndjson_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let value: serde_json::Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => {
                skipped_lines += 1;
                continue;
            }
        };

        // Accumulate usage
        if let Some(usage) = value.get("usage") {
            input_tokens += usage
                .get("input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            output_tokens += usage
                .get("output_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            cache_creation_input_tokens += usage
                .get("cache_creation_input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            cache_read_input_tokens += usage
                .get("cache_read_input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
        }

        // Check nested result.usage
        if value.get("type").and_then(|t| t.as_str()) == Some("result") {
            if let Some(result) = value.get("result") {
                if let Some(usage) = result.get("usage") {
                    input_tokens += usage
                        .get("input_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    output_tokens += usage
                        .get("output_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    cache_creation_input_tokens += usage
                        .get("cache_creation_input_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    cache_read_input_tokens += usage
                        .get("cache_read_input_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                }
            }
        }

        // Extract tool_use events
        let content_items = extract_tool_use_items(&value);

        for item in content_items {
            let tool_name = match item.get("name").and_then(|n| n.as_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };

            let input_obj = item.get("input");
            let arguments_summary = extract_argument_summary(&tool_name, input_obj);

            let is_navigation = is_navigation_tool(&tool_name);
            let is_edit = is_edit_tool(&tool_name);
            let mut is_scope_cmd = false;

            match tool_name.as_str() {
                "Read" => file_reads += 1,
                "Bash" => {
                    if let Some(cmd_str) = input_obj
                        .and_then(|i| i.get("command"))
                        .and_then(|c| c.as_str())
                    {
                        if let Some(scope_cmd) = extract_scope_command(cmd_str) {
                            is_scope_cmd = true;
                            if !scope_commands_called.contains(&scope_cmd) {
                                scope_commands_called.push(scope_cmd);
                            }
                        }
                    }
                }
                _ => {}
            }

            sequence += 1;
            actions.push(AgentAction {
                sequence,
                tool_name,
                arguments_summary,
                is_navigation,
                is_scope_command: is_scope_cmd,
                is_edit,
            });
        }
    }

    if skipped_lines > 0 {
        eprintln!("  [ndjson] Skipped {} non-JSON lines", skipped_lines);
    }

    NdjsonParseResult {
        actions,
        input_tokens,
        output_tokens,
        cache_creation_input_tokens,
        cache_read_input_tokens,
        scope_commands_called,
        file_reads,
    }
}

/// Set up an isolated temporary directory for a benchmark run.
///
/// Copies the entire fixture corpus to a temp directory, installs the
/// appropriate `CLAUDE.md` variant, and optionally restores the `.scope/`
/// index for scope-enabled runs.
fn setup_temp_corpus(
    corpus_path: &Path,
    scope_enabled: bool,
    condition: &str,
    scope_backup: Option<&Path>,
) -> Result<tempfile::TempDir> {
    let temp_dir = tempfile::tempdir().context("Failed to create temporary directory")?;
    let dest = temp_dir.path();

    // Copy the entire fixture directory into the temp dir
    copy_dir_recursive(corpus_path, dest)
        .with_context(|| format!("Failed to copy corpus from {}", corpus_path.display()))?;

    // Select and install the right CLAUDE.md variant
    let variant_name = if scope_enabled {
        "CLAUDE.md.with-scope"
    } else {
        "CLAUDE.md.without-scope"
    };
    let variant_src = dest.join(variant_name);
    let claude_md_dest = dest.join("CLAUDE.md");

    if variant_src.is_file() {
        std::fs::copy(&variant_src, &claude_md_dest)
            .with_context(|| format!("Failed to copy {} to CLAUDE.md", variant_src.display()))?;
    } else {
        // Also check the original corpus path in case the variant wasn't copied
        let variant_original = corpus_path.join(variant_name);
        if variant_original.is_file() {
            std::fs::copy(&variant_original, &claude_md_dest).with_context(|| {
                format!("Failed to copy {} to CLAUDE.md", variant_original.display())
            })?;
        }
    }

    if scope_enabled {
        // Restore the .scope/ directory from backup if provided
        if let Some(backup) = scope_backup {
            let scope_dest = dest.join(".scope");
            // Remove any existing .scope/ that was copied from the corpus
            if scope_dest.exists() {
                std::fs::remove_dir_all(&scope_dest)
                    .context("Failed to remove existing .scope/ in temp dir")?;
            }
            copy_dir_recursive(backup, &scope_dest)
                .context("Failed to restore .scope/ backup into temp dir")?;
        }
    } else {
        // Ensure no .scope/ directory exists for no-scope runs
        let scope_dir = dest.join(".scope");
        if scope_dir.exists() {
            std::fs::remove_dir_all(&scope_dir)
                .context("Failed to remove .scope/ from temp dir for no-scope run")?;
        }
    }

    // Handle preloaded scope map variant
    if condition == "with-scope-preloaded" {
        let preloaded_src = dest.join("CLAUDE.md.with-scope-preloaded");
        if preloaded_src.is_file() {
            let map_output = Command::new("scope")
                .args(["map"])
                .current_dir(dest)
                .output();
            match map_output {
                Ok(output) if output.status.success() => {
                    let map_text = String::from_utf8_lossy(&output.stdout);
                    let template = std::fs::read_to_string(&preloaded_src)?;
                    let rendered = template.replace("{{SCOPE_MAP_OUTPUT}}", &map_text);
                    std::fs::write(dest.join("CLAUDE.md"), rendered)?;
                }
                _ => {
                    // Fall back to regular with-scope
                    eprintln!("  Warning: scope map failed in temp dir. Falling back to with-scope variant.");
                    let fallback = dest.join("CLAUDE.md.with-scope");
                    if fallback.is_file() {
                        std::fs::copy(&fallback, dest.join("CLAUDE.md"))?;
                    }
                }
            }
        }
    }

    Ok(temp_dir)
}

/// Run a coding agent against a task.
///
/// Creates an isolated temp directory with the corpus, invokes the `claude`
/// CLI with stream-json output, parses the NDJSON stream for tool calls
/// and token usage, and returns the captured run data alongside the temp
/// directory handle. The caller MUST keep the returned `TempDir` alive
/// until verification completes — dropping it deletes the work directory.
///
/// Requires `ANTHROPIC_API_KEY` to be set and the `claude` CLI to be installed.
///
/// # Parameters
///
/// * `task` - The task definition containing the prompt and metadata
/// * `scope_enabled` - Whether Scope CLI is available for this run
/// * `condition` - Experimental condition label (e.g. "with-scope", "without-scope", "with-scope-preloaded")
/// * `corpus_path` - Path to the fixture corpus to copy
/// * `scope_backup` - Optional path to a `.scope/` backup to restore
/// * `model` - Optional model override (e.g. "claude-sonnet-4-20250514")
/// * `ndjson_save_path` - Optional path to save the raw NDJSON stream for later replay
pub fn run_agent(
    task: &TaskDef,
    scope_enabled: bool,
    condition: &str,
    corpus_path: &Path,
    scope_backup: Option<&Path>,
    model: Option<&str>,
    ndjson_save_path: Option<&Path>,
) -> Result<(AgentRun, tempfile::TempDir)> {
    // Pre-flight checks
    std::env::var("ANTHROPIC_API_KEY").map_err(|_| {
        anyhow::anyhow!(
            "ANTHROPIC_API_KEY is not set. Set it in your environment to run agent benchmarks."
        )
    })?;

    // Set up isolated temp directory
    let temp_dir = setup_temp_corpus(corpus_path, scope_enabled, condition, scope_backup)?;
    let work_dir = temp_dir.path();

    // Build the claude command.
    // On Windows, npm installs claude as claude.cmd. Batch files can't handle
    // complex arguments (quotes, newlines in prompts). Bypass the .cmd wrapper
    // and call node directly with the claude CLI entry point.
    let mut cmd = build_claude_command()?;
    cmd.arg("-p")
        .arg(&task.prompt.text)
        .arg("--bare")
        .arg("--add-dir")
        .arg(work_dir)
        .arg("--output-format")
        .arg("stream-json")
        .arg("--verbose")
        .arg("--max-turns")
        .arg("30")
        .arg("--allowedTools")
        .arg("Read,Edit,Write,Glob,Grep,Bash");

    if let Some(m) = model {
        cmd.arg("--model").arg(m);
    }

    if !scope_enabled {
        cmd.arg("--disallowedTools").arg("Bash(scope:*)");
    }

    cmd.current_dir(work_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let start = std::time::Instant::now();

    let mut child = cmd
        .spawn()
        .context("Failed to spawn 'claude' CLI. Is it installed and on PATH?")?;

    // Read stdout line by line and parse the NDJSON stream
    let stdout = child.stdout.take().context("Failed to capture stdout")?;
    let reader = std::io::BufReader::new(stdout);

    let mut input_tokens: u64 = 0;
    let mut output_tokens: u64 = 0;
    let mut cache_creation_input_tokens: u64 = 0;
    let mut cache_read_input_tokens: u64 = 0;
    let mut file_reads: u32 = 0;
    let mut file_edits: u32 = 0;
    let mut grep_calls: u32 = 0;
    let mut glob_calls: u32 = 0;
    let mut bash_calls: u32 = 0;
    let mut scope_commands_called: Vec<String> = Vec::new();
    let mut actions: Vec<AgentAction> = Vec::new();
    let mut sequence: u32 = 0;
    let mut raw_lines: Vec<String> = Vec::new();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        raw_lines.push(line.clone());

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let value: serde_json::Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => continue,
        };

        // Accumulate usage from any event that has it
        if let Some(usage) = value.get("usage") {
            input_tokens += usage
                .get("input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            output_tokens += usage
                .get("output_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            cache_creation_input_tokens += usage
                .get("cache_creation_input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            cache_read_input_tokens += usage
                .get("cache_read_input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
        }

        // Also check for usage nested inside a result event's sub-object
        if value.get("type").and_then(|t| t.as_str()) == Some("result") {
            // Top-level usage on result events is already handled above.
            // Some stream formats nest usage under a "result" sub-object.
            if let Some(result) = value.get("result") {
                if let Some(usage) = result.get("usage") {
                    input_tokens += usage
                        .get("input_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    output_tokens += usage
                        .get("output_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    cache_creation_input_tokens += usage
                        .get("cache_creation_input_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    cache_read_input_tokens += usage
                        .get("cache_read_input_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                }
            }
        }

        // Extract tool_use events from assistant messages
        // Pattern: {"type": "assistant", "message": {"content": [{"type": "tool_use", ...}]}}
        let content_items = extract_tool_use_items(&value);

        for item in content_items {
            let tool_name = match item.get("name").and_then(|n| n.as_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };

            let input_obj = item.get("input");
            let arguments_summary = extract_argument_summary(&tool_name, input_obj);

            // Classify the action
            let is_navigation = is_navigation_tool(&tool_name);
            let is_edit = is_edit_tool(&tool_name);
            let mut is_scope_cmd = false;

            match tool_name.as_str() {
                "Read" => file_reads += 1,
                "Edit" | "Write" => file_edits += 1,
                "Grep" => grep_calls += 1,
                "Glob" => glob_calls += 1,
                "Bash" => {
                    bash_calls += 1;
                    if let Some(cmd_str) = input_obj
                        .and_then(|i| i.get("command"))
                        .and_then(|c| c.as_str())
                    {
                        if let Some(scope_cmd) = extract_scope_command(cmd_str) {
                            is_scope_cmd = true;
                            if !scope_commands_called.contains(&scope_cmd) {
                                scope_commands_called.push(scope_cmd);
                            }
                        }
                    }
                }
                _ => {}
            }

            sequence += 1;
            let tool_display = tool_name.clone();
            actions.push(AgentAction {
                sequence,
                tool_name,
                arguments_summary,
                is_navigation,
                is_scope_command: is_scope_cmd,
                is_edit,
            });

            // Live status update
            let elapsed_secs = start.elapsed().as_secs();
            eprint!(
                "\r         ⏳ {}s │ {} actions │ {} reads │ {} out tokens │ last: {}        ",
                elapsed_secs, sequence, file_reads, output_tokens, tool_display
            );
        }
    }
    eprint!(
        "\r                                                                                    \r"
    ); // clear

    let status = child.wait().context("Failed to wait for claude process")?;
    let duration_ms = start.elapsed().as_millis() as u64;

    // Capture and log stderr if the process failed or produced no output
    if !status.success() || raw_lines.is_empty() {
        if let Some(mut stderr_stream) = child.stderr.take() {
            let mut stderr_text = String::new();
            std::io::Read::read_to_string(&mut stderr_stream, &mut stderr_text).ok();
            if !stderr_text.is_empty() {
                eprintln!(
                    "  [agent] claude CLI stderr (exit code {}):\n{}",
                    status.code().unwrap_or(-1),
                    stderr_text.trim()
                );
            }
        }
        if raw_lines.is_empty() {
            eprintln!("  [agent] WARNING: claude CLI produced no stdout output (0 NDJSON lines)");
            eprintln!(
                "  [agent] Exit code: {}, Duration: {}ms",
                status.code().unwrap_or(-1),
                duration_ms
            );
        }
    }

    // Save raw NDJSON if path provided
    if let Some(save_path) = ndjson_save_path {
        if let Some(parent) = save_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let ndjson_content = raw_lines.join("\n");
        std::fs::write(save_path, &ndjson_content)
            .with_context(|| format!("Failed to save NDJSON to {}", save_path.display()))?;
    }

    Ok((
        AgentRun {
            task_id: task.task.id.clone(),
            scope_enabled,
            input_tokens,
            output_tokens,
            cache_creation_input_tokens,
            cache_read_input_tokens,
            file_reads,
            file_edits,
            grep_calls,
            glob_calls,
            bash_calls,
            scope_commands_called,
            duration_ms,
            exit_code: status.code().unwrap_or(-1),
            actions,
        },
        temp_dir,
    ))
}

/// Extract tool_use items from a stream-json event.
///
/// Handles multiple nesting patterns:
/// - `{"type": "assistant", "message": {"content": [{"type": "tool_use", ...}]}}`
/// - `{"type": "content_block_start", "content_block": {"type": "tool_use", ...}}`
pub(crate) fn extract_tool_use_items(value: &serde_json::Value) -> Vec<&serde_json::Value> {
    let mut items = Vec::new();

    // Pattern 1: assistant message with content array
    if let Some(message) = value.get("message") {
        if let Some(content) = message.get("content").and_then(|c| c.as_array()) {
            for item in content {
                if item.get("type").and_then(|t| t.as_str()) == Some("tool_use") {
                    items.push(item);
                }
            }
        }
    }

    // Pattern 2: content_block_start with tool_use block
    if value.get("type").and_then(|t| t.as_str()) == Some("content_block_start") {
        if let Some(block) = value.get("content_block") {
            if block.get("type").and_then(|t| t.as_str()) == Some("tool_use") {
                items.push(block);
            }
        }
    }

    items
}

/// Extract a short human-readable summary of a tool call's arguments.
pub(crate) fn extract_argument_summary(
    tool_name: &str,
    input: Option<&serde_json::Value>,
) -> String {
    let input = match input {
        Some(i) => i,
        None => return String::new(),
    };

    match tool_name {
        "Read" => input
            .get("file_path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        "Edit" | "Write" => input
            .get("file_path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        "Grep" => input
            .get("pattern")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        "Glob" => input
            .get("pattern")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        "Bash" => {
            let cmd = input.get("command").and_then(|v| v.as_str()).unwrap_or("");
            if cmd.len() > 100 {
                format!("{}...", &cmd[..100])
            } else {
                cmd.to_string()
            }
        }
        _ => {
            // For unknown tools, try to produce a compact summary
            let s = input.to_string();
            if s.len() > 100 {
                format!("{}...", &s[..100])
            } else {
                s
            }
        }
    }
}

/// Check if a tool name is a navigation/exploration tool.
pub(crate) fn is_navigation_tool(name: &str) -> bool {
    matches!(name, "Read" | "Grep" | "Glob" | "Bash")
}

/// Check if a tool name is a file-editing tool.
pub(crate) fn is_edit_tool(name: &str) -> bool {
    matches!(name, "Edit" | "Write")
}

/// Extract the scope subcommand from a Bash command string.
///
/// Looks for "scope " anywhere in the command and extracts the subcommand.
/// For example:
/// - `"scope sketch PaymentService"` -> `Some("scope sketch")`
/// - `"cd foo && scope refs bar"` -> `Some("scope refs")`
/// - `"echo hello"` -> `None`
pub(crate) fn extract_scope_command(bash_command: &str) -> Option<String> {
    // Find "scope " in the command
    let idx = bash_command.find("scope ")?;
    let after_scope = &bash_command[idx + "scope ".len()..];

    // The subcommand is the next whitespace-delimited word
    let subcommand = after_scope.split_whitespace().next()?;

    // Filter out things that look like flags rather than subcommands
    if subcommand.starts_with('-') {
        return None;
    }

    Some(format!("scope {}", subcommand))
}

/// Public wrapper for prepare command to copy fixture directories.
/// Build a Command for invoking the claude CLI.
///
/// On Unix, this is simply `Command::new("claude")`.
/// On Windows, npm installs claude as a `.cmd` batch wrapper which cannot
/// handle complex arguments (special characters, newlines in prompts).
/// This function bypasses the wrapper by finding the actual JS entry point
/// and calling `node` directly.
fn build_claude_command() -> Result<Command> {
    if !cfg!(windows) {
        return Ok(Command::new("claude"));
    }

    // On Windows: find the npm global prefix, then the claude CLI JS entry point
    let npm_output = Command::new("cmd")
        .args(["/C", "npm", "config", "get", "prefix"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("Failed to run 'npm config get prefix'. Is npm installed?")?;

    let npm_prefix = String::from_utf8_lossy(&npm_output.stdout)
        .trim()
        .to_string();
    if npm_prefix.is_empty() {
        anyhow::bail!("npm config get prefix returned empty. Is npm installed?");
    }

    // The claude CLI entry point is at <prefix>/node_modules/@anthropic-ai/claude-code/cli.js
    let cli_js = std::path::PathBuf::from(&npm_prefix)
        .join("node_modules")
        .join("@anthropic-ai")
        .join("claude-code")
        .join("cli.js");

    if !cli_js.is_file() {
        anyhow::bail!(
            "Could not find claude CLI at {}. Is @anthropic-ai/claude-code installed globally?",
            cli_js.display()
        );
    }

    let mut cmd = Command::new("node");
    cmd.arg(&cli_js);
    Ok(cmd)
}

pub fn copy_dir_for_prepare(src: &Path, dst: &Path) -> Result<()> {
    copy_dir_recursive(src, dst)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_scope_command_basic() {
        assert_eq!(
            extract_scope_command("scope sketch PaymentService"),
            Some("scope sketch".to_string())
        );
    }

    #[test]
    fn test_extract_scope_command_chained() {
        assert_eq!(
            extract_scope_command("cd /tmp && scope refs processPayment --json"),
            Some("scope refs".to_string())
        );
    }

    #[test]
    fn test_extract_scope_command_with_path() {
        assert_eq!(
            extract_scope_command("./node_modules/.bin/scope find foo"),
            Some("scope find".to_string())
        );
    }

    #[test]
    fn test_extract_scope_command_no_match() {
        assert_eq!(extract_scope_command("echo hello world"), None);
    }

    #[test]
    fn test_extract_scope_command_flag_only() {
        assert_eq!(extract_scope_command("scope --version"), None);
    }

    #[test]
    fn test_extract_scope_command_callers() {
        assert_eq!(
            extract_scope_command("scope callers processPayment"),
            Some("scope callers".to_string())
        );
    }

    #[test]
    fn test_is_navigation_tool() {
        assert!(is_navigation_tool("Read"));
        assert!(is_navigation_tool("Grep"));
        assert!(is_navigation_tool("Glob"));
        assert!(is_navigation_tool("Bash"));
        assert!(!is_navigation_tool("Edit"));
        assert!(!is_navigation_tool("Write"));
    }

    #[test]
    fn test_is_edit_tool() {
        assert!(is_edit_tool("Edit"));
        assert!(is_edit_tool("Write"));
        assert!(!is_edit_tool("Read"));
        assert!(!is_edit_tool("Bash"));
    }

    #[test]
    fn test_extract_argument_summary_read() {
        let input: serde_json::Value = serde_json::json!({"file_path": "src/main.rs"});
        assert_eq!(
            extract_argument_summary("Read", Some(&input)),
            "src/main.rs"
        );
    }

    #[test]
    fn test_extract_argument_summary_bash_truncates() {
        let long_cmd = "a".repeat(200);
        let input = serde_json::json!({"command": long_cmd});
        let summary = extract_argument_summary("Bash", Some(&input));
        assert_eq!(summary.len(), 103); // 100 chars + "..."
        assert!(summary.ends_with("..."));
    }

    #[test]
    fn test_extract_argument_summary_grep() {
        let input = serde_json::json!({"pattern": "processPayment"});
        assert_eq!(
            extract_argument_summary("Grep", Some(&input)),
            "processPayment"
        );
    }

    #[test]
    fn test_extract_tool_use_items_assistant_message() {
        let event = serde_json::json!({
            "type": "assistant",
            "message": {
                "content": [
                    {"type": "tool_use", "name": "Read", "input": {"file_path": "foo.rs"}},
                    {"type": "text", "text": "hello"},
                    {"type": "tool_use", "name": "Grep", "input": {"pattern": "bar"}}
                ]
            }
        });
        let items = extract_tool_use_items(&event);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["name"], "Read");
        assert_eq!(items[1]["name"], "Grep");
    }

    #[test]
    fn test_extract_tool_use_items_content_block_start() {
        let event = serde_json::json!({
            "type": "content_block_start",
            "content_block": {
                "type": "tool_use",
                "name": "Bash",
                "input": {"command": "ls"}
            }
        });
        let items = extract_tool_use_items(&event);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["name"], "Bash");
    }

    #[test]
    fn test_extract_tool_use_items_irrelevant_event() {
        let event = serde_json::json!({"type": "ping"});
        let items = extract_tool_use_items(&event);
        assert!(items.is_empty());
    }

    #[test]
    fn test_parse_ndjson_actions_empty() {
        let result = parse_ndjson_actions("");
        assert!(result.actions.is_empty());
        assert_eq!(result.input_tokens, 0);
        assert_eq!(result.output_tokens, 0);
        assert_eq!(result.file_reads, 0);
    }

    #[test]
    fn test_parse_ndjson_actions_basic() {
        let ndjson = r#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Read","input":{"file_path":"src/main.rs"}},{"type":"tool_use","name":"Bash","input":{"command":"scope sketch PaymentService"}}]}}
{"usage":{"input_tokens":5000,"output_tokens":1000,"cache_creation_input_tokens":200,"cache_read_input_tokens":4000}}"#;

        let result = parse_ndjson_actions(ndjson);
        assert_eq!(result.actions.len(), 2);
        assert_eq!(result.actions[0].tool_name, "Read");
        assert_eq!(result.actions[0].sequence, 1);
        assert!(result.actions[0].is_navigation);
        assert!(!result.actions[0].is_edit);
        assert_eq!(result.actions[1].tool_name, "Bash");
        assert!(result.actions[1].is_scope_command);
        assert_eq!(result.input_tokens, 5000);
        assert_eq!(result.output_tokens, 1000);
        assert_eq!(result.cache_creation_input_tokens, 200);
        assert_eq!(result.cache_read_input_tokens, 4000);
        assert_eq!(result.file_reads, 1);
        assert_eq!(
            result.scope_commands_called,
            vec!["scope sketch".to_string()]
        );
    }

    #[test]
    fn test_parse_ndjson_actions_with_malformed_lines() {
        let ndjson = "not json\n{\"usage\":{\"input_tokens\":100,\"output_tokens\":50}}\n\n";
        let result = parse_ndjson_actions(ndjson);
        assert!(result.actions.is_empty());
        assert_eq!(result.input_tokens, 100);
        assert_eq!(result.output_tokens, 50);
    }
}
