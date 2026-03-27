# Changelog

## v0.9.0 (2026-03-27)

### New Features
- **`scope flow <start> <end>`** — find call paths between any two symbols. Unlike `scope trace` (entry points → target), this traces forward from start to end through the call graph. Supports `--depth`, `--limit`, `--json`.
- **Enum variant extraction** — new `variant` symbol kind. Extracts enum variants for TypeScript, C#, Java, and Rust with parent_id linking to parent enum. `scope sketch` now shows dedicated enum output with variant listing.
- **`this`/`self` edge capture** — TypeScript `this.method()`, C# `this.Method()`, Java `this.method()` calls now correctly captured in the edge graph. Previously missed because `this`/`self` are keyword nodes, not identifiers.
- **Generic name de-ranking** — per-language stopword lists prevent common names (`new`, `constructor`, `__init__`, `toString`) from dominating `scope find` results. Added `generic_name_stopwords()` to the LanguagePlugin trait.
- **Vendor code de-ranking** — `scope find` and `scope refs` partition results: first-party code appears before vendor code. Configurable `vendor_patterns` in config.toml, auto-configured per detected language by `scope init`.

### Improvements
- **Richer sketch output** — `sketch_enum()` handler shows enum variants with data shapes (e.g., `Success { tx_id: String }`). Class sketch JSON now includes `fields` array separate from methods.
- **Rust impl block association** — methods inside `impl` blocks now get `parent_id` linking to their target struct/enum. `scope sketch MyStruct` shows methods. `impl Trait for Type` produces `implements` edges shown in sketch output.
- **Match/switch arm edges** — Rust match arms, Java switch cases, and C# switch cases now produce `references` edges to enum variants. `scope refs SomeVariant` finds usage sites.
- **`base`/`super` edge capture** — C# `base.Method()` and Java `super.method()` calls now captured in the edge graph.
- **Sketch enrichments** — Java class sketch shows `@Override`, `@Deprecated` annotations on methods. Python sketch shows `@staticmethod`, `@property` decorators. Go sketch shows `(p *Type)` receiver on methods.
- **`get_incoming_callers` broad matching** — caller count in sketch output now matches the same broad pattern as `scope refs` (exact ID + bare name + member pattern). Previously under-reported callers.
- **FTS5 query context boost** — when a search query mixes specific and generic terms (e.g., `"payment new"`), specific terms are weighted higher so `PaymentService::new` ranks above `Logger::new`.

### Known Limitations
- **TypeScript enum variant refs** — `PaymentMethod.CreditCard` uses the same `member_expression` AST node as `request.amount`. Cannot distinguish enum access from property access without type information. Rust, Java, and C# enum refs work because their match/switch patterns are syntactically distinct.

---

## v0.8.0 (2026-03-26)

### New Features
- **Go language support** — fifth language plugin. Extracts functions, methods (with receiver metadata), structs, interfaces, constants, type aliases. Export inference from capitalization. Struct embedding produces extends edges. 15 integration tests.
- **Java language support** — sixth language plugin. Extracts classes, interfaces, enums, records, methods, constructors, fields, annotation types. Access modifiers (public/protected/private/package-private), static, final, abstract, synchronized, annotations (@Override, @Deprecated), throws clauses. 22 integration tests.

---

## v0.7.3 (2026-03-26)

### New Features
- **Distributable `code-navigation` skill** — drop-in skill at `skills/code-navigation/SKILL.md` that teaches agents (including subagents) to use Scope for code navigation. Includes decision trees, optimal workflows from 54 benchmarks, anti-patterns, and the 3-command rule. Named `code-navigation` so Claude Code's skill matching triggers on any navigation task.
- **Subagent scope awareness** — CLAUDE.md snippet now instructs the main session to point subagents at the skill file, so they read it and get the full scope workflow guidance.
- **Bash permission reference** — `skills/settings.json` with `Bash(scope:*)` permission for users to merge into their `.claude/settings.local.json`, enabling scope commands in subagents without permission prompts.

---

## v0.7.2 (2026-03-24)

### New Features
- **`scope workspace index --watch`** — watch all workspace members with a single command. Spawns one `scope index --watch` process per member, monitors child processes, and shuts down all watchers on Ctrl+C.

---

## v0.7.1 (2026-03-24)

### Fix
- **Python docstrings now extracted correctly** — moved `extract_docstring` into the `LanguagePlugin` trait with a default comment-based implementation. Python overrides with string-literal-based extraction. Previously all Python symbols had `docstring: None`.
- **Silent failure audit fixes** — 13 issues addressed: workspace query errors now logged instead of swallowed, watch loop survives transient failures, lock file handling hardened, read_dir errors logged in discovery and nested scan.

---

## v0.7.0 (2026-03-24)

### New Features
- **Python language support** — third language plugin. Extracts functions, classes, methods with decorator metadata (@staticmethod, @property, @abstractmethod), access inference from naming conventions (_private, __mangled), Python-specific docstring extraction from string nodes, async support, type annotations. 14 integration tests.
- **Rust language support** — fourth language plugin (dogfooding!). Extracts functions, structs, enums, traits, type aliases, consts, statics. Edge detection for use declarations, call expressions, method calls, macro invocations, type references. Visibility modifiers (pub, pub(crate), pub(super)). 16 integration tests.

---

## v0.6.0 (2026-03-24)

### New Features
- **`scope index --watch`** — auto re-index on file changes using `notify` crate. Debounces events (300ms), emits NDJSON with `--json` (`start`/`reindex`/`stop` events), lock file prevents concurrent watchers, graceful Ctrl+C shutdown with summary stats.
- **`scope workspace init`** — discover projects with `.scope/config.toml` in subdirectories and generate `scope-workspace.toml` manifest.
- **`scope workspace list`** — show all workspace members with index status, symbol counts, and freshness. Supports `--json`.
- **WorkspaceGraph facade** — federated query layer that opens N project databases and fans out `find_symbol`, `find_refs`, `get_entrypoints`, and aggregate stats across all members. Partial workspace support (warns and skips missing `graph.db`).
- **`--workspace` flag on 5 commands** — `scope status`, `scope map`, `scope refs`, `scope find`, and `scope entrypoints` now support `--workspace` to query across all workspace members. Results are tagged with project names and merged.
- **`--project <name>` flag** — target a specific workspace member by name from any directory.
- **Context resolution** — `resolve_project_root()` replaced with `resolve_context()` that detects single-project vs workspace context. Walks upward to find `scope-workspace.toml`.
- **Nested project detection** — file walker now skips subdirectories with `.scope/config.toml` to prevent double-indexing in workspace scenarios.
- **`scope workspace index`** — batch index all workspace members sequentially with per-member progress and summary stats. Supports `--full` and `--json`.

### Refactoring
- **LanguagePlugin trait** — extracted language-specific logic from `parser.rs` into a trait-based plugin system. TypeScript and C# each implement `LanguagePlugin` in their own modules. Adding a new language now requires only a new module + one registration line in `CodeParser::new()`. `parser.rs` shrank by 385 lines.

### Improvements
- **SQLite busy timeout** — `Graph::open()` now sets a 5-second busy timeout for concurrent read/write safety during `--watch` mode.
- **Documentation sync** — all docs (README, landing page, CLAUDE.md snippet) updated to v0.5.3. Snippet rewritten with `scope map` first, deprecated/stub commands removed.

---

## v0.5.3 (2026-03-23)

### Fix
- **Deterministic sort in `scope map` core symbols** — symbols with equal caller counts now sort by name as a tiebreaker. Fixes non-deterministic output order that caused snapshot test failures in CI.

---

## scope-benchmark v0.7.4 (2026-03-22)

### Improvements
- **Live agent progress** — during each run, a live-updating status line shows elapsed time, action count, file reads, output tokens, and the last tool called. Updates on every tool_use event so you can see the agent working in real time.
- **Clean output redesign** — header shows model/runs/conditions/reps in a readable layout, per-run lines show start time and overall progress bar, completion shows full metrics with compilation status.
- **Removed debug output** — no more `[agent] Command:` and `[agent] Work dir:` lines cluttering the output.

---

## scope-benchmark v0.7.3 (2026-03-22)

### Improvements
- **Upgraded benchmark output formatting** — box-drawing header with model/output/run count, per-run completion line showing duration/tokens/reads/actions/compilation, cleaner final summary with cost estimate. Removed debug command/workdir output.

---

## scope-benchmark v0.7.2 (2026-03-22)

### New Features
- **`--resume` flag on `benchmark run`** — reads existing `full_results.json` in the output directory, skips completed task/condition/rep combinations, and runs only what's missing. Safe to Ctrl+C and resume later.

### Fixes
- **Fix JSON round-trip for infinity values** — `navigation_to_edit_ratio` is infinity when an agent makes no edits. JSON serializes this as `null`, which failed deserialization on resume. Added custom deserializer that maps `null` back to `f64::INFINITY`.

---

## scope-benchmark v0.7.1 (2026-03-22)

### Improvements
- **Progress bar with ETA** — `benchmark run` shows progress bar, percentage, elapsed time, and estimated time remaining for sequential runs. Final summary shows total tokens and duration.

---

## scope-benchmark v0.7.0 (2026-03-22)

### New Features
- **`--parallel N` on `benchmark run`** — run N agent processes concurrently using scoped threads. Each run uses its own temp directory. Errors in individual threads are logged and skipped — the harness continues with remaining runs. Default: 1 (sequential).
- **Incremental save** — `full_results.json` is written after every completed run (both sequential and parallel). If the process crashes at run 50, you have 49 runs of data. Markdown summary and environment JSON are written once at the end.

---

## scope-benchmark v0.6.7 (2026-03-22)

### Fix
- **Fix scope_command_sequence empty for `cd && scope` pattern** — On Windows, agents prefix scope commands with `cd "tempdir" &&`. The `extract_scope_subcommand` function only matched commands starting with `scope `, missing the `cd` prefix. Now searches for `scope ` anywhere in the command string.

---

## scope-benchmark v0.6.6 (2026-03-22)

### Fix
- **Add `--verbose` flag for stream-json output** — Claude CLI now requires `--verbose` when using `--output-format stream-json` with `-p` (print mode). Without it, the CLI exits with an error and no NDJSON is produced.

---

## scope-benchmark v0.6.5 (2026-03-22)

### Fix
- **Fix Windows batch file argument limitation** — `.cmd` wrappers can't handle special characters in task prompts (quotes, newlines). Now bypasses the batch wrapper entirely by finding the claude CLI JS entry point and calling `node` directly via npm prefix resolution.

---

## scope-benchmark v0.6.4 (2026-03-22)

### Fix
- **Fix Windows CLI invocation breaking NDJSON stream** — `cmd /C claude` mangled arguments, causing claude CLI to run in text mode instead of stream-json. Changed to `claude.cmd` direct invocation. Same fix for `npx.cmd` and `npm.cmd` in verifier.

---

## scope-benchmark v0.6.3 (2026-03-22)

### Windows Compatibility
- **Fix all npm-installed CLI tools on Windows** — `npx`, `npm`, and `claude` are installed as `.cmd` scripts by npm. Rust's `Command::new()` can't find `.cmd` files. Added `cmd /C` wrapper for all npm-installed tools. Extracted `shell_command()` helper in verifier.rs.
- **Audited all Command::new calls** — `dotnet`, `git`, `scope` are real .exe binaries and work without wrapping.

---

## scope-benchmark v0.6.2 (2026-03-22)

### Critical Fix
- **`--bare` was suppressing CLAUDE.md loading** — the `--bare` flag on the claude CLI skips CLAUDE.md auto-discovery, meaning agents in ALL conditions received no project guidance. Added `--add-dir` flag to explicitly load the work directory's CLAUDE.md while keeping `--bare` for reproducibility (no hooks/LSP/plugins).

---

## scope-benchmark v0.6.1 (2026-03-22)

### New Commands
- **`benchmark test --task <id> --model <model>`** — single-task validation across all 3 conditions (1 rep). Validates telemetry capture (tokens, actions, file reads, scope commands, NDJSON) before committing to a full run. Prints clear PASS/FAIL per condition.

### Improvements
- **`--output-dir` on `benchmark run`** — specify where results are saved. Auto-creates `ndjson/` subdirectory for NDJSON streams.
- **Always writes both JSON and Markdown** — `full_results.json`, `summary.md`, and `environment.json` are always generated (removed the `--output` format selector)

---

## scope-benchmark v0.6.0 (2026-03-22)

### Critical Bug Fixes
- **Fix verification running on original fixture instead of agent's work** — `verifier::verify_task` now runs on the temp dir where the agent made changes. Previously it checked the unmodified original, so verification always passed against clean code.
- **Remove dangerous `git reset --hard HEAD` on fixture path** — `reset_corpus()` ran git reset on the fixture directory, which is inside the scope repo. Would have destroyed ALL uncommitted work in the repo. Removed entirely since agents work on temp copies.
- **Fix `--disallowedTools` syntax** — changed from incorrect space-separated args to `Bash(scope:*)` glob pattern that the claude CLI expects

### New Features
- **`--model <name>` flag on `benchmark run`** — specify Sonnet, Opus, or Haiku for agent runs. Passed directly to claude CLI. Required for cost control and reproducibility.
- **`--conditions 3` on `benchmark run`** — enables the 3-arm experiment (without-scope, with-scope, with-scope-preloaded). Previously only available on `prepare`.
- **`--save-ndjson <dir>` flag** — persists raw NDJSON streams from every agent run. Enables post-hoc analysis, action replay, and token decomposition.
- **`--bare` flag on claude CLI invocation** — ensures consistent behavior (no hooks, no LSP, no plugin sync) for reproducible benchmark runs
- **Preloaded condition in automated runs** — `setup_temp_corpus` handles `with-scope-preloaded` by running `scope map` and baking output into CLAUDE.md before the agent starts
- **`run_agent` returns `(AgentRun, TempDir)`** — caller controls temp dir lifetime, enabling verification before cleanup

### Breaking Changes
- `run_agent` signature changed — now accepts `condition`, `model`, `ndjson_save_path` parameters and returns tuple with TempDir
- Default reps changed from 5 to 3

---

## v0.5.2 (2026-03-22)

### Performance (from skill-validated code review)
- **Fix O(N) linear scan in `resolve_caller_count`** — the initial N+1 fix in v0.5.1 replaced SQL N+1 but introduced an O(N) HashMap scan for suffix matching. Now uses pre-computed `by_suffix` HashMap for true O(1) lookups across all three matching patterns.

### Code Quality
- Move trace truncation message into formatter (was bypassing output layer)
- Improve `--limit` help text on `scope trace` for LLM readability

### Tests
- Add 6 unit tests in `graph.rs` for `resolve_caller_count` (all three matching patterns, combined, no-match, is_test_file)

---

## v0.5.1 (2026-03-22)

### Performance
- **Fix N+1 queries in `scope map` and `scope entrypoints`** — replaced per-symbol `get_caller_count()` with single aggregate query (`get_all_caller_counts`), replaced per-entrypoint outgoing count with pre-computed HashMap, replaced O(N×M) `.ends_with()` scan with O(1) HashSet lookup
- These three fixes compound: `scope map` on large codebases should now stay well within the < 500ms performance target

### CLI
- **`scope trace --limit N`** — truncate paths output (default 20) with "... N more paths" message. Prevents unbounded output on heavily-called symbols.

### Code Quality
- Extract shared `collapse_and_group()` function for entrypoint processing (was duplicated between entrypoints.rs and map.rs)
- Remove no-op `format_step_name` function, inline at call sites

---

## scope-benchmark v0.5.1 (2026-03-22)

### Fixes (from skill-validated code review)
- Fix duplicate rows in 3-arm Token Decomposition report (skip boolean split when per-condition data exists)
- Set condition labels in `run_benchmarks()` for automated runs (was `String::new()`, making condition-aware reporting dead code)
- Replace manual `Clone`/`Default` impls with derives on `BenchmarkRun` and `CorrectnessResult`
- Extract `unique_conditions()` helper (was duplicated 3×)
- Log WalkDir errors in manifest.rs instead of silent swallow
- Add human-readable default output to `benchmark verify` (new `--json` flag)
- Log skipped non-JSON line count in `parse_ndjson_actions`

### Tests
- Add `insta` snapshot tests for `scope trace`, `scope entrypoints`, `scope map` (output format is a contract)

---

## scope-benchmark v0.5.0 (2026-03-22)

### Phase 11 Infrastructure
Based on the Phase 10 Technical Review, this release adds the infrastructure needed for the 3-arm benchmark experiment (without-scope / with-scope / with-scope-preloaded).

### New Features
- **NDJSON action import**: `benchmark import --ndjson-dir <path>` parses saved Claude CLI NDJSON streams to populate action-level data (tools used, scope commands, file reads). Fixes the P10 gap where 0/72 runs had action data.
- **3-arm experiment support**: `benchmark prepare --conditions 3` generates three variants per task: without-scope, with-scope, and with-scope-preloaded (scope map output baked into CLAUDE.md)
- **Caller coverage verification**: Replaced stub in verifier.rs with real unified diff parser that checks ground truth callers against agent changes (±5 line context window)
- **Token decomposition**: Captures `cache_creation_input_tokens` and `cache_read_input_tokens` from NDJSON usage events. New "Token Decomposition" section in markdown reports shows fresh vs cached input tokens.
- **Condition-aware reporting**: Markdown reports now group by experimental condition (not just scope_enabled bool), supporting the 3-arm analysis

### Fixture Changes
- Added `CLAUDE.md.with-scope-preloaded` templates for both TypeScript and C# fixtures — includes `{{SCOPE_MAP_OUTPUT}}` placeholder replaced at prepare time

### Task Definition
- Added optional `[ground_truth]` section to task TOML schema with `callers` field for caller coverage verification

---

## scope-benchmark v0.4.0 (2026-03-21)

### Fixture Overhaul
Phase 10 (72 runs) revealed 3 of 6 benchmark categories had fixture problems. This release fixes all 6 categories, adds fixture integrity protection, and adds a `benchmark verify` command.

### Fixture Changes
- **Cat-A (Discovery)**: Updated prompts to resist grep — TS focuses on "charge decline handling" (word "retry" removed), CS focuses on "permanently abandoned notifications" (word "delivery" removed)
- **Cat-B (Bug Fix)**: CS bug replaced — compile-error (`payment.PaymentMethod.Last4Digits`) → runtime data integrity bug (RecordPayment called before status check)
- **Cat-D (New Feature)**: TS `PaymentAnalyticsService.ts` deleted (was pre-existing, defeating "new feature" purpose), CS `PaymentReceipt.cs` entity added for domain context
- **Cat-F (Cross-cutting)**: TS now has 6 catch blocks (was 2) for meaningful cross-cutting work, CS task changed from CancellationToken propagation to structured logging (CancellationToken would break fixture for other tasks)
- **Cat-C, Cat-E**: Verified clean, no changes needed

### New Commands
- **`benchmark manifest --generate`**: Generates SHA256 manifests for all fixture source files
- **`benchmark manifest --verify`**: Verifies fixtures match their stored manifests — prevents accidental corruption
- **`benchmark verify --dir <path>`**: Runs correctness checks (compilation, tests) on a completed work directory and outputs JSON scores

### Import Changes
- Correctness data is now optional in `benchmark import` — defaults to zeros with a warning
- Removes need for hardcoded correctness values in manually captured results

---

## scope-benchmark v0.3.0 (2026-03-21)

### Methodology Redesign
Phases 7-9 revealed fundamental methodology problems: 1-rep results had up to 10× variance (per research paper), Cat-E "document the architecture" was open-ended and unrealistic, and no correctness verification beyond compilation. This release addresses all three.

### Changes
- **6 task categories** (was 5): added Category F (cross-cutting changes across multiple files)
- **Cat-E reframed**: "document the full architecture" → "explain payment flow for debugging" — focused exploration with a purpose and natural stopping point
- **Cat-F added**: TS (add structured error logging to all catch blocks) + C# (add CancellationToken to interface methods and all callers) — tests whether Scope helps with changes that touch many files
- **3 reps per condition** for statistical reliability (72 runs per phase, was 20)
- **Standard deviations** reported alongside means in all aggregate metrics
- **12 tasks total** (6 TypeScript + 6 C#), up from 10

### Why
- Research paper: "token consumption varies up to 10× across runs on the same task" — single-rep comparisons are unreliable
- CS Cat-E consistently produced 18-sketch outliers across P7/P8/P9 due to open-ended task framing
- Cross-cutting changes (Cat-F) are a common real-world workflow not previously tested

---

## v0.5.0 (2026-03-21)

### Features
- **`scope map`** — full repository overview in ~500-1000 tokens: entry points, core symbols ranked by caller count, architecture summary with directory stats. Replaces 5-17 scope sketch calls for orientation tasks. Research shows repo-level maps enable 4.2× fewer tokens (Aider) and +12.2% accuracy (RIG).
- **`scope entrypoints`** — lists API controllers, background workers, and event handlers grouped by type. Entry points are symbols with zero incoming call edges. Saves 2-3 navigation actions per task.
- **Importance scoring in FTS5** — symbols with more callers rank higher in `scope find` results. Importance tiers (high/medium) are embedded in the FTS5 index for natural BM25 boosting.

### Research-Driven
These changes are based on the LLM Coding Agent Tool-Use Research Report (50+ sources):
- Aider's repo-map enables 4.2× fewer tokens than Claude Code (the `scope map` motivation)
- Repository Intelligence Graph showed +12.2% accuracy and -53.9% time from architectural overview
- Meta-tools reduce agent reasoning steps by 11.9% (the `scope entrypoints` motivation)
- PageRank-style importance scoring is what makes Aider's repo-map effective

---

## v0.4.0 (2026-03-20)

### Features
- **`scope trace <symbol>`** — new command showing entry-point-to-symbol call paths for bug-fix workflows; uses recursive CTE to walk the call graph backward from the target to entry points
- **`scope callers --depth N`** — merged impact analysis into callers; `--depth 1` (default) shows direct callers with snippets, `--depth 2+` shows transitive callers grouped by depth with test file separation
- **Enriched sketch output** — methods now show `async`, `private`, `static`, `abstract`, `virtual`, `override` modifiers extracted from tree-sitter metadata
- **Enriched FTS5 search** — `scope find` now indexes caller/callee relationships, file path components, and snake_case splits; BM25 weights tuned to boost file path matches

### Deprecations
- `scope impact` now delegates to `scope callers --depth N` with a deprecation notice to stderr; existing scripts continue to work

### Improvements
- FTS5 query builder now splits snake_case terms (`payment_retry` → `payment* OR retry*`)
- FTS5 indexed text includes caller names (`called-by`), callee names (`calls`), and directory path segments (`path payments services`)
- Sketch modifier display omits defaults (public is not shown; only private/protected/async/static shown)

---

## v0.3.1 (2026-03-20)

### Features
- **`benchmark prepare`** — sets up isolated work directories with CLAUDE.md variants and .scope/ indexes, outputs manifest.json with prompts for manual runs. No API key required.
- **`benchmark import`** — ingests manually captured results (tokens, actions, tool calls), recomputes behavior metrics, generates full analysis reports with CLI recommendations.
- Large fixture resolver now prefers `{lang}-large` over `{lang}-api`

### Fixes
- Fix task count assertion (10 tasks, not 20) after test matrix consolidation

---

## v0.3.0 (2026-03-20)

### Features
- **Automated benchmark runner** — `agent.rs` implements full `claude -p` invocation with stream-json parsing, temp directory isolation, and NDJSON tool call extraction
- **Agent behavior analysis** — new `behavior.rs` module computes navigation efficiency, scope anti-patterns, tool overlap metrics, and generates data-driven CLI recommendations
- **Behavior-aware reports** — markdown summaries now include agent behavior section with navigation:edit ratios, anti-pattern counts, and CLI evolution recommendations
- Large fixture support in corpus resolver (prefers `{lang}-large` over `{lang}-api`)

### Benchmark Runner (scope-benchmark v0.2.0)
- `agent.rs`: temp dir isolation per run (no cross-contamination), CLAUDE.md swapping, `--disallowedTools` for baseline condition, stream-json parsing for tool call capture
- `behavior.rs`: `BehaviorMetrics` struct, `compute_behavior_metrics()`, `aggregate_behavior()`, anti-pattern detection (sketch-then-read, grep-after-find, callers+refs overlap)
- `reporter.rs`: behavior analysis section with navigation tables, anti-pattern counts, CLI recommendations
- 34 unit tests (13 agent + 16 behavior + 3 verifier + 2 task)

---

## v0.2.0 (2026-03-19)

### Breaking Changes
- **Renamed binary from `sc` to `scope`** — all commands are now `scope sketch`, `scope refs`, etc.

### Features
- `scope callers` alias — shorthand for `scope refs --kind calls`
- Source line snippets in `scope refs` output — each reference now shows the actual source line at the call site
- Benchmark harness with diverse 5-category test matrix (discovery, bug fix, refactoring, new feature, exploration)
- Large fixtures: TypeScript (194 files) and C# (182 files) for realistic benchmarks

### Fixes
- Fix FTS5 prefix matching for camelCase symbols in `scope find`
- Fix output spacing in `scope find` results
- Fix parser edge `from_id` to use enclosing scope instead of synthetic `__module__` IDs
- Fix symbol ID collisions by including line numbers in IDs

### Benchmarks
- Phase 6 diverse matrix: 20 runs across 5 categories show Scope's value is task-dependent
  - New feature integration: 25-33% token savings (sweet spot)
  - Discovery: 7-29% savings
  - Bug fixing: 9-71% worse (agents over-investigate)
  - Refactoring: ~0% (neutral)
  - Exploration: 6-21% mixed

---

## v0.1.0 (2026-03-17)

### Features
- `scope init` — initialise Scope for a project
- `scope index` — build/refresh code index (incremental by default)
- `scope sketch` — structural overview of classes, methods, interfaces, files
- `scope refs` — find all references with kind filtering
- `scope deps` — dependency queries with transitive depth
- `scope impact` — blast radius analysis with recursive CTE
- `scope find` — semantic search via SQLite FTS5
- `scope status` — index health reporting
- TypeScript + C# language support
- `--json` output on all commands

### Known Limitations
- `scope find` uses FTS5 (keyword matching), not vector embeddings
- `scope rdeps`, `scope similar`, `scope source` not yet implemented
- No `--watch` mode
