# Phase 11 — Rep 1 Results

**Date:** 2026-03-22
**Scope version:** 0.5.2
**Benchmark version:** 0.5.1
**Model:** Claude Opus 4.6 (1M context)
**Runs:** 36 (12 tasks × 3 conditions × 1 rep)
**Conditions:** without-scope, with-scope, with-scope-preloaded (scope map baked into CLAUDE.md)

## Token Consumption by Condition

| Condition | Mean tokens | Total tokens |
|-----------|------------|-------------|
| without-scope | 33,269 | 399,233 |
| with-scope | 33,390 | 400,678 |
| **with-scope-preloaded** | **32,297** | **387,565** |

**Preloaded vs without-scope:** -2.9% mean tokens
**Preloaded vs with-scope:** -3.3% mean tokens
**With-scope vs without-scope:** +0.4% (neutral/worse)

## Per-Category Breakdown

| Category | without | with | preloaded | Best | Δ vs worst |
|----------|---------|------|-----------|------|------------|
| CS Discovery | 32,078 | 31,962 | **31,008** | preloaded | -3.3% |
| CS Bug Fix | **20,045** | 24,438 | 26,015 | without | -23% |
| CS Refactoring | 31,803 | **29,691** | 34,041 | with | -13% |
| CS New Feature | **32,926** | 35,717 | 35,007 | without | -8% |
| CS Exploration | 45,854 | 45,178 | **42,063** | preloaded | -8.3% |
| CS Cross-cutting | **35,866** | 37,181 | 35,516 | preloaded | -4.4% |
| TS Discovery | **25,926** | 31,858 | 27,603 | without | -19% |
| TS Bug Fix | 31,143 | 35,829 | **30,526** | preloaded | -15% |
| TS Refactoring | 39,320 | 29,372 | **28,449** | preloaded | -28% |
| TS New Feature | 26,954 | **25,746** | 27,720 | with | -7% |
| TS Exploration | 44,896 | 42,468 | **38,278** | preloaded | -15% |
| TS Cross-cutting | 32,422 | **31,238** | 31,339 | with | -3.7% |

## Win Count by Condition

| Condition | Wins | Categories |
|-----------|------|-----------|
| without-scope | 3 | CS bug fix, CS new feature, TS discovery |
| with-scope | 3 | CS refactoring, TS new feature, TS cross-cutting |
| **with-scope-preloaded** | **6** | CS discovery, CS exploration, CS cross-cutting, TS bug fix, TS refactoring, TS exploration |

## Key Observations

1. **Preloaded wins 6/12 categories** — the scope map pre-loading hypothesis is directionally validated
2. **Exploration is preloaded's strongest category** — CS -8%, TS -15% (consistent with the technical review's prediction that architectural overview drives the largest gains)
3. **TS Refactoring shows -28% for preloaded** — strongest single-category signal in any Phase 11 data
4. **Bug fix remains problematic for scope** — CS without-scope wins by 23%, though TS preloaded narrowly wins
5. **With-scope (current) is barely different from without-scope** (+0.4%) — the opt-in scope map usage problem identified in Phase 10 persists
6. **Cross-cutting quality gap** — without-scope found 6/6 TS catch blocks, scope conditions found 4/6 (scope guidance may have narrowed search scope)

## Duration

| Condition | Mean duration (s) | Total duration (s) |
|-----------|------------------|-------------------|
| without-scope | 108 | 1,299 |
| with-scope | 109 | 1,310 |
| with-scope-preloaded | 107 | 1,285 |

Duration is nearly identical across conditions — token savings don't translate to wall-clock time at this fixture scale.

## Caveats

- **Single rep** — these results have high variance. Phase 10 showed categories can swing ±6,000 tokens across reps.
- **No action-level data** — agents run via Agent tool, not claude CLI with NDJSON capture. Behavior metrics unavailable.
- **Correctness is compilation-only** — no semantic verification, no caller coverage.
- **Token counts are total_tokens** (input + output + cache), not decomposed.
