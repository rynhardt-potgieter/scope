mod agent;
mod behavior;
mod git;
mod manifest;
mod reporter;
mod task;
mod verifier;

use anyhow::{Context, Result};
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
    ///
    /// Requires ANTHROPIC_API_KEY and the `claude` CLI installed.
    Run(RunArgs),

    /// Prepare isolated work directories and print prompts for manual runs.
    ///
    /// Sets up temp directories with CLAUDE.md variants and .scope/ indexes,
    /// then prints the exact prompts to use. Run each prompt manually in a
    /// Claude Code session, then use `benchmark import` to ingest results.
    /// Does NOT require an API key.
    Prepare(PrepareArgs),

    /// Import manually captured benchmark results for analysis.
    ///
    /// Reads a JSON file with agent run data (tokens, tool calls, actions)
    /// and generates behavior analysis reports. Use this when benchmarks
    /// are run manually (e.g., via Claude Code Agent tool) instead of via
    /// `benchmark run`.
    Import(ImportArgs),

    /// Generate a report from existing benchmark results.
    ///
    /// Reads a previously saved JSON results file and produces a summary
    /// in the requested format (JSON or Markdown).
    Report(ReportArgs),

    /// Generate or verify fixture integrity manifests.
    ///
    /// Use --generate to create a new manifest from the current fixture state.
    /// Use --verify to check that fixtures match their stored manifests.
    /// Manifests protect fixtures from accidental corruption between runs.
    Manifest(ManifestArgs),

    /// Verify correctness of a completed benchmark work directory.
    ///
    /// Runs compilation, test, and task-specific checks on a directory
    /// where an agent has already completed its work. Use this after
    /// manual benchmark runs to compute correctness scores.
    Verify(VerifyArgs),

    /// Run a single-task validation test across all 3 conditions.
    ///
    /// Runs one task with 1 rep across without-scope, with-scope, and
    /// with-scope-preloaded conditions. Validates that telemetry data is
    /// captured correctly (tokens, actions, file reads, scope commands)
    /// before committing to a full benchmark run.
    ///
    /// Examples:
    ///   benchmark test --task ts-cat-a-01 --model sonnet
    ///   benchmark test --language typescript --model opus
    Test(TestArgs),
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

    /// Number of repetitions per task per condition (default: 3)
    #[arg(long, default_value = "3")]
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

    /// Model to use for agent runs (e.g. "sonnet", "opus", "haiku").
    /// Passed directly to the claude CLI --model flag.
    #[arg(long)]
    pub model: Option<String>,

    /// Number of experimental conditions: 1 = with-scope only (default),
    /// 2 = with --compare, 3 = without + with + preloaded,
    /// 4 = all 3 + with-mcp (scope via MCP tools instead of Bash)
    #[arg(long, default_value = "1")]
    pub conditions: u32,

    /// Directory to save raw NDJSON streams from each run
    #[arg(long)]
    pub save_ndjson: Option<String>,

    /// Output directory for results (default: benchmarks/results/latest/).
    /// Results include full_results.json, summary.md, and environment.json.
    #[arg(long)]
    pub output_dir: Option<String>,

    /// Number of parallel agent runs (default: 1 = sequential).
    /// Each parallel run uses its own temp directory and claude process.
    /// Increase to speed up benchmarks if your API rate limits allow it.
    #[arg(long, default_value = "1")]
    pub parallel: usize,

    /// Resume a previous run by skipping already-completed task/condition/rep
    /// combinations found in the output directory's full_results.json.
    #[arg(long)]
    pub resume: bool,
}

#[derive(Parser, Debug)]
pub struct PrepareArgs {
    /// Run all tasks in the task suite
    #[arg(long)]
    pub all: bool,

    /// Prepare a single task by its ID
    #[arg(long)]
    pub task: Option<String>,

    /// Prepare all tasks for a specific language
    #[arg(long)]
    pub language: Option<String>,

    /// Also prepare without-scope variant for comparison
    #[arg(long)]
    pub compare: bool,

    /// Number of experimental conditions (default: 1 = with-scope only,
    /// 2 = with --compare, 3 = without-scope + with-scope + with-scope-preloaded)
    #[arg(long, default_value = "1")]
    pub conditions: u32,

    /// Output directory for prepared work dirs (default: benchmarks/prepared/)
    #[arg(long)]
    pub output_dir: Option<String>,
}

#[derive(Parser, Debug)]
pub struct ImportArgs {
    /// Path to a JSON file with manually captured results
    #[arg(long)]
    pub input: String,

    /// Directory containing raw NDJSON files from Claude CLI sessions.
    /// Files should be named: <task_id>-<condition>.ndjson
    /// (e.g. "ts-cat-a-01-with-scope.ndjson")
    #[arg(long)]
    pub ndjson_dir: Option<String>,

    /// Output format for the analysis report
    #[arg(long, value_enum, default_value = "markdown")]
    pub output: OutputFormat,

    /// Output directory for results
    #[arg(long)]
    pub output_dir: Option<String>,
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

#[derive(Parser, Debug)]
pub struct ManifestArgs {
    /// Generate a new manifest from the current fixture state
    #[arg(long, conflicts_with = "verify")]
    pub generate: bool,

    /// Verify fixtures against their stored manifests
    #[arg(long, conflicts_with = "generate")]
    pub verify: bool,

    /// Path to a specific fixture directory (default: auto-detect all fixtures)
    #[arg(long)]
    pub fixture: Option<String>,
}

#[derive(Parser, Debug)]
pub struct VerifyArgs {
    /// Path to a completed work directory to verify
    #[arg(long)]
    pub dir: String,

    /// Task ID to verify against (auto-detected from directory name if omitted)
    #[arg(long)]
    pub task: Option<String>,

    /// Output as JSON instead of human-readable
    #[arg(long)]
    pub json: bool,
}

#[derive(Parser, Debug)]
pub struct TestArgs {
    /// Task ID to test (e.g. ts-cat-a-01). If omitted, picks the first task
    /// matching --language, or ts-cat-a-01 as default.
    #[arg(long)]
    pub task: Option<String>,

    /// Language to pick a task from if --task not specified
    #[arg(long)]
    pub language: Option<String>,

    /// Model to use (e.g. "sonnet", "opus", "haiku"). Required.
    #[arg(long)]
    pub model: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run(args) => run_benchmarks(&args),
        Commands::Prepare(args) => prepare_benchmarks(&args),
        Commands::Import(args) => import_results(&args),
        Commands::Report(args) => generate_report(&args),
        Commands::Manifest(args) => manage_manifests(&args),
        Commands::Verify(args) => verify_work_dir(&args),
        Commands::Test(args) => run_test(&args),
    }
}

/// A fully-specified benchmark job ready for execution.
///
/// Built upfront so that both sequential and parallel paths iterate
/// over the same job list. Each job captures everything needed to
/// invoke `agent::run_agent` without re-borrowing mutable state.
struct RunJob {
    task_def_idx: usize,
    rep: u32,
    condition_label: String,
    scope_enabled: bool,
    corpus_path: std::path::PathBuf,
    backup_path: Option<std::path::PathBuf>,
    ndjson_path: Option<std::path::PathBuf>,
}

/// Format seconds as human-readable duration (e.g. "1h 23m 45s").
fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m {}s", secs / 3600, (secs % 3600) / 60, secs % 60)
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

    // Determine which conditions to run (same tuple approach as prepare_benchmarks)
    let conditions: Vec<(&str, bool)> = if args.conditions >= 4 {
        vec![
            ("without-scope", false),
            ("with-scope", true),
            ("with-scope-preloaded", true),
            ("with-mcp", true),
        ]
    } else if args.conditions >= 3 {
        vec![
            ("without-scope", false),
            ("with-scope", true),
            ("with-scope-preloaded", true),
        ]
    } else if args.compare || args.conditions == 2 {
        vec![("with-scope", true), ("without-scope", false)]
    } else if args.no_scope {
        vec![("without-scope", false)]
    } else {
        // Default and --scope-only both run with-scope only
        vec![("with-scope", true)]
    };

    // Determine NDJSON directory: explicit --save-ndjson, auto under --output-dir, or none
    let ndjson_dir: Option<std::path::PathBuf> = if args.save_ndjson.is_some() {
        args.save_ndjson.as_ref().map(std::path::PathBuf::from)
    } else if args.output_dir.is_some() {
        let rd = std::path::PathBuf::from(
            args.output_dir
                .as_ref()
                .map_or("benchmarks/results/latest", |s| s.as_str()),
        );
        Some(rd.join("ndjson"))
    } else {
        None
    };

    if let Some(ref dir) = ndjson_dir {
        std::fs::create_dir_all(dir)?;
    }

    // Set up results directory BEFORE the run loop so incremental saves work
    let results_dir = if let Some(ref dir) = args.output_dir {
        std::path::PathBuf::from(dir)
    } else {
        find_results_dir()?
    };
    std::fs::create_dir_all(&results_dir)?;
    let json_path = results_dir.join("full_results.json");

    // Build all jobs upfront
    let mut jobs: Vec<RunJob> = Vec::new();
    // Track backups per corpus so we only create one backup per fixture
    let mut corpus_backups: std::collections::HashMap<String, Option<std::path::PathBuf>> =
        std::collections::HashMap::new();

    for (task_idx, task_def) in tasks.iter().enumerate() {
        let corpus_path = resolve_corpus_path(task_def)?;
        let corpus_key = corpus_path.display().to_string();

        let backup = if let Some(existing) = corpus_backups.get(&corpus_key) {
            existing.clone()
        } else {
            let scope_dir = corpus_path.join(".scope");
            let b = if scope_dir.is_dir() {
                Some(git::backup_scope_index(&corpus_path)?)
            } else {
                None
            };
            corpus_backups.insert(corpus_key, b.clone());
            b
        };

        for rep in 1..=args.reps {
            for &(condition_label, scope_enabled) in &conditions {
                let ndjson_path = ndjson_dir.as_ref().map(|dir| {
                    dir.join(format!(
                        "{}-{}-rep{}.ndjson",
                        task_def.task.id, condition_label, rep
                    ))
                });

                jobs.push(RunJob {
                    task_def_idx: task_idx,
                    rep,
                    condition_label: condition_label.to_string(),
                    scope_enabled,
                    corpus_path: corpus_path.clone(),
                    backup_path: backup.clone(),
                    ndjson_path,
                });
            }
        }
    }

    // Resume: load existing results and skip completed jobs
    let mut resumed_runs: Vec<reporter::BenchmarkRun> = Vec::new();
    let mut completed_keys: std::collections::HashSet<String> = std::collections::HashSet::new();

    if args.resume && json_path.is_file() {
        let content = std::fs::read_to_string(&json_path).with_context(|| {
            format!(
                "Failed to read existing results for resume: {}",
                json_path.display()
            )
        })?;
        match serde_json::from_str::<reporter::ResultsWrapper>(&content) {
            Ok(wrapper) => {
                for run in &wrapper.runs {
                    let key = format!("{}|{}|{}", run.task_id, run.condition, run.repetition);
                    completed_keys.insert(key);
                }
                eprintln!(
                    "Resuming: found {} completed runs in {}",
                    wrapper.runs.len(),
                    json_path.display()
                );
                resumed_runs = wrapper.runs;
            }
            Err(e) => {
                eprintln!(
                    "  [resume] Warning: failed to parse existing results: {}",
                    e
                );
                eprintln!("  [resume] Starting from scratch.");
            }
        }
    }

    // Filter out already-completed jobs
    let original_count = jobs.len();
    if !completed_keys.is_empty() {
        jobs.retain(|job| {
            let key = format!(
                "{}|{}|{}",
                tasks[job.task_def_idx].task.id, job.condition_label, job.rep
            );
            !completed_keys.contains(&key)
        });
        eprintln!(
            "Skipping {} completed, {} remaining\n",
            original_count - jobs.len(),
            jobs.len()
        );
    }

    let total_runs = jobs.len();
    if total_runs == 0 {
        eprintln!("All runs already completed. Nothing to do.");
        // Still write final markdown summary
        let md_path = results_dir.join("summary.md");
        reporter::write_markdown_summary(&resumed_runs, &md_path)?;
        let env_path = results_dir.join("environment.json");
        reporter::write_environment(&env_path)?;
        return Ok(());
    }

    let model_label = args.model.as_deref().unwrap_or("default");
    eprintln!();
    eprintln!("  scope benchmark");
    eprintln!("  ─────────────────────────────────────────────────────────");
    eprintln!("  model:      {}", model_label);
    eprintln!(
        "  runs:       {} total ({} tasks × {} conditions × {} reps)",
        total_runs,
        tasks.len(),
        conditions.len(),
        args.reps
    );
    eprintln!("  parallel:   {}", args.parallel);
    eprintln!("  output:     {}", results_dir.display());
    eprintln!("  ─────────────────────────────────────────────────────────");
    eprintln!();
    let benchmark_start = std::time::Instant::now();

    let all_runs = if args.parallel <= 1 {
        // Sequential execution with incremental save
        let mut runs: Vec<reporter::BenchmarkRun> = resumed_runs;

        for (i, job) in jobs.iter().enumerate() {
            let task_def = tasks[job.task_def_idx];

            // Overall progress line
            let elapsed = benchmark_start.elapsed().as_secs();
            let eta = if i > 0 {
                let avg_per_run = elapsed as f64 / i as f64;
                let remaining = avg_per_run * (total_runs - i) as f64;
                format_duration(remaining as u64)
            } else {
                "calculating".to_string()
            };
            let pct = (i as f64 / total_runs as f64 * 100.0) as u32;
            let bar_width = 20;
            let filled = (bar_width as f64 * i as f64 / total_runs as f64) as usize;
            let bar: String = "━".repeat(filled) + &"╌".repeat(bar_width - filled);

            let started_at = chrono::Local::now().format("%H:%M:%S").to_string();
            eprintln!(
                "  [{}/{}] {} {} │ {} │ rep {}",
                i + 1,
                total_runs,
                task_def.task.id,
                job.condition_label,
                task_def.task.category,
                job.rep
            );
            eprintln!(
                "         {bar} {pct}%  {elapsed}  eta {eta}  started {started_at}",
                elapsed = format_duration(elapsed),
            );

            // Agent.rs emits live status updates via \r on stderr during the run

            // Pass model through the job by calling run_agent directly
            let (agent_run, work_dir) = agent::run_agent(
                task_def,
                job.scope_enabled,
                &job.condition_label,
                &job.corpus_path,
                job.backup_path.as_deref(),
                args.model.as_deref(),
                job.ndjson_path.as_deref(),
            )?;

            let verification = verifier::verify_task(work_dir.path(), task_def)?;
            let bm = behavior::compute_behavior_metrics(&agent_run.actions);
            let agent_run_duration_ms = agent_run.duration_ms;
            let run_output_tokens = agent_run.output_tokens;
            let run_file_reads = agent_run.file_reads;
            let run_action_count = agent_run.actions.len();

            runs.push(reporter::BenchmarkRun {
                task_id: task_def.task.id.clone(),
                repetition: job.rep,
                scope_enabled: job.scope_enabled,
                condition: job.condition_label.clone(),
                cache_creation_input_tokens: agent_run.cache_creation_input_tokens,
                cache_read_input_tokens: agent_run.cache_read_input_tokens,
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

            // Incremental save
            reporter::write_json_results(&runs, &json_path)?;

            let comp_str = if verification.compilation_pass {
                "✓ pass"
            } else {
                "✗ FAIL"
            };
            eprintln!(
                "         done in {}s  │  {} tokens out  │  {} reads  │  {} actions  │  {}",
                agent_run_duration_ms / 1000,
                run_output_tokens,
                run_file_reads,
                run_action_count,
                comp_str,
            );
            eprintln!();

            // work_dir (TempDir) is dropped here, cleaning up
        }

        runs
    } else {
        // Parallel execution using std::thread::scope
        let runs_mutex = std::sync::Mutex::new(resumed_runs);
        let completed = std::sync::atomic::AtomicUsize::new(0);
        let failed = std::sync::atomic::AtomicUsize::new(0);
        let model = args.model.as_deref();

        for chunk in jobs.chunks(args.parallel) {
            std::thread::scope(|s| {
                let handles: Vec<_> = chunk
                    .iter()
                    .map(|job| {
                        let task_def = tasks[job.task_def_idx];
                        s.spawn(move || -> Result<reporter::BenchmarkRun> {
                            eprintln!(
                                "  [parallel] Starting {} ({}, rep {})",
                                task_def.task.id, job.condition_label, job.rep
                            );

                            let (agent_run, work_dir) = agent::run_agent(
                                task_def,
                                job.scope_enabled,
                                &job.condition_label,
                                &job.corpus_path,
                                job.backup_path.as_deref(),
                                model,
                                job.ndjson_path.as_deref(),
                            )?;

                            let verification = verifier::verify_task(work_dir.path(), task_def)?;
                            let bm = behavior::compute_behavior_metrics(&agent_run.actions);

                            Ok(reporter::BenchmarkRun {
                                task_id: task_def.task.id.clone(),
                                repetition: job.rep,
                                scope_enabled: job.scope_enabled,
                                condition: job.condition_label.clone(),
                                cache_creation_input_tokens: agent_run.cache_creation_input_tokens,
                                cache_read_input_tokens: agent_run.cache_read_input_tokens,
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
                            })
                        })
                    })
                    .collect();

                // Collect results from this batch
                for handle in handles {
                    match handle.join() {
                        Ok(Ok(run)) => {
                            let mut runs = runs_mutex.lock().unwrap_or_else(|e| e.into_inner());
                            runs.push(run);
                            drop(runs); // Release lock before file I/O

                            let c = completed.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                            eprintln!("  [parallel] Completed {}/{}", c, total_runs);

                            // Incremental save
                            let runs = runs_mutex.lock().unwrap_or_else(|e| e.into_inner());
                            if let Err(e) = reporter::write_json_results(&runs, &json_path) {
                                eprintln!("  Warning: Failed to save incremental results: {}", e);
                            }
                        }
                        Ok(Err(e)) => {
                            failed.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                            eprintln!("  [parallel] Run failed: {}", e);
                        }
                        Err(_) => {
                            failed.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                            eprintln!("  [parallel] Thread panicked");
                        }
                    }
                }
            });
        }

        let fail_count = failed.load(std::sync::atomic::Ordering::SeqCst);
        if fail_count > 0 {
            eprintln!(
                "  Warning: {} of {} runs failed or panicked",
                fail_count, total_runs
            );
        }

        runs_mutex.into_inner().unwrap_or_else(|e| e.into_inner())
    };

    // Final summary
    let total_elapsed = benchmark_start.elapsed().as_secs();
    let total_output: u64 = all_runs.iter().map(|r| r.output_tokens).sum();
    let _total_cache_create: u64 = all_runs.iter().map(|r| r.cache_creation_input_tokens).sum();
    let _total_cache_read: u64 = all_runs.iter().map(|r| r.cache_read_input_tokens).sum();

    let comp_pass = all_runs
        .iter()
        .filter(|r| r.correctness.compilation_pass)
        .count();
    let est_cost = all_runs
        .iter()
        .map(|r| {
            (r.cache_creation_input_tokens as f64 * 3.75
                + r.cache_read_input_tokens as f64 * 0.375
                + r.output_tokens as f64 * 15.0
                + r.input_tokens as f64 * 3.0)
                / 1_000_000.0
        })
        .sum::<f64>();

    eprintln!();
    eprintln!("  benchmark complete");
    eprintln!("  ─────────────────────────────────────────────────────────");
    eprintln!("  runs:         {}/{}", all_runs.len(), total_runs);
    eprintln!("  duration:     {}", format_duration(total_elapsed));
    eprintln!(
        "  compilation:  {}/{} pass ({:.0}%)",
        comp_pass,
        all_runs.len(),
        comp_pass as f64 / all_runs.len() as f64 * 100.0,
    );
    eprintln!("  output tkns:  {}", total_output);
    eprintln!("  est. cost:    ${:.2}", est_cost);
    eprintln!("  ─────────────────────────────────────────────────────────");

    // Final JSON save (ensures we have the complete set even for parallel path)
    reporter::write_json_results(&all_runs, &json_path)?;
    eprintln!("\n  📄 {}", json_path.display());

    // Write markdown summary and environment (only at the end)
    let md_path = results_dir.join("summary.md");
    reporter::write_markdown_summary(&all_runs, &md_path)?;
    eprintln!("  📊 {}", md_path.display());

    let env_path = results_dir.join("environment.json");
    reporter::write_environment(&env_path)?;
    eprintln!("  🖥  {}", env_path.display());

    Ok(())
}

/// Prepare isolated work directories for manual benchmark runs.
///
/// Creates temp directories with the right CLAUDE.md variant and .scope/ index,
/// then prints the exact prompt for each task+condition so the user can run
/// them manually in Claude Code sessions.
fn prepare_benchmarks(args: &PrepareArgs) -> Result<()> {
    let tasks_dir = find_tasks_dir()?;
    let all_tasks = task::load_tasks(&tasks_dir)?;

    let tasks = if args.all {
        all_tasks.iter().collect::<Vec<_>>()
    } else if let Some(ref task_id) = args.task {
        all_tasks.iter().filter(|t| t.task.id == *task_id).collect()
    } else if let Some(ref lang) = args.language {
        all_tasks
            .iter()
            .filter(|t| t.task.language == *lang)
            .collect()
    } else {
        anyhow::bail!("Specify --all, --task <id>, or --language <lang>.");
    };

    if tasks.is_empty() {
        anyhow::bail!("No tasks matched the given filters.");
    }

    // Verify fixture integrity before preparing
    let mut verified_fixtures: std::collections::HashSet<String> = std::collections::HashSet::new();

    let output_base = args.output_dir.as_deref().unwrap_or("benchmarks/prepared");
    std::fs::create_dir_all(output_base)?;

    let ndjson_dir = std::path::PathBuf::from(output_base).join("ndjson");
    std::fs::create_dir_all(&ndjson_dir)?;

    let conditions: Vec<(&str, bool)> = if args.conditions >= 4 {
        vec![
            ("without-scope", false),
            ("with-scope", true),
            ("with-scope-preloaded", true),
            ("with-mcp", true),
        ]
    } else if args.conditions >= 3 {
        vec![
            ("without-scope", false),
            ("with-scope", true),
            ("with-scope-preloaded", true),
        ]
    } else if args.compare || args.conditions == 2 {
        vec![("with-scope", true), ("without-scope", false)]
    } else {
        vec![("with-scope", true)]
    };

    let mut manifest = Vec::new();

    for task_def in &tasks {
        let corpus_path = resolve_corpus_path(task_def)?;

        // Verify fixture integrity (once per fixture)
        let corpus_key = corpus_path.display().to_string();
        if !verified_fixtures.contains(&corpus_key) {
            let manifest_file = corpus_path.join(".fixture-manifest.sha256");
            if manifest_file.is_file() {
                manifest::verify_manifest(&corpus_path)?;
            } else {
                eprintln!(
                    "  Warning: No manifest found for {}. Skipping integrity check.",
                    corpus_path.display()
                );
            }
            verified_fixtures.insert(corpus_key);
        }

        for &(condition_label, scope_enabled) in &conditions {
            let dir_name = format!("{}-{}", task_def.task.id, condition_label);
            let work_dir = std::path::PathBuf::from(output_base).join(&dir_name);

            // Clean and create
            if work_dir.exists() {
                std::fs::remove_dir_all(&work_dir)?;
            }
            std::fs::create_dir_all(&work_dir)?;

            // Copy fixture
            agent::copy_dir_for_prepare(&corpus_path, &work_dir)?;

            // Install CLAUDE.md variant
            let variant = if scope_enabled {
                "CLAUDE.md.with-scope"
            } else {
                "CLAUDE.md.without-scope"
            };
            let variant_src = work_dir.join(variant);
            if variant_src.is_file() {
                std::fs::copy(&variant_src, work_dir.join("CLAUDE.md"))?;
            }

            // Handle .scope/ directory
            if !scope_enabled {
                let scope_dir = work_dir.join(".scope");
                if scope_dir.exists() {
                    std::fs::remove_dir_all(&scope_dir)?;
                }
            }

            // Handle pre-loaded scope map variant
            if condition_label == "with-scope-preloaded" {
                let preloaded_variant = work_dir.join("CLAUDE.md.with-scope-preloaded");
                if preloaded_variant.is_file() {
                    // Try to run scope map on the work directory
                    let map_output = std::process::Command::new("scope")
                        .args(["map"])
                        .current_dir(&work_dir)
                        .output();

                    match map_output {
                        Ok(output) if output.status.success() => {
                            let map_text = String::from_utf8_lossy(&output.stdout);
                            let template = std::fs::read_to_string(&preloaded_variant)?;
                            let rendered = template.replace("{{SCOPE_MAP_OUTPUT}}", &map_text);
                            std::fs::write(work_dir.join("CLAUDE.md"), rendered)?;
                        }
                        _ => {
                            eprintln!(
                                "  Warning: scope map failed for {}. Falling back to with-scope variant.",
                                dir_name
                            );
                            let fallback = work_dir.join("CLAUDE.md.with-scope");
                            if fallback.is_file() {
                                std::fs::copy(&fallback, work_dir.join("CLAUDE.md"))?;
                            }
                        }
                    }
                } else {
                    // No preloaded template — fall back to with-scope
                    let fallback = work_dir.join("CLAUDE.md.with-scope");
                    if fallback.is_file() {
                        std::fs::copy(&fallback, work_dir.join("CLAUDE.md"))?;
                    }
                }
            }

            let entry = serde_json::json!({
                "task_id": task_def.task.id,
                "category": task_def.task.category,
                "language": task_def.task.language,
                "scope_enabled": scope_enabled,
                "condition": condition_label,
                "work_dir": work_dir.display().to_string(),
                "prompt": task_def.prompt.text,
            });
            manifest.push(entry);

            eprintln!("  Prepared: {}", dir_name);
        }
    }

    // Write manifest
    let manifest_path = std::path::PathBuf::from(output_base).join("manifest.json");
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    std::fs::write(&manifest_path, &manifest_json)?;

    eprintln!(
        "\nPrepared {} work directories. Manifest: {}",
        manifest.len(),
        manifest_path.display()
    );
    eprintln!("\nTo run manually, for each entry in manifest.json:");
    eprintln!("  1. cd into the work_dir");
    eprintln!("  2. Run the prompt as a Claude Code agent");
    eprintln!("  3. Save the raw NDJSON output: claude -p \"<prompt>\" --output-format stream-json > {}/{{task_id}}-{{condition}}.ndjson", ndjson_dir.display());
    eprintln!("  4. Record: total_tokens, tool_uses, duration_ms from agent metadata");
    eprintln!(
        "  5. Use `benchmark import --input <results.json> --ndjson-dir {}` to ingest",
        ndjson_dir.display()
    );

    Ok(())
}

/// Import manually captured benchmark results and generate analysis.
///
/// Expects a JSON file with this schema:
/// ```json
/// [
///   {{
///     "task_id": "ts-cat-a-01",
///     "scope_enabled": true,
///     "input_tokens": 24635,
///     "output_tokens": 8000,
///     "duration_ms": 74981,
///     "tool_uses": 15,
///     "actions": [
///       {{"tool_name": "Bash", "arguments_summary": "scope find \"retry\"", ...}}
///     ]
///   }}
/// ]
/// ```
fn import_results(args: &ImportArgs) -> Result<()> {
    let content = std::fs::read_to_string(&args.input)
        .map_err(|e| anyhow::anyhow!("Failed to read import file {}: {}", args.input, e))?;

    // Try parsing as array of runs or as ResultsWrapper
    let runs: Vec<reporter::BenchmarkRun> =
        if let Ok(wrapper) = serde_json::from_str::<reporter::ResultsWrapper>(&content) {
            wrapper.runs
        } else if let Ok(runs) = serde_json::from_str::<Vec<reporter::BenchmarkRun>>(&content) {
            runs
        } else {
            anyhow::bail!(
                "Failed to parse import file. Expected a JSON array of BenchmarkRun objects \
                 or a ResultsWrapper object with a 'runs' field."
            );
        };

    if runs.is_empty() {
        anyhow::bail!("Import file contains no runs.");
    }

    // Recompute behavior metrics if missing
    let mut runs: Vec<reporter::BenchmarkRun> = runs
        .into_iter()
        .map(|mut run| {
            if run.behavior.is_none() && !run.actions.is_empty() {
                run.behavior = Some(behavior::compute_behavior_metrics(&run.actions));
            }
            run
        })
        .collect();

    // Populate actions from NDJSON files if provided
    if let Some(ref ndjson_dir) = args.ndjson_dir {
        let ndjson_path = std::path::Path::new(ndjson_dir);
        if !ndjson_path.is_dir() {
            anyhow::bail!("NDJSON directory does not exist: {}", ndjson_dir);
        }

        let mut parsed_count = 0u32;
        for run in &mut runs {
            if !run.actions.is_empty() {
                continue; // Already has actions, skip
            }

            let condition_label = if !run.condition.is_empty() {
                run.condition.clone()
            } else if run.scope_enabled {
                "with-scope".to_string()
            } else {
                "without-scope".to_string()
            };

            let ndjson_file =
                ndjson_path.join(format!("{}-{}.ndjson", run.task_id, condition_label));

            if ndjson_file.is_file() {
                let ndjson_text = std::fs::read_to_string(&ndjson_file).with_context(|| {
                    format!("Failed to read NDJSON file: {}", ndjson_file.display())
                })?;
                let parsed = agent::parse_ndjson_actions(&ndjson_text);

                run.actions = parsed.actions;
                run.scope_commands_called = parsed.scope_commands_called;
                if run.file_reads == 0 {
                    run.file_reads = parsed.file_reads;
                }

                // Recompute behavior metrics with the new actions
                if !run.actions.is_empty() {
                    run.behavior = Some(behavior::compute_behavior_metrics(&run.actions));
                }

                parsed_count += 1;
                eprintln!(
                    "  Parsed {} actions from NDJSON for {} ({})",
                    run.actions.len(),
                    run.task_id,
                    condition_label
                );
            }
        }

        if parsed_count > 0 {
            eprintln!("Enriched {} runs with NDJSON action data.", parsed_count);
        } else {
            eprintln!(
                "Warning: --ndjson-dir provided but no matching NDJSON files found for any runs."
            );
        }
    }

    eprintln!("Imported {} runs.", runs.len());

    // Warn about missing correctness data
    let missing_correctness = runs
        .iter()
        .filter(|r| r.correctness.overall_score == 0 && !r.correctness.compilation_pass)
        .count();
    if missing_correctness > 0 {
        eprintln!(
            "Warning: {} run(s) imported without correctness data. Run 'benchmark verify' on work dirs to compute correctness.",
            missing_correctness
        );
    }

    let output_dir = args
        .output_dir
        .as_deref()
        .unwrap_or("benchmarks/results/latest");
    let output_path = std::path::PathBuf::from(output_dir);
    std::fs::create_dir_all(&output_path)?;

    match args.output {
        OutputFormat::Json => {
            let path = output_path.join("full_results.json");
            reporter::write_json_results(&runs, &path)?;
            eprintln!("Results written to {}", path.display());
        }
        OutputFormat::Markdown => {
            let path = output_path.join("summary.md");
            reporter::write_markdown_summary(&runs, &path)?;
            eprintln!("Summary written to {}", path.display());
        }
    }

    let env_path = output_path.join("environment.json");
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

/// Run a single-task validation test across all 3 conditions.
///
/// Validates telemetry capture before committing to a full benchmark run.
/// Checks that actions, tokens, file reads, and scope commands are captured.
fn run_test(args: &TestArgs) -> Result<()> {
    let tasks_dir = find_tasks_dir()?;
    let all_tasks = task::load_tasks(&tasks_dir)?;

    // Select task
    let task_def = if let Some(ref task_id) = args.task {
        all_tasks
            .iter()
            .find(|t| t.task.id == *task_id)
            .ok_or_else(|| anyhow::anyhow!("Task '{}' not found.", task_id))?
    } else if let Some(ref lang) = args.language {
        all_tasks
            .iter()
            .find(|t| t.task.language == *lang)
            .ok_or_else(|| anyhow::anyhow!("No tasks found for language '{}'.", lang))?
    } else {
        all_tasks
            .iter()
            .find(|t| t.task.id == "ts-cat-a-01")
            .or_else(|| all_tasks.first())
            .ok_or_else(|| anyhow::anyhow!("No tasks found."))?
    };

    eprintln!("=== BENCHMARK TEST ===");
    eprintln!("Task: {} ({})", task_def.task.id, task_def.task.description);
    eprintln!("Model: {}", args.model);
    eprintln!("Conditions: without-scope, with-scope, with-scope-preloaded");
    eprintln!();

    let corpus_path = resolve_corpus_path(task_def)?;

    // Verify fixture integrity
    let manifest_file = corpus_path.join(".fixture-manifest.sha256");
    if manifest_file.is_file() {
        manifest::verify_manifest(&corpus_path)?;
        eprintln!("Fixture integrity: OK");
    }

    // Back up scope index
    let scope_dir = corpus_path.join(".scope");
    let backup = if scope_dir.is_dir() {
        Some(git::backup_scope_index(&corpus_path)?)
    } else {
        anyhow::bail!("No .scope/ directory found in fixture. Run 'scope index' first.");
    };

    // Create temp output dir for test
    let test_output = std::path::PathBuf::from("benchmarks/results/test");
    std::fs::create_dir_all(&test_output)?;
    let ndjson_dir = test_output.join("ndjson");
    std::fs::create_dir_all(&ndjson_dir)?;

    let conditions: Vec<(&str, bool)> = vec![
        ("without-scope", false),
        ("with-scope", true),
        ("with-scope-preloaded", true),
    ];

    let mut all_runs: Vec<reporter::BenchmarkRun> = Vec::new();
    let mut all_valid = true;

    for &(condition_label, scope_enabled) in &conditions {
        eprintln!("\n--- {} ---", condition_label);

        let ndjson_path =
            ndjson_dir.join(format!("{}-{}.ndjson", task_def.task.id, condition_label));

        let (agent_run, work_dir) = agent::run_agent(
            task_def,
            scope_enabled,
            condition_label,
            &corpus_path,
            backup.as_deref(),
            Some(&args.model),
            Some(&ndjson_path),
        )?;

        let verification = verifier::verify_task(work_dir.path(), task_def)?;
        let bm = behavior::compute_behavior_metrics(&agent_run.actions);

        // Validate telemetry
        let mut issues: Vec<String> = Vec::new();

        if agent_run.input_tokens == 0 {
            issues.push("input_tokens = 0".to_string());
        }
        if agent_run.output_tokens == 0 {
            issues.push("output_tokens = 0".to_string());
        }
        if agent_run.actions.is_empty() {
            issues.push("actions[] is empty".to_string());
        }
        if agent_run.file_reads == 0 {
            issues.push("file_reads = 0".to_string());
        }
        if scope_enabled && agent_run.scope_commands_called.is_empty() {
            issues
                .push("scope_commands_called is empty (agent may not have used scope)".to_string());
        }
        if !ndjson_path.is_file() {
            issues.push("NDJSON file was not saved".to_string());
        } else {
            let ndjson_size = std::fs::metadata(&ndjson_path)
                .map(|m| m.len())
                .unwrap_or(0);
            if ndjson_size == 0 {
                issues.push("NDJSON file is empty".to_string());
            } else {
                eprintln!(
                    "  NDJSON saved: {} ({} bytes)",
                    ndjson_path.display(),
                    ndjson_size
                );
            }
        }

        eprintln!("  Input tokens:    {}", agent_run.input_tokens);
        eprintln!("  Output tokens:   {}", agent_run.output_tokens);
        eprintln!("  Cache read:      {}", agent_run.cache_read_input_tokens);
        eprintln!(
            "  Cache creation:  {}",
            agent_run.cache_creation_input_tokens
        );
        eprintln!("  File reads:      {}", agent_run.file_reads);
        eprintln!("  Actions:         {}", agent_run.actions.len());
        eprintln!("  Scope commands:  {:?}", agent_run.scope_commands_called);
        eprintln!("  Duration:        {}ms", agent_run.duration_ms);
        eprintln!(
            "  Compilation:     {}",
            if verification.compilation_pass {
                "PASS"
            } else {
                "FAIL"
            }
        );

        if issues.is_empty() {
            eprintln!("  Telemetry:       VALID");
        } else {
            eprintln!("  Telemetry:       ISSUES DETECTED");
            for issue in &issues {
                eprintln!("    WARNING: {}", issue);
            }
            all_valid = false;
        }

        all_runs.push(reporter::BenchmarkRun {
            task_id: task_def.task.id.clone(),
            repetition: 1,
            scope_enabled,
            condition: condition_label.to_string(),
            cache_creation_input_tokens: agent_run.cache_creation_input_tokens,
            cache_read_input_tokens: agent_run.cache_read_input_tokens,
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
    }

    // Write test results
    let json_path = test_output.join("test_results.json");
    reporter::write_json_results(&all_runs, &json_path)?;
    let md_path = test_output.join("test_summary.md");
    reporter::write_markdown_summary(&all_runs, &md_path)?;

    eprintln!("\n=== TEST COMPLETE ===");
    eprintln!("Results: {}", test_output.display());

    if all_valid {
        eprintln!("Telemetry: ALL VALID - safe to proceed with full benchmark run");
    } else {
        eprintln!("Telemetry: ISSUES DETECTED - fix before running full benchmarks");
        eprintln!("Check the warnings above. Common causes:");
        eprintln!("  - ANTHROPIC_API_KEY not set or invalid");
        eprintln!("  - claude CLI not on PATH or wrong version");
        eprintln!("  - scope CLI not on PATH (needed for with-scope conditions)");
        // Return error instead of process::exit to follow project conventions
        anyhow::bail!("Telemetry validation failed. See warnings above.");
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

/// Generate or verify fixture integrity manifests.
fn manage_manifests(args: &ManifestArgs) -> Result<()> {
    if !args.generate && !args.verify {
        anyhow::bail!("Specify --generate or --verify.");
    }

    let fixtures = if let Some(ref fixture_path) = args.fixture {
        vec![std::path::PathBuf::from(fixture_path)]
    } else {
        find_all_fixture_dirs()?
    };

    if fixtures.is_empty() {
        anyhow::bail!("No fixture directories found.");
    }

    for fixture in &fixtures {
        if args.generate {
            manifest::generate_manifest(fixture)?;
        }
        if args.verify {
            manifest::verify_manifest(fixture)?;
        }
    }

    Ok(())
}

/// Verify correctness of a completed work directory.
fn verify_work_dir(args: &VerifyArgs) -> Result<()> {
    let work_dir = std::path::PathBuf::from(&args.dir);
    if !work_dir.is_dir() {
        anyhow::bail!("Work directory does not exist: {}", args.dir);
    }

    // Determine task ID from args or directory name
    let task_id = if let Some(ref id) = args.task {
        id.clone()
    } else {
        // Extract from dir name: e.g. "ts-cat-a-01-with-scope" -> "ts-cat-a-01"
        let dir_name = work_dir.file_name().and_then(|n| n.to_str()).unwrap_or("");
        extract_task_id_from_dir_name(dir_name).ok_or_else(|| {
            anyhow::anyhow!(
                "Could not extract task ID from directory name '{}'. Use --task to specify.",
                dir_name
            )
        })?
    };

    // Load the task definition
    let tasks_dir = find_tasks_dir()?;
    let all_tasks = task::load_tasks(&tasks_dir)?;
    let task_def = all_tasks
        .iter()
        .find(|t| t.task.id == task_id)
        .ok_or_else(|| anyhow::anyhow!("Task '{}' not found in task definitions.", task_id))?;

    // Run verification
    eprintln!("Verifying work directory for task '{}'...", task_id);
    let result = verifier::verify_task(&work_dir, task_def)?;

    if args.json {
        let output = serde_json::json!({
            "task_id": task_id,
            "compilation_pass": result.compilation_pass,
            "tests_pass": result.tests_pass,
            "caller_coverage": result.caller_coverage,
            "overall_score": result.overall_score,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        let pass = |b: bool| if b { "PASS" } else { "FAIL" };
        let coverage_str = match result.caller_coverage {
            Some(c) => format!("{:.0}%", c * 100.0),
            None => "N/A".to_string(),
        };
        println!("Verification: {}", task_id);
        println!("  Compilation: {}", pass(result.compilation_pass));
        println!("  Tests:       {}", pass(result.tests_pass));
        println!("  Coverage:    {}", coverage_str);
        println!("  Score:       {}", result.overall_score);
    }

    Ok(())
}

/// Extract task ID from a prepared directory name.
///
/// Directory names look like "ts-cat-a-01-with-scope", "cs-cat-b-01-without-scope",
/// or "ts-cat-a-01-with-scope-preloaded".
/// The task ID is everything before the condition suffix.
fn extract_task_id_from_dir_name(dir_name: &str) -> Option<String> {
    // Check longest suffix first to avoid partial match
    let suffixes = ["-with-scope-preloaded", "-without-scope", "-with-scope"];
    for suffix in &suffixes {
        if let Some(idx) = dir_name.find(suffix) {
            return Some(dir_name[..idx].to_string());
        }
    }
    None
}

/// Find all fixture directories.
fn find_all_fixture_dirs() -> Result<Vec<std::path::PathBuf>> {
    let candidates = [
        "benchmarks/fixtures",
        "../fixtures",
        "../../benchmarks/fixtures",
    ];

    for base in &candidates {
        let base_path = std::path::PathBuf::from(base);
        if base_path.is_dir() {
            let mut dirs = Vec::new();
            for entry in std::fs::read_dir(&base_path)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    // Only include fixture directories (contain "-large" or "-api")
                    if name.contains("-large") || name.contains("-api") {
                        dirs.push(entry.path());
                    }
                }
            }
            if !dirs.is_empty() {
                dirs.sort();
                return Ok(dirs);
            }
        }
    }

    anyhow::bail!("Could not find benchmarks/fixtures/ directory.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_task_id_with_scope() {
        assert_eq!(
            extract_task_id_from_dir_name("ts-cat-a-01-with-scope"),
            Some("ts-cat-a-01".to_string())
        );
    }

    #[test]
    fn test_extract_task_id_without_scope() {
        assert_eq!(
            extract_task_id_from_dir_name("cs-cat-b-01-without-scope"),
            Some("cs-cat-b-01".to_string())
        );
    }

    #[test]
    fn test_extract_task_id_unknown_format() {
        assert_eq!(extract_task_id_from_dir_name("random-dir-name"), None);
    }

    #[test]
    fn test_extract_task_id_with_scope_preloaded() {
        assert_eq!(
            extract_task_id_from_dir_name("ts-cat-a-01-with-scope-preloaded"),
            Some("ts-cat-a-01".to_string())
        );
    }
}
