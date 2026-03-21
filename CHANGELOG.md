# Changelog

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
