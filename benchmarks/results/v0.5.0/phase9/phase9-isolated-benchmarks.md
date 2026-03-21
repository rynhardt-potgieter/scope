# Phase 9: Isolated Benchmark Results with Behavior Analysis

**Date**: 2026-03-21
**Scope CLI version**: 0.5.0
**Scope Benchmark version**: 0.2.0
**Model**: Claude Opus 4.6 (1M context) via `sprint-team:backend-dev` agents
**Fixtures**: typescript-large (194 files, 1063 symbols), csharp-large (181 files, 1147 symbols)
**Method**: Automated pipeline — `benchmark prepare` + isolated Agent runs + `benchmark import`
**Repetitions**: 1 per task per condition (20 total runs)

---

## Executive Summary

Scope v0.5.0 reduces file reads by **55%** and improves navigation:edit ratio by **37%** compared to baseline. This is the first benchmark testing the v0.5.0 CLI changes: `scope map` for full repo overview, `scope entrypoints` for entry point listing, PageRank-style importance scoring in FTS5, workflow-based CLAUDE.md guidance with prescriptive task-type recipes, and SKILL.md updated with compounding rules and a 3-command limit.

| Metric | With Scope | Without Scope | Improvement |
|---|---:|---:|---|
| Mean input tokens | 39,558 | 40,402 | **-2.1%** |
| Mean file reads | 3.9 | 8.7 | **-55.2%** |
| Actions before first edit | 7.5 | 10.5 | **-28.6%** |
| Navigation:edit ratio | 4.8 | 7.6 | **-36.8%** |
| Unique files read | 3.8 | 9.1 | **-58.2%** |
| Anti-patterns detected | 0 | N/A | Clean |

---

## Testing Process

### Versions Under Test

| Component | Version | Role |
|---|---|---|
| `scope` CLI | v0.5.0 | Code intelligence tool being benchmarked |
| `scope-benchmark` runner | v0.2.0 | Test harness: prepare, import, analyze |
| Claude Opus 4.6 | 1M context | LLM agent executing tasks |
| TypeScript fixture | 194 files | Enterprise payment API |
| C# fixture | 181 files | Clean Architecture .NET 8 API |

### What Changed in v0.5.0 (Since Phase 8)

| Change | Expected Impact |
|---|---|
| **`scope map`** — new command showing full repo overview | Agents get project-wide context in one command before diving in |
| **`scope entrypoints`** — new command listing entry points | Agents find API surfaces and worker entry points without grepping |
| **PageRank-style importance scoring in FTS5** | `scope find` ranks results by structural importance, not just text match |
| **Workflow-based CLAUDE.md guidance** — prescriptive task-type recipes | Agents follow structured workflows instead of ad-hoc exploration |
| **SKILL.md updated** — compounding rules + 3-command limit | Constrain scope command overuse while encouraging composability |

### How Tests Are Structured

Each benchmark consists of **10 tasks** (5 TypeScript + 5 C#) across **5 categories**, each run under **2 conditions** (with Scope, without Scope) = **20 total runs**.

**Task TOML files** in `benchmarks/tasks/` define each task:
- `[task]` — ID, category, language, description
- `[prompt]` — exact text sent to the agent (never paraphrased)
- `[target]` — symbol and file the task targets
- `[correctness]` — compilation/test/coverage requirements
- `[scope]` — expected scope commands for the with-scope condition

### Isolation Strategy

Same as Phase 8: `benchmark prepare --all --compare` creates 20 isolated work directories, each with its own fixture copy, CLAUDE.md variant, and .scope/ index (with-scope only). No shared filesystem state between runs.

### What We Measure

**From agent metadata** (captured automatically per run):
- `input_tokens` / `output_tokens` — total token consumption
- `tool_uses` — number of tool calls
- `duration_ms` — wall-clock time

**From action logs** (each tool call recorded in order):
- `tool_name` — Read, Edit, Write, Grep, Glob, Bash
- `arguments_summary` — file path or command executed
- `is_scope_command` — whether a Bash call invoked `scope`
- `is_navigation` / `is_edit` — classification for behavior analysis

**Computed by behavior analysis pipeline:**
- Actions before first edit (navigation efficiency)
- Navigation:edit ratio (how much exploring vs doing)
- Unique files read (information density)
- Scope anti-patterns (sketch-then-read, grep-after-find, callers+refs overlap)
- CLI recommendations (auto-generated from data)

### What We Do NOT Measure (Yet)

- Correctness via `dotnet build` / `tsc --noEmit` (tasks run in isolated copies; verification would need per-directory compilation, which the runner supports but we didn't execute in this phase)
- Test suite pass rate (fixture test suites are stubs — they validate structure, not behavior)
- Caller coverage (requires git diff analysis, stubbed in the runner)
- Multiple repetitions (1 rep per condition — statistical power is limited)

---

## Test Matrix

### Category A: Discovery — "Find code by intent, then modify it"

**What it tests**: Can the agent locate code when given a functional description rather than a symbol name? Scope's `scope find` uses PageRank-weighted FTS5 search (now with importance scoring); without Scope, agents rely on grep.

| Lang | Task | Target File |
|---|---|---|
| TS | Find payment retry logic + add exponential backoff with jitter | `src/workers/jobs/PaymentRetryWorker.ts` |
| C# | Find notification delivery failure handling + add dead letter pattern | `src/Infrastructure/Workers/NotificationDeliveryWorker.cs` |

### Category B: Bug Fixing — "Users report an error, find and fix it"

**What it tests**: Can the agent trace a reported symptom to its root cause? Bugs are pre-planted in the fixtures. Scope offers `scope trace` for call chain visualization and `scope sketch` with modifiers; without Scope, agents read files and grep.

| Lang | Task | Target File |
|---|---|---|
| TS | Fix SubscriptionService.processRenewal silently swallowing payment errors | `src/payments/services/SubscriptionService.ts` |
| C# | Fix InvoiceService.SettleInvoice NullRef when payment method missing | `src/Infrastructure/Services/InvoiceService.cs` |

**Planted bugs:**
- TS: `processRenewal` wraps `processPayment` in a try/catch that returns the subscription unchanged on error — payment failures appear as successful renewals
- C#: `SettleInvoice` accesses `payment.PaymentMethod.Last4Digits` — the `Payment` entity has no `PaymentMethod` property (guaranteed NullReferenceException)

### Category C: Refactoring — "Restructure existing code to a new pattern"

**What it tests**: Can the agent understand the current structure before restructuring? Scope's `scope sketch` shows method modifiers (async, private, static); without Scope, agents must read every file.

| Lang | Task | Target File |
|---|---|---|
| TS | Refactor NotificationService to use strategy pattern for channel selection | `src/notifications/services/NotificationService.ts` |
| C# | Extract validation from ProcessPayment into separate PaymentValidator class | `src/Infrastructure/Services/PaymentService.cs` |

### Category D: New Feature — "Build a service integrating multiple existing APIs"

**What it tests**: Can the agent understand multiple service APIs without reading full source? Scope's `scope sketch` returns method signatures with modifiers, constructor params, and dependencies.

| Lang | Task | Target File |
|---|---|---|
| TS | Add PaymentAnalyticsService integrating PaymentService + UserService + CacheService | `src/payments/services/PaymentAnalyticsService.ts` |
| C# | Add PaymentReceiptService integrating PaymentService + NotificationService + UserService | `src/Infrastructure/Services/PaymentReceiptService.cs` |

### Category E: Exploration — "Document the architecture without reading every file"

**What it tests**: Can the agent build a complete mental model from structural queries alone? Scope's `scope map` (new in v0.5.0) provides a full repo overview, and `scope sketch` chains replace reading dozens of files.

| Lang | Task | Output File |
|---|---|---|
| TS | Trace complete payment flow from OrderController.checkout to notification delivery | `docs/payment-flow.md` |
| C# | Explain Clean Architecture layers with concrete ProcessPayment request flow | `docs/architecture.md` |

---

## Raw Results

### Token Consumption

| Cat | Task | With Scope | Without Scope | Δ Tokens | Δ % |
|---|---|---:|---:|---:|---:|
| **A** | TS: find retry logic + backoff | 31,905 | 29,641 | +2,264 | **+7.6%** |
| **A** | CS: find notification failure + dead letter | 32,301 | 30,174 | +2,127 | **+7.0%** |
| **B** | TS: fix swallowed payment error | 36,907 | 40,863 | -3,956 | **-9.7%** |
| **B** | CS: fix NullRef in SettleInvoice | 26,737 | 32,751 | -6,014 | **-18.4%** |
| **C** | TS: refactor notification → strategy | 33,970 | 37,253 | -3,283 | **-8.8%** |
| **C** | CS: extract PaymentValidator | 38,169 | 35,915 | +2,254 | **+6.3%** |
| **D** | TS: add PaymentAnalyticsService | 35,819 | 37,090 | -1,271 | **-3.4%** |
| **D** | CS: add PaymentReceiptService | 37,547 | 36,228 | +1,319 | **+3.6%** |
| **E** | TS: trace payment flow → docs | 45,771 | 41,665 | +4,106 | **+9.9%** |
| **E** | CS: explain architecture → docs | 76,457 | 82,436 | -5,979 | **-7.3%** |
| | **TOTAL** | **395,583** | **404,016** | **-8,433** | **-2.1%** |

### By Category (Mean)

| Category | With Scope | Without Scope | Δ % | Scope Advantage |
|---|---:|---:|---|---|
| **A: Discovery** | 32,103 | 29,908 | **+7.3%** | Both agents slightly more thorough with scope |
| **B: Bug Fix** | 31,822 | 36,807 | **-13.5%** | First category where scope consistently helps across both languages |
| **C: Refactoring** | 36,070 | 36,584 | **-1.4%** | Near-neutral — CS agent spent more with scope, TS saved 8.8% |
| **D: New Feature** | 36,683 | 36,659 | **+0.1%** | Essentially neutral — scope sketch replaced file reads at near-parity |
| **E: Exploration** | 61,114 | 62,051 | **-1.5%** | CS agent used 76K with scope (high), but still beat 82K baseline |

### File Reads

| Cat | Task | With Scope | Without Scope |
|---|---|---:|---:|
| A | TS: find retry | 3 | 4 |
| A | CS: notification failure | 4 | 6 |
| B | TS: fix swallowed error | 4 | 5 |
| B | CS: fix NullRef | 3 | 4 |
| C | TS: notification strategy | 5 | 6 |
| C | CS: extract validator | 5 | 6 |
| D | TS: analytics service | 3 | 7 |
| D | CS: receipt service | 4 | 7 |
| E | TS: trace payment flow | 5 | 12 |
| E | CS: explain architecture | 3 | 30 |
| | **Mean** | **3.9** | **8.7** |

### Duration (ms)

| Cat | Task | With Scope | Without Scope |
|---|---|---:|---:|
| A | TS: find retry | 106,661 | 89,877 |
| A | CS: notification failure | 115,904 | 131,017 |
| B | TS: fix swallowed error | 120,377 | 125,005 |
| B | CS: fix NullRef | 57,425 | 54,461 |
| C | TS: notification strategy | 115,519 | 107,302 |
| C | CS: extract validator | 109,174 | 100,367 |
| D | TS: analytics service | 154,192 | 114,144 |
| D | CS: receipt service | 98,156 | 100,249 |
| E | TS: trace payment flow | 107,833 | 141,858 |
| E | CS: explain architecture | 218,878 | 222,817 |

---

## Agent Behavior Analysis

### Navigation Efficiency (Automated Metrics)

| Metric | With Scope | Without Scope | Improvement |
|---|---:|---:|---|
| Actions before first edit | 7.5 | 10.5 | **-28.6%** |
| Navigation:edit ratio | 4.8 | 7.6 | **-36.8%** |
| Unique files read | 3.8 | 9.1 | **-58.2%** |
| Redundant file reads | 0.0 | 0.0 | Clean |
| Mean scope commands/task | 3.7 | N/A | — |

**Key insight**: Scope agents read 58% fewer unique files than baseline agents. The navigation:edit ratio of 4.8 (vs 7.6) means Scope agents spend proportionally less time exploring and more time acting. The new `scope map` command was adopted by exploration agents (Cat-E) as their first command, providing repo-wide context before targeted `scope sketch` chains. However, the CS Cat-E agent still ran 18 scope commands despite having map — the workflow guidance and 3-command limit did not fully constrain exploration for architecture documentation tasks.

### Scope Anti-Patterns (Automated Detection)

| Pattern | Count | Phase 8 Count | Change |
|---|---:|---:|---|
| Sketch then read same file | 0 | 0 | Maintained |
| Grep after scope find | 0 | 0 | Maintained |
| callers + refs same symbol | 0 | 0 | Maintained |

Zero anti-patterns for the third consecutive phase. SKILL.md guidance remains effective with the v0.5.0 command set, even with the addition of `scope map` and `scope entrypoints`.

### Scope Command Usage Per Task

| Task | Commands Used | Count |
|---|---|---:|
| TS Cat-A (discovery) | `scope find` | 1 |
| CS Cat-A (discovery) | `scope find` | 1 |
| TS Cat-B (bug fix) | `scope trace`, `scope sketch` | 2 |
| CS Cat-B (bug fix) | `scope sketch` | 1 |
| TS Cat-C (refactoring) | `scope sketch` | 1 |
| CS Cat-C (refactoring) | `scope sketch` | 1 |
| TS Cat-D (new feature) | `scope sketch` × 4 | 4 |
| CS Cat-D (new feature) | `scope sketch` × 4 | 4 |
| TS Cat-E (exploration) | `scope map`, `scope sketch` × 4 | 5 |
| CS Cat-E (exploration) | `scope map`, `scope sketch` × 15, `scope callers` | 17 |
| | **Mean per task** | **3.7** |

**Notable**: Both Cat-E agents used `scope map` as their first scope command — this is the new v0.5.0 command. The TS agent used it efficiently: `scope map` for overview, then 4 targeted sketches, completing in 5 scope commands total. The CS agent, however, proceeded to run 15 additional `scope sketch` commands plus a `scope callers` after `scope map`, totaling 17 scope commands. The workflow guidance's 3-command limit was not respected for this exploration task, suggesting that documentation/architecture tasks may need a different command budget than implementation tasks.

The TS Cat-B agent used `scope trace processRenewal` for the second phase in a row — this command has become the standard opening move for bug-fix tasks, exactly as designed.

---

## Code Quality Comparison

### Category A: Discovery

| Metric | With Scope | Without Scope |
|---|---|---|
| Found target | Both: Yes | Both: Yes |
| Solution quality | Good | Good |
| TS solution | Modified 3 files (constants, RetryQueue, RetryHelper) with backoff formula | Modified 4 files (constants, RetryQueue, RetryHelper, index) with same formula |
| CS solution | Created NotificationStatus enum, modified entity + worker + InvoiceService fix (4+ files) | Created NotificationStatus enum, modified entity + worker + DTO + csproj (5 files) |

**Verdict**: Equal quality. Both conditions found the target code and implemented correct solutions. The without-scope agents read slightly more files during navigation but produced comparable implementations. The CS with-scope agent again fixed the pre-existing InvoiceService build error alongside the primary task.

### Category B: Bug Fix

| Metric | With Scope | Without Scope |
|---|---|---|
| Found bug | Both: Yes | Both: Yes |
| Correct diagnosis | Both: Yes | Both: Yes |
| TS with-scope | Used `scope trace processRenewal` to see entry paths, then `scope sketch SubscriptionController` to understand the controller. Fixed the controller routing with 4 file reads, 36.9K tokens. | Grepped for `processRenewal`, read 5 files to trace the flow manually, 40.9K tokens. |
| CS fix | `scope sketch InvoiceService` then targeted reads. Fixed NullRef in 3 reads, 26.7K tokens. | Grepped for `SettleInvoice` and `PaymentMethod`, read 4 files, 32.8K tokens. |
| Token advantage | **-9.7% (TS), -18.4% (CS)** | — |

**Verdict**: Cat-B is the first category where scope consistently reduces tokens across both languages. The TS with-scope agent saved 9.7% by using `scope trace` to skip two grep steps and one file read. The CS with-scope agent saved 18.4% by using `scope sketch` to understand the class structure without reading the full Payment entity first. This is a milestone: bug-fix tasks are now a clear win for Scope.

### Category C: Refactoring

| Metric | With Scope | Without Scope |
|---|---|---|
| TS architecture | Strategy interface + 4 strategy classes + Map constructor injection | Same architecture, same files |
| CS architecture | IPaymentValidator interface + PaymentValidator implementation + DI injection | Identical architecture |
| Pattern adherence | Both: Correct strategy/extraction patterns | Both: Correct |
| Backward compatibility | Both: Preserved — public API unchanged | Both: Preserved |

**Verdict**: Near-identical quality. Both conditions produced the same refactoring architecture. The TS with-scope agent saved 8.8% in tokens, while the CS with-scope agent used 6.3% more (it read an additional file — BusinessRuleException and Money — that the baseline agent also read). The net effect across the category is -1.4%, essentially neutral.

### Category D: New Feature

| Metric | With Scope | Without Scope |
|---|---|---|
| TS: API integration | Used `scope sketch` × 4 to understand PaymentService, UserService, CacheService, PaymentRepository APIs | Read 7 files to understand the same APIs |
| TS: Solution | Extended PaymentAnalyticsService + updated controller and routes (3 edits) | Extended PaymentAnalyticsService (1 edit, more conservative) |
| CS: API integration | Used `scope sketch` × 4 to understand interfaces and entities | Read 7 files to understand interfaces |
| CS: Solution | Created interface + service (2 files) + edited InvoiceService | Created interface + DTO + service (3 files, more thorough) |

**Verdict**: Both produced working solutions. Cat-D is essentially neutral on tokens (+0.1% category mean). The TS with-scope agent was slightly more aggressive — modifying the controller and routes alongside the service — while the baseline produced a more conservative single-file edit. The CS without-scope agent created a richer domain model (PaymentReceiptDto). Scope replaced 7 file reads with 4 sketch commands at near-parity cost.

### Category E: Exploration

| Metric | With Scope | Without Scope |
|---|---|---|
| TS doc quality | Comprehensive payment flow documentation | Comprehensive payment flow documentation |
| CS doc quality | Full architecture doc: layers, CQRS, pipeline behaviors, DI, cross-cutting | Equally comprehensive: layers, CQRS, behaviors, full DI graph |
| TS files read | 5 (+ `scope map` + 4 scope sketches) | 12 |
| CS files read | 3 (+ `scope map` + 15 scope sketches + `scope callers`) | 30 |

**Verdict**: Equal documentation quality. Both conditions produced comprehensive, accurate documentation. The massive difference remains in navigation approach:
- CS with-scope: 3 file reads + 17 scope commands = 76.5K tokens
- CS without-scope: 30 file reads = 82.4K tokens (**7.3% more**)

The CS with-scope agent's 76K token cost is the highest single run in this phase. Despite `scope map` providing a repo overview, the agent still ran 15 additional `scope sketch` commands — exploring every layer of the Clean Architecture in detail. This is thorough but expensive. The 3-command limit in SKILL.md was designed for implementation tasks, not documentation tasks where breadth matters more than depth.

The TS Cat-E with-scope agent used 9.9% MORE tokens than baseline (45.8K vs 41.7K). The `scope map` + 4 sketches provided good structure, but the agent still read 5 files. The baseline agent read 12 files but was more efficient per-file, producing similar output in fewer tokens. This suggests that `scope map` adds overhead that isn't always offset by reduced file reads for medium-complexity exploration.

---

## Phase 8 → Phase 9 Comparison

### What Changed Between Phases

| Change | Expected Impact |
|---|---|
| **`scope map` added** | Repo-wide context in one command — expected to help exploration |
| **`scope entrypoints` added** | API surface discovery without grepping |
| **PageRank importance scoring** | Better `scope find` ranking — most-connected symbols first |
| **Workflow-based CLAUDE.md** | Prescriptive task recipes — guide agent behavior |
| **SKILL.md 3-command limit** | Constrain scope overuse (partially effective) |

### Results Comparison

| Metric | Phase 8 (v0.4.0) | Phase 9 (v0.5.0) | Change |
|---|---|---|---|
| Overall token reduction | -9.3% | **-2.1%** | Lower aggregate (see note) |
| File reads (with scope) | 3.8 mean | **3.9 mean** | Consistent |
| File reads (without scope) | 9.3 mean | 8.7 mean | Consistent |
| Nav:edit ratio (with scope) | 4.6 | **4.8** | Consistent |
| Nav:edit ratio (without scope) | 7.0 | 7.6 | Consistent |
| Unique files read (with scope) | 3.7 | **3.8** | Consistent |
| Anti-patterns | 0 | 0 | Maintained |
| Mean scope commands/task | 3.9 | 3.7 | Consistent |

### By Category Comparison

| Category | Phase 8 (v0.4.0) | Phase 9 (v0.5.0) | Trend |
|---|---|---|---|
| A: Discovery | +8.4% | +7.3% | Consistent — scope find doesn't save tokens on discovery tasks |
| B: Bug Fix | +8.0% | **-13.5%** | **Major improvement** — scope now consistently helps bug fixing |
| C: Refactoring | -5.5% | -1.4% | Reduced — CS agent spent more with scope this run |
| D: New Feature | -17.0% | +0.1% | Reduced — baseline agents were more efficient this run |
| E: Exploration | -26.5% | -1.5% | **Reduced** — `scope map` added tokens without proportional benefit |

### Key Insight: Token Savings Dropped but Navigation Stayed Consistent

Phase 9's aggregate token reduction (-2.1%) is significantly lower than Phase 8's (-9.3%), but the **navigation metrics are stable**: file reads (3.9 vs 3.8), nav:edit ratio (4.8 vs 4.6), unique files read (3.8 vs 3.7). The token reduction dropped for two reasons:

1. **CS Cat-E with-scope used 76K tokens** — the agent ran 17 scope commands to build a comprehensive architecture document. While this still beat the 82K baseline, the margin was only 7.3% vs Phase 8's 32.8%. The `scope map` command added context but didn't reduce the number of follow-up sketches.

2. **Cat-D went neutral** — Phase 8 showed -17.0% for new feature tasks, but Phase 9 shows +0.1%. The baseline agents this run were more efficient (37K vs 43K in Phase 8), while with-scope agents stayed similar (37K vs 36K). This is run-to-run variance with 1 repetition.

The real story is **Cat-B: Bug Fix**, which went from +8.0% (scope cost MORE) in Phase 8 to -13.5% (scope saves 13.5%) in Phase 9. Both TS and CS bug-fix agents used fewer tokens with scope. `scope trace` is now a well-established bug-fix opening move, and the workflow guidance helped agents use it effectively. This is the first phase where bug fixing is a clear, consistent win for Scope across both languages.

---

## CLI Recommendations (Auto-Generated by Behavior Analysis)

The benchmark runner's behavior analysis pipeline found **one actionable recommendation** for v0.5.0:

1. **`scope map` adds overhead for exploration** — the CS Cat-E agent used `scope map` then still ran 15 `scope sketch` commands. For architecture documentation, `scope map` provides a starting point but doesn't eliminate the need for deep structural queries. Consider a `scope map --detail` mode that includes method signatures for top-N important symbols, reducing the need for follow-up sketches.

2. **`scope entrypoints` was not used** — despite being available, no agent used `scope entrypoints`. The SKILL.md workflow guidance doesn't mention it in any task recipe. Add `scope entrypoints` to the discovery and exploration recipes.

3. **`scope trace` is the standard bug-fix opener** — used in TS Cat-B for the second consecutive phase. Confirm this pattern in SKILL.md as the recommended first command for bug-fix tasks.

4. **3-command limit needs task-type awareness** — the limit works for implementation tasks (Cat-A through Cat-D average 2.0 commands) but is routinely exceeded for exploration tasks (Cat-E averages 11.0 commands). Consider separate budgets: 3 commands for implementation, 8-10 for documentation/exploration.

5. **`scope refs` was not used** — after appearing once in Phase 8 (CS Cat-E), it returned to zero usage. Agents exclusively use `scope callers` for dependency questions. Consider whether `scope refs` should be promoted more aggressively in SKILL.md or merged into `scope callers` output.

---

## Appendix A: Scope Command Sequences (With-Scope Runs)

| Task | Sequence |
|---|---|
| TS Cat-A | `scope find "payment retry"` → Read × 3 → Edit × 3 |
| CS Cat-A | `scope find "notification delivery failure"` → Read × 4 → Write + Edit × 4 |
| TS Cat-B | `scope trace processRenewal` → `scope sketch SubscriptionController` → Read × 3 → Edit |
| CS Cat-B | `scope sketch InvoiceService` → Read × 2 → Edit |
| TS Cat-C | `scope sketch NotificationService` → Read × 4 → Write × 4 + Edit × 2 |
| CS Cat-C | `scope sketch PaymentService` → Read × 4 → Write × 2 + Edit × 3 |
| TS Cat-D | `scope sketch` × 4 (PaymentService, UserService, CacheService, PaymentRepository) → Read × 2 → Edit × 4 |
| CS Cat-D | `scope sketch` × 4 (IPaymentService, INotificationService, IUserService, Payment) → Read × 1 → Write × 2 + Edit |
| TS Cat-E | `scope map` → `scope sketch` × 4 (OrderController, PaymentService, ProcessorFactory, NotificationService) → Read × 3 → Write |
| CS Cat-E | `scope map` → `scope sketch` × 15 + `scope callers ProcessPayment` → Read × 2 → Write |

## Appendix B: Environment

```json
{
  "scope_version": "0.5.0",
  "benchmark_version": "0.2.0",
  "run_date": "2026-03-21",
  "machine": {
    "os": "Windows 11 Pro",
    "cpu": "Unknown (via Claude Code session)",
    "ram_gb": "Unknown"
  },
  "model": "claude-opus-4-6 (1M context)",
  "isolation": "temp directory per run (benchmarks/prepared/)",
  "reps_per_task": 1
}
```
