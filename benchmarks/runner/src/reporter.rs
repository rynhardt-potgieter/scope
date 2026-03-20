use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::Path;

use crate::agent::AgentAction;
use crate::behavior::{self, BehaviorMetrics};

/// A single benchmark run result.
#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkRun {
    pub task_id: String,
    pub repetition: u32,
    pub scope_enabled: bool,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub file_reads: u32,
    pub scope_commands_called: Vec<String>,
    pub correctness: CorrectnessResult,
    pub duration_ms: u64,
    #[serde(default)]
    pub actions: Vec<AgentAction>,
    #[serde(default)]
    pub behavior: Option<BehaviorMetrics>,
}

/// Correctness verification results for a single run.
#[derive(Debug, Serialize, Deserialize)]
pub struct CorrectnessResult {
    pub compilation_pass: bool,
    pub tests_pass: bool,
    pub caller_coverage: Option<f64>,
    pub overall_score: u32,
}

/// Top-level wrapper for serialized results.
#[derive(Debug, Serialize, Deserialize)]
pub struct ResultsWrapper {
    pub scope_version: String,
    pub benchmark_version: String,
    pub run_date: String,
    pub runs: Vec<BenchmarkRun>,
}

/// Write benchmark results as JSON.
pub fn write_json_results(results: &[BenchmarkRun], path: &Path) -> Result<()> {
    let wrapper = ResultsWrapper {
        scope_version: detect_scope_version(),
        benchmark_version: "1".to_string(),
        run_date: chrono::Utc::now().to_rfc3339(),
        runs: results.to_vec(),
    };

    let json =
        serde_json::to_string_pretty(&wrapper).context("Failed to serialize results to JSON")?;

    let mut file = std::fs::File::create(path)
        .with_context(|| format!("Failed to create results file: {}", path.display()))?;
    file.write_all(json.as_bytes())?;

    Ok(())
}

/// Write a Markdown summary of benchmark results.
pub fn write_markdown_summary(results: &[BenchmarkRun], path: &Path) -> Result<()> {
    let mut out = String::new();

    let run_date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let scope_version = detect_scope_version();

    out.push_str(&format!("## Scope {} Benchmark Results\n\n", scope_version));
    out.push_str(&format!("**Benchmark date:** {}\n", run_date));

    let task_ids: Vec<&str> = results
        .iter()
        .map(|r| r.task_id.as_str())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    out.push_str(&format!("**Tasks:** {}\n", task_ids.len()));

    let max_rep = results.iter().map(|r| r.repetition).max().unwrap_or(1);
    out.push_str(&format!(
        "**Repetitions:** {} per task per condition\n\n",
        max_rep
    ));

    // Token consumption summary
    let with_scope: Vec<&BenchmarkRun> = results.iter().filter(|r| r.scope_enabled).collect();
    let without_scope: Vec<&BenchmarkRun> = results.iter().filter(|r| !r.scope_enabled).collect();

    out.push_str("### Token Consumption\n\n");
    out.push_str("| Condition | Mean input tokens | Reduction |\n");
    out.push_str("|-----------|-------------------|-----------|\n");

    let mean_without = mean_tokens(&without_scope);
    let mean_with = mean_tokens(&with_scope);

    if !without_scope.is_empty() {
        out.push_str(&format!("| Without Scope | {:.0} | — |\n", mean_without));
    }
    if !with_scope.is_empty() {
        let reduction = if mean_without > 0.0 {
            format!("**{:.1}%**", (1.0 - mean_with / mean_without) * 100.0)
        } else {
            "N/A".to_string()
        };
        out.push_str(&format!(
            "| With Scope | {:.0} | {} |\n",
            mean_with, reduction
        ));
    }

    // Correctness summary
    out.push_str("\n### Task Correctness\n\n");
    out.push_str("| Condition | Compilation pass | Tests pass | Mean score |\n");
    out.push_str("|-----------|-----------------|------------|------------|\n");

    if !without_scope.is_empty() {
        let (comp_pct, test_pct, mean_score) = correctness_stats(&without_scope);
        out.push_str(&format!(
            "| Without Scope | {:.0}% | {:.0}% | {:.0} |\n",
            comp_pct, test_pct, mean_score
        ));
    }
    if !with_scope.is_empty() {
        let (comp_pct, test_pct, mean_score) = correctness_stats(&with_scope);
        out.push_str(&format!(
            "| With Scope | {:.0}% | {:.0}% | {:.0} |\n",
            comp_pct, test_pct, mean_score
        ));
    }

    // File reads summary
    out.push_str("\n### File Reads per Task\n\n");
    out.push_str("| Condition | Mean file reads |\n");
    out.push_str("|-----------|----------------|\n");

    if !without_scope.is_empty() {
        let mean_reads = mean_file_reads(&without_scope);
        out.push_str(&format!("| Without Scope | {:.1} |\n", mean_reads));
    }
    if !with_scope.is_empty() {
        let mean_reads = mean_file_reads(&with_scope);
        out.push_str(&format!("| With Scope | {:.1} |\n", mean_reads));
    }

    // Per-category breakdown
    out.push_str("\n### By Category\n\n");
    out.push_str("| Category | With Scope (tokens) | Without Scope (tokens) | Reduction |\n");
    out.push_str("|----------|--------------------|-----------------------|-----------|\n");

    let categories = extract_categories(results);
    for category in &categories {
        let cat_with: Vec<&BenchmarkRun> = with_scope
            .iter()
            .filter(|r| task_category(&r.task_id) == *category)
            .copied()
            .collect();
        let cat_without: Vec<&BenchmarkRun> = without_scope
            .iter()
            .filter(|r| task_category(&r.task_id) == *category)
            .copied()
            .collect();

        let m_with = mean_tokens(&cat_with);
        let m_without = mean_tokens(&cat_without);
        let reduction = if m_without > 0.0 {
            format!("{:.1}%", (1.0 - m_with / m_without) * 100.0)
        } else {
            "N/A".to_string()
        };

        out.push_str(&format!(
            "| {} | {:.0} | {:.0} | {} |\n",
            category, m_with, m_without, reduction
        ));
    }

    out.push_str(&format!(
        "\n*All results are means across {} repetitions per task.*\n",
        max_rep
    ));

    // Behavior analysis section
    let with_behaviors: Vec<BehaviorMetrics> = with_scope
        .iter()
        .filter_map(|r| r.behavior.clone())
        .collect();
    let without_behaviors: Vec<BehaviorMetrics> = without_scope
        .iter()
        .filter_map(|r| r.behavior.clone())
        .collect();

    if !with_behaviors.is_empty() || !without_behaviors.is_empty() {
        let comparison = behavior::aggregate_behavior(&with_behaviors, &without_behaviors);
        out.push('\n');
        out.push_str(&behavior::format_behavior_markdown(&comparison));

        let scope_sequences: Vec<Vec<String>> = with_behaviors
            .iter()
            .map(|b| b.scope_command_sequence.clone())
            .collect();
        out.push('\n');
        out.push_str(&behavior::generate_recommendations(&comparison, &scope_sequences));
    }

    let mut file = std::fs::File::create(path)
        .with_context(|| format!("Failed to create summary file: {}", path.display()))?;
    file.write_all(out.as_bytes())?;

    Ok(())
}

/// Write environment information to a JSON file.
pub fn write_environment(path: &Path) -> Result<()> {
    let env = serde_json::json!({
        "scope_version": detect_scope_version(),
        "benchmark_version": "1",
        "run_date": chrono::Utc::now().to_rfc3339(),
        "machine": {
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
        },
    });

    let json =
        serde_json::to_string_pretty(&env).context("Failed to serialize environment info")?;

    let mut file = std::fs::File::create(path)
        .with_context(|| format!("Failed to create environment file: {}", path.display()))?;
    file.write_all(json.as_bytes())?;

    Ok(())
}

/// Detect the Scope version by running `sc --version` or falling back to "unknown".
fn detect_scope_version() -> String {
    std::process::Command::new("scope")
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| {
            String::from_utf8(output.stdout)
                .ok()
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_else(|| "unknown".to_string())
}

/// Calculate mean input tokens from a set of runs.
fn mean_tokens(runs: &[&BenchmarkRun]) -> f64 {
    if runs.is_empty() {
        return 0.0;
    }
    let total: u64 = runs.iter().map(|r| r.input_tokens).sum();
    total as f64 / runs.len() as f64
}

/// Calculate mean file reads from a set of runs.
fn mean_file_reads(runs: &[&BenchmarkRun]) -> f64 {
    if runs.is_empty() {
        return 0.0;
    }
    let total: u32 = runs.iter().map(|r| r.file_reads).sum();
    total as f64 / runs.len() as f64
}

/// Calculate correctness statistics: (compilation_pass_%, tests_pass_%, mean_score).
fn correctness_stats(runs: &[&BenchmarkRun]) -> (f64, f64, f64) {
    if runs.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    let n = runs.len() as f64;
    let comp = runs
        .iter()
        .filter(|r| r.correctness.compilation_pass)
        .count() as f64;
    let test = runs.iter().filter(|r| r.correctness.tests_pass).count() as f64;
    let score: u32 = runs.iter().map(|r| r.correctness.overall_score).sum();

    (comp / n * 100.0, test / n * 100.0, score as f64 / n)
}

/// Extract unique categories from task IDs.
///
/// Derives category from task ID pattern: "ts-cat-X-NN" => "cat-X"
fn extract_categories(results: &[BenchmarkRun]) -> Vec<String> {
    let mut categories: Vec<String> = results
        .iter()
        .map(|r| task_category(&r.task_id))
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    categories.sort();
    categories
}

/// Extract category from a task ID (e.g. "ts-cat-a-01" => "cat-a").
fn task_category(task_id: &str) -> String {
    let parts: Vec<&str> = task_id.split('-').collect();
    if parts.len() >= 3 {
        format!("{}-{}", parts[1], parts[2])
    } else {
        "unknown".to_string()
    }
}

// We need Clone on BenchmarkRun and CorrectnessResult for serialization in the wrapper
impl Clone for BenchmarkRun {
    fn clone(&self) -> Self {
        Self {
            task_id: self.task_id.clone(),
            repetition: self.repetition,
            scope_enabled: self.scope_enabled,
            input_tokens: self.input_tokens,
            output_tokens: self.output_tokens,
            file_reads: self.file_reads,
            scope_commands_called: self.scope_commands_called.clone(),
            correctness: self.correctness.clone(),
            duration_ms: self.duration_ms,
            actions: self.actions.clone(),
            behavior: self.behavior.clone(),
        }
    }
}

impl Clone for CorrectnessResult {
    fn clone(&self) -> Self {
        Self {
            compilation_pass: self.compilation_pass,
            tests_pass: self.tests_pass,
            caller_coverage: self.caller_coverage,
            overall_score: self.overall_score,
        }
    }
}
