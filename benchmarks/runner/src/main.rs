mod agent;
mod behavior;
mod git;
mod reporter;
mod task;
mod verifier;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};

/// Benchmark harness for the Scope CLI tool.
///
/// Runs coding tasks with and without Scope enabled, measures token consumption,
/// task correctness, and navigation efficiency. Results are written as JSON or
/// Markdown for inclusion in release notes.
#[derive(Parser, Debug)]
#[command(
    name = "benchmark",
    about = "Benchmark harness for Scope CLI",
    version,
    propagate_version = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run benchmark tasks against codebases with and without Scope.
    ///
    /// By default, runs all tasks with Scope enabled. Use --compare to run
    /// both with and without Scope for a side-by-side comparison. Use --no-scope
    /// to run only the baseline (no Scope) condition.
    Run(RunArgs),

    /// Generate a report from existing benchmark results.
    ///
    /// Reads a previously saved JSON results file and produces a summary
    /// in the requested format (JSON or Markdown).
    Report(ReportArgs),
}

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Json,
    Markdown,
}

#[derive(Parser, Debug)]
pub struct RunArgs {
    /// Run all tasks in the task suite
    #[arg(long)]
    pub all: bool,

    /// Run a single task by its ID (e.g. ts-cat-a-01)
    #[arg(long)]
    pub task: Option<String>,

    /// Run all tasks in a specific category (e.g. signature-refactoring)
    #[arg(long)]
    pub category: Option<String>,

    /// Run all tasks for a specific language (e.g. typescript)
    #[arg(long)]
    pub language: Option<String>,

    /// Number of repetitions per task per condition (default: 5)
    #[arg(long, default_value = "5")]
    pub reps: u32,

    /// Run both with-Scope and without-Scope conditions and compare results
    #[arg(long)]
    pub compare: bool,

    /// Run only the baseline condition (no Scope)
    #[arg(long, conflicts_with = "scope_only")]
    pub no_scope: bool,

    /// Run only the Scope-enabled condition
    #[arg(long, conflicts_with = "no_scope")]
    pub scope_only: bool,

    /// Output format for results
    #[arg(long, value_enum, default_value = "json")]
    pub output: OutputFormat,
}

#[derive(Parser, Debug)]
pub struct ReportArgs {
    /// Path to a JSON results file or directory containing results
    #[arg(long)]
    pub input: String,

    /// Output format for the report
    #[arg(long, value_enum, default_value = "markdown")]
    pub output: OutputFormat,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run(args) => run_benchmarks(&args),
        Commands::Report(args) => generate_report(&args),
    }
}

/// Execute benchmark tasks according to the provided arguments.
fn run_benchmarks(args: &RunArgs) -> Result<()> {
    // Determine tasks directory relative to the runner binary
    let tasks_dir = find_tasks_dir()?;
    let all_tasks = task::load_tasks(&tasks_dir)?;

    if all_tasks.is_empty() {
        anyhow::bail!(
            "No task files found in {}. Expected TOML files in benchmarks/tasks/.",
            tasks_dir.display()
        );
    }

    // Filter tasks based on CLI arguments
    let tasks = filter_tasks(&all_tasks, args)?;

    if tasks.is_empty() {
        anyhow::bail!("No tasks matched the given filters.");
    }

    eprintln!(
        "Selected {} task(s), {} rep(s) each",
        tasks.len(),
        args.reps
    );

    // Determine which conditions to run
    let run_with_scope = !args.no_scope;
    let run_without_scope = args.compare || args.no_scope;

    let mut all_runs: Vec<reporter::BenchmarkRun> = Vec::new();

    for task_def in &tasks {
        let corpus_path = resolve_corpus_path(task_def)?;

        // Back up the scope index before starting
        let backup = if run_with_scope {
            Some(git::backup_scope_index(&corpus_path)?)
        } else {
            None
        };

        for rep in 1..=args.reps {
            if run_with_scope {
                eprintln!(
                    "  [{}/{}] {} (with Scope, rep {})",
                    tasks
                        .iter()
                        .position(|t| t.task.id == task_def.task.id)
                        .unwrap_or(0)
                        + 1,
                    tasks.len(),
                    task_def.task.id,
                    rep
                );

                let agent_run = agent::run_agent(
                    task_def,
                    true,
                    &corpus_path,
                    backup.as_deref(),
                )?;
                let verification = verifier::verify_task(&corpus_path, task_def)?;
                let bm = behavior::compute_behavior_metrics(&agent_run.actions);

                all_runs.push(reporter::BenchmarkRun {
                    task_id: task_def.task.id.clone(),
                    repetition: rep,
                    scope_enabled: true,
                    input_tokens: agent_run.input_tokens,
                    output_tokens: agent_run.output_tokens,
                    file_reads: agent_run.file_reads,
                    scope_commands_called: agent_run.scope_commands_called,
                    correctness: reporter::CorrectnessResult {
                        compilation_pass: verification.compilation_pass,
                        tests_pass: verification.tests_pass,
                        caller_coverage: verification.caller_coverage,
                        overall_score: verification.overall_score,
                    },
                    duration_ms: agent_run.duration_ms,
                    actions: agent_run.actions,
                    behavior: Some(bm),
                });

                // Reset corpus between runs
                git::reset_corpus(&corpus_path)?;
                if let Some(ref backup_path) = backup {
                    git::restore_scope_index(&corpus_path, backup_path)?;
                }
            }

            if run_without_scope {
                eprintln!(
                    "  [{}/{}] {} (without Scope, rep {})",
                    tasks
                        .iter()
                        .position(|t| t.task.id == task_def.task.id)
                        .unwrap_or(0)
                        + 1,
                    tasks.len(),
                    task_def.task.id,
                    rep
                );

                let agent_run = agent::run_agent(
                    task_def,
                    false,
                    &corpus_path,
                    backup.as_deref(),
                )?;
                let verification = verifier::verify_task(&corpus_path, task_def)?;
                let bm = behavior::compute_behavior_metrics(&agent_run.actions);

                all_runs.push(reporter::BenchmarkRun {
                    task_id: task_def.task.id.clone(),
                    repetition: rep,
                    scope_enabled: false,
                    input_tokens: agent_run.input_tokens,
                    output_tokens: agent_run.output_tokens,
                    file_reads: agent_run.file_reads,
                    scope_commands_called: agent_run.scope_commands_called,
                    correctness: reporter::CorrectnessResult {
                        compilation_pass: verification.compilation_pass,
                        tests_pass: verification.tests_pass,
                        caller_coverage: verification.caller_coverage,
                        overall_score: verification.overall_score,
                    },
                    duration_ms: agent_run.duration_ms,
                    actions: agent_run.actions,
                    behavior: Some(bm),
                });

                // Reset corpus between runs
                git::reset_corpus(&corpus_path)?;
                if let Some(ref backup_path) = backup {
                    git::restore_scope_index(&corpus_path, backup_path)?;
                }
            }
        }
    }

    // Write results
    let results_dir = find_results_dir()?;
    std::fs::create_dir_all(&results_dir)?;

    match args.output {
        OutputFormat::Json => {
            let path = results_dir.join("full_results.json");
            reporter::write_json_results(&all_runs, &path)?;
            eprintln!("Results written to {}", path.display());
        }
        OutputFormat::Markdown => {
            let path = results_dir.join("summary.md");
            reporter::write_markdown_summary(&all_runs, &path)?;
            eprintln!("Summary written to {}", path.display());
        }
    }

    let env_path = results_dir.join("environment.json");
    reporter::write_environment(&env_path)?;

    Ok(())
}

/// Generate a report from previously saved results.
fn generate_report(args: &ReportArgs) -> Result<()> {
    let input_path = std::path::Path::new(&args.input);

    let results_path = if input_path.is_dir() {
        input_path.join("full_results.json")
    } else {
        input_path.to_path_buf()
    };

    let content = std::fs::read_to_string(&results_path).map_err(|e| {
        anyhow::anyhow!(
            "Failed to read results from {}: {}",
            results_path.display(),
            e
        )
    })?;

    let wrapper: reporter::ResultsWrapper = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse results JSON: {}", e))?;

    match args.output {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&wrapper)?);
        }
        OutputFormat::Markdown => {
            let output_path = if input_path.is_dir() {
                input_path.join("summary.md")
            } else {
                input_path.with_extension("md")
            };
            reporter::write_markdown_summary(&wrapper.runs, &output_path)?;
            eprintln!("Summary written to {}", output_path.display());
        }
    }

    Ok(())
}

/// Filter tasks based on CLI arguments.
fn filter_tasks<'a>(tasks: &'a [task::TaskDef], args: &RunArgs) -> Result<Vec<&'a task::TaskDef>> {
    if args.all {
        return Ok(tasks.iter().collect());
    }

    if let Some(ref task_id) = args.task {
        let matched: Vec<&task::TaskDef> = tasks.iter().filter(|t| t.task.id == *task_id).collect();
        if matched.is_empty() {
            anyhow::bail!(
                "Task '{}' not found. Available tasks: {}",
                task_id,
                tasks
                    .iter()
                    .map(|t| t.task.id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        return Ok(matched);
    }

    if let Some(ref category) = args.category {
        let matched: Vec<&task::TaskDef> = tasks
            .iter()
            .filter(|t| t.task.category == *category)
            .collect();
        if matched.is_empty() {
            anyhow::bail!("No tasks found for category '{}'", category);
        }
        return Ok(matched);
    }

    if let Some(ref language) = args.language {
        let matched: Vec<&task::TaskDef> = tasks
            .iter()
            .filter(|t| t.task.language == *language)
            .collect();
        if matched.is_empty() {
            anyhow::bail!("No tasks found for language '{}'", language);
        }
        return Ok(matched);
    }

    anyhow::bail!(
        "Specify --all, --task <id>, --category <name>, or --language <lang> to select tasks."
    );
}

/// Find the tasks directory relative to the benchmark runner.
fn find_tasks_dir() -> Result<std::path::PathBuf> {
    // Try relative to current directory first
    let candidates = [
        std::path::PathBuf::from("benchmarks/tasks"),
        std::path::PathBuf::from("../tasks"),
        std::path::PathBuf::from("../../benchmarks/tasks"),
    ];

    for candidate in &candidates {
        if candidate.is_dir() {
            return Ok(candidate.clone());
        }
    }

    anyhow::bail!(
        "Could not find benchmarks/tasks/ directory. \
         Run this command from the project root or benchmarks/runner/."
    );
}

/// Find the results output directory.
fn find_results_dir() -> Result<std::path::PathBuf> {
    let candidates = [
        std::path::PathBuf::from("benchmarks/results/latest"),
        std::path::PathBuf::from("../results/latest"),
        std::path::PathBuf::from("../../benchmarks/results/latest"),
    ];

    for candidate in &candidates {
        // Use the first candidate whose parent exists
        if let Some(parent) = candidate.parent() {
            if parent.is_dir() || parent == std::path::Path::new("") {
                return Ok(candidate.clone());
            }
        }
    }

    // Default
    Ok(std::path::PathBuf::from("benchmarks/results/latest"))
}

/// Resolve the corpus path for a given task.
fn resolve_corpus_path(task_def: &task::TaskDef) -> Result<std::path::PathBuf> {
    let corpus_name = &task_def.task.corpus;

    let fixture_candidates = match corpus_name.as_str() {
        "fixture" => vec![
            format!("benchmarks/fixtures/{}-large", task_def.task.language),
            format!("../fixtures/{}-large", task_def.task.language),
            format!("../../benchmarks/fixtures/{}-large", task_def.task.language),
            format!("benchmarks/fixtures/{}-api", task_def.task.language),
            format!("../fixtures/{}-api", task_def.task.language),
            format!("../../benchmarks/fixtures/{}-api", task_def.task.language),
        ],
        other => vec![
            format!("benchmarks/corpora/{}", other),
            format!("../corpora/{}", other),
            format!("../../benchmarks/corpora/{}", other),
        ],
    };

    for candidate in &fixture_candidates {
        let path = std::path::PathBuf::from(candidate);
        if path.is_dir() {
            return Ok(path);
        }
    }

    anyhow::bail!(
        "Corpus '{}' for task '{}' not found. Expected directory at benchmarks/fixtures/{}-api/ or benchmarks/corpora/{}.",
        corpus_name, task_def.task.id, task_def.task.language, corpus_name
    );
}
