use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

use crate::task::TaskDef;

/// Result of verifying an agent's work on a task.
#[derive(Debug)]
pub struct VerificationResult {
    /// Whether the code compiles after the agent's changes
    pub compilation_pass: bool,
    /// Whether the test suite passes after the agent's changes
    pub tests_pass: bool,
    /// Caller coverage score (0.0–1.0), if applicable for this task
    pub caller_coverage: Option<f64>,
    /// Overall correctness score (0–100)
    pub overall_score: u32,
}

/// Verify the correctness of an agent's work on a task.
///
/// Runs compilation, tests, and (where applicable) caller coverage checks
/// against the corpus after the agent has made its changes.
pub fn verify_task(corpus_path: &Path, task: &TaskDef) -> Result<VerificationResult> {
    let compilation_pass = if task.correctness.require_compilation {
        run_compilation(corpus_path, &task.task.language)?
    } else {
        true
    };

    let tests_pass = if task.correctness.require_tests_pass {
        run_tests(corpus_path, &task.task.language)?
    } else {
        true
    };

    let caller_coverage = if task.correctness.require_caller_coverage {
        Some(check_caller_coverage(corpus_path, task)?)
    } else {
        None
    };

    let overall_score = calculate_score(compilation_pass, tests_pass, caller_coverage, task);

    Ok(VerificationResult {
        compilation_pass,
        tests_pass,
        caller_coverage,
        overall_score,
    })
}

/// Run the language-specific compilation check.
fn run_compilation(corpus_path: &Path, language: &str) -> Result<bool> {
    let status = match language {
        "typescript" => Command::new("npx")
            .args(["tsc", "--noEmit"])
            .current_dir(corpus_path)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .status()
            .context("Failed to run TypeScript compiler. Is npx/tsc installed?")?,
        "csharp" => Command::new("dotnet")
            .args(["build", "--no-restore"])
            .current_dir(corpus_path)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .status()
            .context("Failed to run dotnet build. Is the .NET SDK installed?")?,
        other => {
            anyhow::bail!("Unsupported language for compilation check: {}", other);
        }
    };

    Ok(status.success())
}

/// Run the language-specific test suite.
fn run_tests(corpus_path: &Path, language: &str) -> Result<bool> {
    let status = match language {
        "typescript" => Command::new("npm")
            .args(["test"])
            .current_dir(corpus_path)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .status()
            .context("Failed to run npm test. Is npm installed?")?,
        "csharp" => Command::new("dotnet")
            .args(["test", "--no-restore"])
            .current_dir(corpus_path)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .status()
            .context("Failed to run dotnet test. Is the .NET SDK installed?")?,
        other => {
            anyhow::bail!("Unsupported language for test check: {}", other);
        }
    };

    Ok(status.success())
}

/// Check caller coverage by comparing the git diff against ground truth callers.
///
/// Uses the Python verification script to cross-reference the diff against
/// the known callers from `scope refs --json`.
fn check_caller_coverage(corpus_path: &Path, task: &TaskDef) -> Result<f64> {
    // Generate the diff
    let diff_output = Command::new("git")
        .args(["diff", "HEAD"])
        .current_dir(corpus_path)
        .output()
        .context("Failed to run git diff for caller coverage check")?;

    if !diff_output.status.success() {
        anyhow::bail!("git diff failed during caller coverage check");
    }

    let diff_text = String::from_utf8_lossy(&diff_output.stdout);

    if diff_text.is_empty() {
        // No changes made — 0% coverage
        return Ok(0.0);
    }

    // For now, return a placeholder. The full implementation will invoke
    // the Python verification script or implement the logic in Rust.
    //
    // TODO: Integrate with benchmarks/verify/verify_caller_coverage.py
    eprintln!(
        "  [verifier] Caller coverage check for {} — stub returning 0.0",
        task.target.symbol
    );
    Ok(0.0)
}

/// Calculate the overall correctness score (0–100) from individual checks.
///
/// Scoring:
/// - Compilation pass: 40 points (or proportionally more if fewer checks apply)
/// - Tests pass: 40 points
/// - Caller coverage: 20 points (scaled by coverage ratio)
///
/// If caller coverage is not applicable, compilation and tests are each worth 50 points.
fn calculate_score(
    compilation_pass: bool,
    tests_pass: bool,
    caller_coverage: Option<f64>,
    task: &TaskDef,
) -> u32 {
    match caller_coverage {
        Some(coverage) => {
            let mut score = 0u32;
            if compilation_pass {
                score += 40;
            }
            if tests_pass {
                score += 40;
            }
            // Scale caller coverage: threshold defines the minimum acceptable
            let coverage_score = if coverage >= task.correctness.caller_coverage_threshold {
                20
            } else {
                (coverage / task.correctness.caller_coverage_threshold * 20.0) as u32
            };
            score += coverage_score;
            score
        }
        None => {
            let mut score = 0u32;
            if compilation_pass {
                score += 50;
            }
            if tests_pass {
                score += 50;
            }
            score
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::{CorrectnessDef, PromptDef, ScopeDef, TargetDef, TaskMeta};

    fn make_task_def(require_caller_coverage: bool) -> TaskDef {
        TaskDef {
            task: TaskMeta {
                id: "test-task".to_string(),
                category: "test".to_string(),
                language: "typescript".to_string(),
                corpus: "fixture".to_string(),
                description: "Test task".to_string(),
            },
            prompt: PromptDef {
                text: "Do something".to_string(),
            },
            target: TargetDef {
                symbol: "Foo.bar".to_string(),
                file: "src/foo.ts".to_string(),
            },
            correctness: CorrectnessDef {
                require_compilation: true,
                require_tests_pass: true,
                require_caller_coverage,
                caller_coverage_threshold: 1.0,
                pattern_match_threshold: 0.0,
            },
            scope: ScopeDef {
                expected_commands: vec!["scope refs".to_string()],
            },
        }
    }

    #[test]
    fn test_score_calculation_without_caller_coverage() {
        let task = make_task_def(false);
        assert_eq!(calculate_score(true, true, None, &task), 100);
        assert_eq!(calculate_score(true, false, None, &task), 50);
        assert_eq!(calculate_score(false, true, None, &task), 50);
        assert_eq!(calculate_score(false, false, None, &task), 0);
    }

    #[test]
    fn test_score_calculation_with_full_caller_coverage() {
        let task = make_task_def(true);
        assert_eq!(calculate_score(true, true, Some(1.0), &task), 100);
        assert_eq!(calculate_score(true, true, Some(0.5), &task), 90);
        assert_eq!(calculate_score(false, false, Some(1.0), &task), 20);
    }

    #[test]
    fn test_score_calculation_with_partial_caller_coverage() {
        let task = make_task_def(true);
        // 0.8 coverage with 1.0 threshold => 16 points from coverage
        assert_eq!(calculate_score(true, true, Some(0.8), &task), 96);
    }
}
