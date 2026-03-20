# Changelog

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
