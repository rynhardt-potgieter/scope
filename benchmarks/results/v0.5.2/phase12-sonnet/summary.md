# Phase 12 — First Fully Automated Benchmark (Sonnet 4.6)

**Date:** 2026-03-22
**Scope version:** 0.5.2
**Benchmark runner:** scope-benchmark v0.7.2
**Model:** Claude Sonnet 4.6 (via Claude CLI, `--bare` mode)
**Fixture:** csharp-large (181 files, Clean Architecture, CQRS, DI)
**Total runs:** 54 (6 C# tasks × 3 conditions × 3 reps)
**Execution:** Fully automated via `benchmark run` with NDJSON capture, incremental save, and resume support. First benchmark phase executed entirely via the Anthropic API — no Agent tool, no manual intervention.

## Overall Results

| Metric | without-scope | with-scope | preloaded | Δ preloaded vs without |
|--------|:---:|:---:|:---:|:---:|
| **Output tokens** (mean) | 16,474 | 11,297 | **10,692** | **-35%** |
| **Cost per run** | $0.580 | $0.410 | **$0.393** | **-32%** |
| **File reads** | **6.8** | 9.4 | 9.7 | +43% |
| **Total actions** | 30.2 | 31.9 | **29.8** | -1% |
| **Actions before first edit** | **22.3** | 24.4 | 23.6 | +6% |
| **Compilation pass rate** | 83.3% | **94.4%** | 88.9% | +5.6pp |
| **Cache read tokens** | 507,774 | 369,724 | **361,925** | **-29%** |

### Headline Findings

1. **Preloaded saves 35% on output tokens and 32% on cost** vs without-scope. This is the strongest aggregate signal in any benchmark phase.
2. **With-scope has the highest compilation rate (94.4%)** — Scope guidance improves code quality.
3. **Without-scope reads fewer files (6.8 vs 9.7)** but uses more output tokens — it's surgically narrow but generates more reasoning/code text.
4. **Cache reads drop 29% with preloading** — fewer conversational turns mean less context re-sent.

## Per-Category Breakdown

| Category | without | with | preloaded | Best | Cost winner |
|----------|:---:|:---:|:---:|:---:|:---:|
| **A — Discovery** | 8,344 out | 8,672 out | **8,100 out** | preloaded -3% | without ($0.30) |
| **B — Bug Fix** | 8,773 out | 4,934 out | **3,395 out** | **preloaded -61%** | **preloaded ($0.14)** |
| **C — Refactoring** | **5,045 out** | 4,774 out | 5,494 out | with -5% | without ($0.22) |
| **D — New Feature** | 12,661 out | 8,901 out | **6,730 out** | **preloaded -47%** | **preloaded ($0.29)** |
| **E — Exploration** | 32,968 out | 26,176 out | **25,920 out** | **preloaded -21%** | **with-scope ($0.78)** |
| **F — Cross-cutting** | 31,054 out | **14,327 out** | 14,515 out | **with -54%** | **with-scope ($0.51)** |

### Compilation Pass Rates

| Category | without | with | preloaded |
|----------|:---:|:---:|:---:|
| A — Discovery | 100% | 100% | 100% |
| B — Bug Fix | 100% | 100% | 100% |
| C — Refactoring | 100% | 100% | 100% |
| D — New Feature | 100% | 100% | 100% |
| E — Exploration | 100% | 100% | 100% |
| **F — Cross-cutting** | **0%** | **67%** | **33%** |

Cat-F (cross-cutting logging changes across PaymentService + UserService) is the only category with compilation failures. The without-scope condition failed all 3 reps.

## Key Observations

### Preloaded dominates cost efficiency
Across all categories, preloaded averages $0.393/run vs $0.580 for without-scope — a 32% cost reduction. The savings come from:
- **Fewer output tokens** (10,692 vs 16,474) — the agent generates less reasoning text when it already has architectural context
- **Fewer cache reads** (361K vs 507K) — fewer conversational turns mean less context re-sent per turn

### Bug Fix result reversed from Phase 10
In Phase 10, bug fix was neutral (without-scope won by 0.4%). In Phase 12 with proper telemetry, **preloaded saves 61% on output tokens for bug fixes**. The preloaded agent finds and fixes the bug in 15 actions (vs 24 for without-scope) because the scope map tells it exactly where InvoiceService sits in the architecture.

### New Feature is preloaded's strongest category by cost
Preloaded saves 47% output tokens on new feature creation. The scope map provides the architectural context needed to understand existing service patterns before creating a new one — exactly the use case the Phase 10 Technical Review predicted.

### Cross-cutting is broken
Cat-F has a 0% compilation rate for without-scope and 33% for preloaded. The task (adding structured logging to all public methods in PaymentService + UserService) produces large code changes that Sonnet struggles to get right across multiple files. This category needs task redesign or is inherently harder than the other categories.

## Cost Summary

| | without-scope | with-scope | preloaded |
|---|:---:|:---:|:---:|
| Total cost (18 runs) | $10.44 | $7.38 | $7.07 |
| Mean cost per run | $0.580 | $0.410 | $0.393 |
| Cheapest category | B: $0.35 | B: $0.20 | B: $0.14 |
| Most expensive | E: $1.09 | E: $0.78 | E: $0.80 |

Total Phase 12 cost: **$24.89** for 54 runs.

## Scope Command Usage

| Command | Frequency (with-scope) | Frequency (preloaded) |
|---------|:---:|:---:|
| `scope find` | Common | Common |
| `scope map` | Used in discovery, exploration | N/A (already in CLAUDE.md) |
| `scope trace` | Used in bug fix, exploration | Used in bug fix, exploration |
| `scope sketch` | Used in new feature, refactoring | Used in new feature, refactoring |

The preloaded condition doesn't need `scope map` since the output is already baked into CLAUDE.md. Agents still use targeted commands (`scope find`, `scope trace`, `scope sketch`) for specific queries.

## Methodology Notes

- **Fully automated**: All 54 runs executed via `benchmark run --all --conditions 3 --reps 3 --model sonnet` with `--resume` support
- **NDJSON captured**: Raw Claude CLI streams saved for every run (54 files, ~200KB-300KB each)
- **Incremental save**: Results written to disk after every completed run — safe to interrupt and resume
- **Duration throttled by rate limits**: 8K output tokens/min, 30K input tokens/min — wall-clock time is not a reliable speed metric
- **Compilation verification**: `dotnet build --no-restore` run against the agent's temp directory after each run
