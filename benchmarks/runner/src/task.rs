use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;
use walkdir::WalkDir;

/// A complete task definition loaded from a TOML file.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct TaskDef {
    pub task: TaskMeta,
    pub prompt: PromptDef,
    pub target: TargetDef,
    pub correctness: CorrectnessDef,
    #[serde(default)]
    pub ground_truth: GroundTruthDef,
    pub scope: ScopeDef,
}

/// Metadata about the task — id, category, language, corpus.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct TaskMeta {
    /// Unique identifier for the task (e.g. "ts-cat-a-01")
    pub id: String,
    /// Task category (e.g. "signature-refactoring", "cross-cutting")
    pub category: String,
    /// Programming language of the target corpus (e.g. "typescript")
    pub language: String,
    /// Which corpus to run against ("fixture" or a named corpus)
    pub corpus: String,
    /// Human-readable description of the task
    pub description: String,
}

/// The prompt that gets sent to the coding agent.
#[derive(Debug, Deserialize)]
pub struct PromptDef {
    /// The exact text sent to the agent — never paraphrased
    pub text: String,
}

/// The target symbol and file for the task.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct TargetDef {
    /// The target symbol (e.g. "PaymentService.processPayment")
    pub symbol: String,
    /// The file containing the target symbol
    pub file: String,
}

/// Correctness criteria for verifying the agent's work.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct CorrectnessDef {
    /// Whether the code must compile after the agent's changes
    pub require_compilation: bool,
    /// Whether the test suite must pass after the agent's changes
    pub require_tests_pass: bool,
    /// Whether caller coverage is checked (Category A and D tasks)
    pub require_caller_coverage: bool,
    /// Minimum caller coverage threshold (0.0–1.0). Only used when require_caller_coverage is true.
    pub caller_coverage_threshold: f64,
    /// Minimum pattern match threshold (0.0–1.0). Only used for Category F tasks.
    pub pattern_match_threshold: f64,
}

/// Ground truth data for verification (optional).
///
/// Used by the verifier to check whether an agent's changes cover
/// all known callers of the target symbol.
#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
pub struct GroundTruthDef {
    /// Known caller locations in "file_path:line" format.
    /// Example: ["src/Api/Controllers/PaymentController.cs:45"]
    #[serde(default)]
    pub callers: Vec<String>,
}

/// Expected Scope commands for verifying the agent used Scope.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ScopeDef {
    /// The sc commands a well-instrumented agent should call
    pub expected_commands: Vec<String>,
}

/// Load all task definitions from a directory, recursively searching for TOML files.
pub fn load_tasks(tasks_dir: &Path) -> Result<Vec<TaskDef>> {
    let mut tasks = Vec::new();

    for entry in WalkDir::new(tasks_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file() && e.path().extension().is_some_and(|ext| ext == "toml")
        })
    {
        let task = load_task(entry.path())
            .with_context(|| format!("Failed to load task from {}", entry.path().display()))?;
        tasks.push(task);
    }

    // Sort by task ID for deterministic ordering
    tasks.sort_by(|a, b| a.task.id.cmp(&b.task.id));

    Ok(tasks)
}

/// Load a single task definition from a TOML file.
pub fn load_task(path: &Path) -> Result<TaskDef> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Could not read task file: {}", path.display()))?;

    let task_def: TaskDef = toml::from_str(&content)
        .with_context(|| format!("Invalid TOML in task file: {}", path.display()))?;

    // Validate required fields
    if task_def.task.id.is_empty() {
        anyhow::bail!("Task in {} has empty id", path.display());
    }
    if task_def.prompt.text.is_empty() {
        anyhow::bail!("Task {} has empty prompt text", task_def.task.id);
    }

    Ok(task_def)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_load_all_task_files_parse_successfully() {
        // Find the tasks directory relative to this test file
        let candidates = [
            PathBuf::from("benchmarks/tasks"),
            PathBuf::from("../tasks"),
            PathBuf::from("../../benchmarks/tasks"),
            PathBuf::from("../../tasks"),
        ];

        let tasks_dir = candidates
            .iter()
            .find(|p| p.is_dir())
            .expect("Could not find benchmarks/tasks/ directory. Run tests from project root or benchmarks/runner/.");

        let tasks = load_tasks(tasks_dir).expect("Failed to load tasks");
        assert!(
            !tasks.is_empty(),
            "Expected at least one task file in {}",
            tasks_dir.display()
        );
        assert_eq!(
            tasks.len(),
            16,
            "Expected 16 tasks (8 TypeScript + 8 C#), found {}",
            tasks.len()
        );

        for task in &tasks {
            assert!(!task.task.id.is_empty(), "Task has empty id");
            assert!(
                !task.task.category.is_empty(),
                "Task {} has empty category",
                task.task.id
            );
            assert!(
                !task.task.language.is_empty(),
                "Task {} has empty language",
                task.task.id
            );
            assert!(
                !task.prompt.text.is_empty(),
                "Task {} has empty prompt",
                task.task.id
            );
            assert!(
                !task.target.symbol.is_empty(),
                "Task {} has empty target symbol",
                task.task.id
            );
            assert!(
                !task.target.file.is_empty(),
                "Task {} has empty target file",
                task.task.id
            );
        }

        eprintln!("Successfully loaded {} task(s)", tasks.len());
    }
}
