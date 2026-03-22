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
/// Build a Command that works on both Unix and Windows.
///
/// On Windows, npm-installed tools (npx, npm) are .cmd scripts that
/// `Command::new("npx")` won't find. Use the .cmd extension directly.
fn shell_command(program: &str) -> Command {
    if cfg!(windows) {
        Command::new(format!("{program}.cmd"))
    } else {
        Command::new(program)
    }
}

fn run_compilation(corpus_path: &Path, language: &str) -> Result<bool> {
    let status = match language {
        "typescript" => shell_command("npx")
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
        "typescript" => shell_command("npm")
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

/// Parse a unified diff and extract modified line numbers per file.
///
/// Returns a map from file path to set of modified line numbers.
/// Only tracks lines in the new version (lines with `+` prefix in the diff).
fn parse_diff_modifications(
    diff_text: &str,
) -> std::collections::HashMap<String, std::collections::HashSet<i64>> {
    let mut modifications: std::collections::HashMap<String, std::collections::HashSet<i64>> =
        std::collections::HashMap::new();
    let mut current_file: Option<String> = None;
    let mut current_line: i64 = 0;

    for line in diff_text.lines() {
        // Track current file from "+++ b/path" lines
        if let Some(path) = line.strip_prefix("+++ b/") {
            current_file = Some(path.to_string());
            continue;
        }
        if line.starts_with("+++ ") || line.starts_with("--- ") {
            continue;
        }

        // Parse hunk headers: @@ -old_start,old_count +new_start,new_count @@
        if line.starts_with("@@ ") {
            if let Some(plus_idx) = line.find('+') {
                let after_plus = &line[plus_idx + 1..];
                let num_str: String = after_plus
                    .chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect();
                if let Ok(start) = num_str.parse::<i64>() {
                    current_line = start;
                }
            }
            continue;
        }

        // Track modifications
        if let Some(ref file) = current_file {
            if line.starts_with('+') {
                modifications
                    .entry(file.clone())
                    .or_default()
                    .insert(current_line);
                current_line += 1;
            } else if line.starts_with('-') {
                // Deleted line — don't increment new line counter
            } else {
                // Context line
                current_line += 1;
            }
        }
    }

    modifications
}

/// Check if a line (within a context window) appears in the diff modifications.
fn is_line_modified(
    modifications: &std::collections::HashMap<String, std::collections::HashSet<i64>>,
    file_path: &str,
    line: i64,
    context: i64,
) -> bool {
    if let Some(file_mods) = modifications.get(file_path) {
        file_mods
            .iter()
            .any(|&mod_line| (line - mod_line).abs() <= context)
    } else {
        false
    }
}

/// Parse a caller location string in "file_path:line" format.
fn parse_caller_location(s: &str) -> Result<(&str, i64)> {
    let colon_idx = s.rfind(':').ok_or_else(|| {
        anyhow::anyhow!(
            "Invalid caller location format '{}': expected 'file:line'",
            s
        )
    })?;
    let file_path = &s[..colon_idx];
    let line: i64 = s[colon_idx + 1..]
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid line number in caller location '{}': {}", s, e))?;
    Ok((file_path, line))
}

/// Check caller coverage by comparing the git diff against ground truth callers.
///
/// Cross-references the diff against the known callers from the task's
/// ground truth definition.
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
        return Ok(0.0);
    }

    // Get ground truth callers from task definition
    let callers = &task.ground_truth.callers;
    if callers.is_empty() {
        eprintln!(
            "  [verifier] No ground truth callers defined for {}. Skipping coverage check.",
            task.target.symbol
        );
        return Ok(0.0);
    }

    // Parse the diff to find modified lines
    let modifications = parse_diff_modifications(&diff_text);

    // Check which callers were touched (±5 line context window)
    let context = 5i64;
    let total = callers.len();
    let mut covered = 0usize;

    for caller in callers {
        let (file_path, line) = parse_caller_location(caller)?;
        if is_line_modified(&modifications, file_path, line, context) {
            covered += 1;
        }
    }

    let coverage = covered as f64 / total as f64;
    eprintln!(
        "  [verifier] Caller coverage for {}: {}/{} ({:.0}%)",
        task.target.symbol,
        covered,
        total,
        coverage * 100.0
    );

    Ok(coverage)
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
    use crate::task::{CorrectnessDef, GroundTruthDef, PromptDef, ScopeDef, TargetDef, TaskMeta};

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
            ground_truth: GroundTruthDef::default(),
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

    #[test]
    fn test_parse_diff_modifications_basic() {
        let diff = "\
diff --git a/src/foo.ts b/src/foo.ts
--- a/src/foo.ts
+++ b/src/foo.ts
@@ -10,3 +10,4 @@ function foo() {
     existing line
+    new line
     another line
";
        let mods = parse_diff_modifications(diff);
        assert!(mods.contains_key("src/foo.ts"));
        assert!(mods["src/foo.ts"].contains(&11));
        assert!(!mods["src/foo.ts"].contains(&10));
    }

    #[test]
    fn test_parse_diff_modifications_multifile() {
        let diff = "\
diff --git a/src/a.ts b/src/a.ts
--- a/src/a.ts
+++ b/src/a.ts
@@ -5,2 +5,3 @@
     context
+    added in a
diff --git a/src/b.ts b/src/b.ts
--- a/src/b.ts
+++ b/src/b.ts
@@ -20,2 +20,3 @@
     context
+    added in b
";
        let mods = parse_diff_modifications(diff);
        assert_eq!(mods.len(), 2);
        assert!(mods.contains_key("src/a.ts"));
        assert!(mods.contains_key("src/b.ts"));
        assert!(mods["src/a.ts"].contains(&6));
        assert!(mods["src/b.ts"].contains(&21));
    }

    #[test]
    fn test_parse_diff_modifications_empty() {
        let mods = parse_diff_modifications("");
        assert!(mods.is_empty());
    }

    #[test]
    fn test_is_line_modified_within_context() {
        let mut mods = std::collections::HashMap::new();
        let mut lines = std::collections::HashSet::new();
        lines.insert(50i64);
        mods.insert("src/foo.ts".to_string(), lines);

        assert!(is_line_modified(&mods, "src/foo.ts", 48, 5));
        assert!(is_line_modified(&mods, "src/foo.ts", 55, 5));
        assert!(is_line_modified(&mods, "src/foo.ts", 50, 5));
    }

    #[test]
    fn test_is_line_modified_outside_context() {
        let mut mods = std::collections::HashMap::new();
        let mut lines = std::collections::HashSet::new();
        lines.insert(50i64);
        mods.insert("src/foo.ts".to_string(), lines);

        assert!(!is_line_modified(&mods, "src/foo.ts", 60, 5));
        assert!(!is_line_modified(&mods, "src/foo.ts", 40, 5));
        assert!(!is_line_modified(&mods, "other.ts", 50, 5));
    }

    #[test]
    fn test_parse_caller_location() {
        let (file, line) =
            parse_caller_location("src/Api/Controllers/PaymentController.cs:45").unwrap();
        assert_eq!(file, "src/Api/Controllers/PaymentController.cs");
        assert_eq!(line, 45);
    }

    #[test]
    fn test_parse_caller_location_invalid() {
        assert!(parse_caller_location("no-colon").is_err());
        assert!(parse_caller_location("file:notanumber").is_err());
    }
}
