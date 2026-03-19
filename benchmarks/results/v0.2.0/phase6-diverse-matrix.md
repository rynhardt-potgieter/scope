# Phase 6: Diverse Test Matrix Benchmark Results

**Date**: 2026-03-19
**Scope version**: 0.1.0
**Model**: Claude Opus 4.6 (1M context) via sprint-team:backend-dev agents
**Fixtures**: typescript-large (194 files), csharp-large (182 files)
**Method**: Manual agent invocation with worktree isolation, 1 rep per condition

---

## Executive Summary

20 benchmark runs (10 tasks × 2 conditions) across 5 task categories and 2 languages. Overall token reduction with Scope: **9.1%** — but the aggregate hides stark category-level differences. Scope's value is **highly task-dependent**.

| Where Scope Helps | Token Reduction | Why |
|---|---|---|
| New Feature (integrating APIs) | **25-33%** | `scope sketch` reads API signatures without full file reads |
| Discovery (find code by intent) | **7-29%** | `scope find` locates code faster than grep |
| Exploration (architecture docs) | **21%** (C# only) | `scope sketch` chain maps architecture quickly |

| Where Scope Hurts | Token Increase | Why |
|---|---|---|
| Bug Fixing | **9-71% MORE** | Agents over-investigate instead of reading the buggy file |
| Refactoring (known target) | **~0%** (neutral) | Target file is known; grep and direct read are equally fast |

---

## Raw Results

### Token Consumption

| Cat | Task | With Scope | Without Scope | Δ Tokens | Δ % |
|---|---|---:|---:|---:|---:|
| **A** | TS: find retry logic + backoff | 24,635 | 26,532 | -1,897 | **-7%** |
| **A** | CS: find notification failure + dead letter | 29,770 | 42,156 | -12,386 | **-29%** |
| **B** | TS: fix swallowed payment error | 25,240 | 23,092 | +2,148 | **+9%** |
| **B** | CS: fix NullRef in SettleInvoice | 46,393 | 27,183 | +19,210 | **+71%** |
| **C** | TS: refactor notification → strategy | 29,213 | 29,236 | -23 | **0%** |
| **C** | CS: extract PaymentValidator | 32,881 | 37,013 | -4,132 | **-11%** |
| **D** | TS: add PaymentAnalyticsService | 34,149 | 50,742 | -16,593 | **-33%** |
| **D** | CS: add PaymentReceiptService | 26,491 | 35,421 | -8,930 | **-25%** |
| **E** | TS: trace payment flow → docs | 40,235 | 37,899 | +2,336 | **+6%** |
| **E** | CS: explain architecture → docs | 53,579 | 67,495 | -13,916 | **-21%** |
| | **TOTAL** | **342,586** | **376,769** | **-34,183** | **-9.1%** |

### Tool Uses

| Cat | Task | With Scope | Without Scope | Δ |
|---|---|---:|---:|---:|
| A | TS: find retry | 15 | 13 | +2 |
| A | CS: notification failure | 24 | 32 | -8 |
| B | TS: fix swallowed error | 11 | 9 | +2 |
| B | CS: fix NullRef | 35 | 15 | +20 |
| C | TS: notification strategy | 32 | 29 | +3 |
| C | CS: extract validator | 23 | 19 | +4 |
| D | TS: analytics service | 24 | 35 | -11 |
| D | CS: receipt service | 19 | 34 | -15 |
| E | TS: trace payment flow | 27 | 35 | -8 |
| E | CS: explain architecture | 45 | 48 | -3 |
| | **TOTAL** | **255** | **269** | **-14** |

### Duration (ms)

| Cat | Task | With Scope | Without Scope |
|---|---|---:|---:|
| A | TS: find retry | 74,981 | 84,581 |
| A | CS: notification failure | 145,711 | 177,695 |
| B | TS: fix swallowed error | 78,508 | 54,242 |
| B | CS: fix NullRef | 273,829 | 69,572 |
| C | TS: notification strategy | 129,864 | 113,451 |
| C | CS: extract validator | 125,310 | 108,710 |
| D | TS: analytics service | 139,114 | 210,314 |
| D | CS: receipt service | 115,829 | 123,879 |
| E | TS: trace payment flow | 206,622 | 134,942 |
| E | CS: explain architecture | 249,726 | 214,926 |

---

## Code Quality Analysis

### Category A: Discovery — "Find X and add Y"

**Scope advantage: `scope find` locates code by intent.**

| Metric | TS With | TS Without | CS With | CS Without |
|---|---|---|---|---|
| Found target | Yes | Yes | Yes | Yes |
| Solution quality | Excellent | Excellent | Excellent | Excellent |
| Files modified | 3 | 3 | 4 | 4 |

**Observations:**
- Both conditions found the same files and applied the same solution
- TS: Identical quality — both updated `constants.ts`, `RetryHelper.ts`, `PaymentRetryQueue.ts` with the same backoff formula
- CS: Both implemented dead letter pattern with `NotificationStatus` enum, `MarkAsDeadLettered()` method, and structured logging. With-scope was 29% cheaper
- **Winner: Tie on quality.** Scope saved tokens on discovery but didn't improve the solution

### Category B: Bug Fixing — "X is returning an error, find and fix"

**Scope disadvantage: agents over-investigate with scope commands.**

| Metric | TS With | TS Without | CS With | CS Without |
|---|---|---|---|---|
| Found bug | Yes | Yes | Yes | Yes |
| Correct diagnosis | Yes | Yes | Yes | Yes |
| Fix quality | Good | Good | Over-engineered | **Excellent** |
| Lines changed | ~10 | ~10 | ~30 | **2** |

**TS Bug (swallowed error):**
- Both agents identified the silently swallowing `catch` block in `SubscriptionService.processRenewal`
- Both applied the same fix: increment failed attempts, log error, throw `ValidationError`
- With-scope ran 4 scope commands (find × 2, sketch, callers) — unnecessary for this task
- Without-scope grep'd for "renew", read 3 files, found the bug. More direct

**CS Bug (NullRef in SettleInvoice):**
- **With-scope (46K tokens, 35 tool uses, 274s)**: Ran 5 scope commands (`sketch InvoiceService`, `callers SettleInvoice`, `sketch Invoice`, `sketch Payment`, `sketch PaymentMethod`). Correctly identified `payment.PaymentMethod.Last4Digits` as the bug. But then **over-engineered**: added 3 extra defensive guards (null-check on `TotalAmount`/`PaidAmount`, payment method validation via `ValidatePaymentMethod`, null-check on `payment` result). These guards are defensive but go far beyond the actual bug.
- **Without-scope (27K tokens, 15 tool uses, 70s)**: Read 4 files. Found the dead code. Removed the 2 offending lines (`// Log payment details for audit trail` + `var methodDetails = payment.PaymentMethod.Last4Digits;`). Clean, minimal, correct. **4× faster, 41% fewer tokens.**
- **Winner: Without-scope.** The minimal fix was the correct fix. Scope encouraged over-investigation.

### Category C: Refactoring — "Refactor X to do Y"

**Scope neutral: target is already known.**

| Metric | TS With | TS Without | CS With | CS Without |
|---|---|---|---|---|
| Correct refactoring | Yes | Yes | Yes | Yes |
| Pattern adherence | Excellent | Excellent | Excellent | Excellent |
| Preserved behavior | Yes | Yes | Yes | Yes |

**TS (strategy pattern):**
- Both agents produced identical architecture: `NotificationStrategy` interface, 3 strategy classes, `Map<NotificationChannel, NotificationStrategy>` in constructor
- With-scope discovered pre-existing strategy directory (from fixture structure); without-scope created the files from scratch
- Both updated `NotificationService.deliver()` to delegate via strategy map
- Token usage virtually identical (29,213 vs 29,236)

**CS (extract validator):**
- Both agents created `IPaymentValidator` in Application/Interfaces and `PaymentValidator` in Infrastructure/Services
- Both extracted the same 3 validation rules (empty userId, non-positive amount, missing token)
- Both updated PaymentService constructor and DI registration
- With-scope was 11% cheaper (2 scope commands vs reading 9 files)

**Winner: Tie on quality.** Slight token edge to Scope for C# due to `scope sketch` reducing file reads.

### Category D: New Feature — "Build feature integrating X, Y, and Z"

**Scope advantage: `scope sketch` reads API signatures without full file reads.**

| Metric | TS With | TS Without | CS With | CS Without |
|---|---|---|---|---|
| API understanding | Excellent | Excellent | Excellent | Excellent |
| Integration quality | Excellent | Good | Excellent | Excellent |
| Backward compat | Yes (preserved existing) | Yes (optional params) | Yes | Yes |

**TS (PaymentAnalyticsService):**
- **With-scope (34K tokens)**: Ran 4 `scope sketch` commands to understand `PaymentService`, `UserService`, `CacheService`, `PaymentRepository` APIs. Learned constructor signatures, method names, return types — all without reading the full source files. Preserved existing `getRevenueSummary`/`getDailyRevenue` for backward compat with `AnalyticsController`.
- **Without-scope (51K tokens)**: Read 14 files to understand the same APIs. Made constructor params optional. Equivalent solution but **50% more tokens** and **46% more tool uses**.
- **Winner: With-scope.** Same quality, 33% fewer tokens. This is Scope's sweet spot — understanding multiple APIs before writing integration code.

**CS (PaymentReceiptService):**
- **With-scope (26K tokens)**: Ran 7 `scope sketch` commands. Created `PaymentReceipt` entity with factory method, `IPaymentReceiptService` interface, service implementation.
- **Without-scope (35K tokens)**: Read 14 files. Built equivalent solution.
- **Winner: With-scope.** 25% fewer tokens, equivalent quality.

### Category E: Exploration — "Explain a complex feature"

**Scope mixed: powerful for architecture mapping but prone to over-navigation.**

| Metric | TS With | TS Without | CS With | CS Without |
|---|---|---|---|---|
| Doc comprehensiveness | Excellent | Excellent | Excellent | Excellent |
| Accuracy | High | High | High | High |
| Scope commands used | 9 | N/A | **33** | N/A |
| Files read directly | 9 | 16+ | 3 | 34 |

**TS (trace payment flow):**
- With-scope: 9 sketch commands + 9 file reads. Documented 8-step flow.
- Without-scope: 35 tool uses, 16+ file reads. Documented same 8-step flow.
- With-scope used 6% MORE tokens despite fewer file reads. The scope command overhead outweighed the savings.

**CS (explain architecture):**
- **With-scope: 33 scope commands** — this is the over-navigation anti-pattern in action. The agent ran `scope sketch` on nearly every class in the codebase. Despite this, it only read 3 files directly, producing a 529-line doc.
- Without-scope: Read 34 files, produced equally comprehensive doc.
- With-scope saved 21% tokens despite the excessive commands, because each `scope sketch` is cheaper than a full file read.
- **Winner: With-scope on tokens (-21%), but the agent violated the "don't run 3+ commands before editing" rule.**

---

## Key Findings

### 1. Scope's Sweet Spot: Multi-Service Integration (Cat D)

When agents need to understand multiple APIs before writing new code, `scope sketch` provides exactly the right information density — method signatures, constructor params, dependencies — without loading 100+ line source files. **25-33% token reduction.**

### 2. Scope's Weakness: Bug Fixing (Cat B)

Bug fixing requires reading and understanding actual source code, not structural summaries. Scope commands add overhead without adding insight. Worse, they can encourage over-investigation (CS Cat-B: 5 scope commands + 3 unnecessary defensive guards = 71% MORE tokens). **Agents should just read the file.**

### 3. Discovery Works, But Grep Works Too (Cat A)

`scope find` is semantically richer than grep, but for well-named code, grep finds the same targets. The 7-29% reduction is real but modest. Scope's advantage grows in larger, less well-structured codebases.

### 4. Exploration Over-Navigation (Cat E)

The CS with-scope agent ran 33 scope commands — the exact anti-pattern our skill file warns against. Despite this, it still saved 21% tokens. This suggests the decision tree is right (scope sketch IS efficient for exploration), but the "max 2-3 commands" guidance needs enforcement.

### 5. Code Quality Was Equal Across Conditions

Both conditions produced correct, compilable solutions for all 10 tasks. The only quality difference was CS Cat-B where without-scope produced a **cleaner** fix (2-line removal vs 30-line over-engineering). Scope didn't improve problem-solving ability — it primarily affects navigation efficiency.

---

## Recommendations

### For the Scope Agent Skill (SKILL.md)

1. **Add explicit bug-fix guidance**: "For bug fixing, read the suspected file directly. Don't scope-sketch it — you need the actual code, not a summary."
2. **Enforce the 3-command limit**: The CS Cat-E agent ran 33 commands. The skill should say "If you've run 5+ scope commands, you're exploring for its own sake. Start writing."
3. **Differentiate 'find' vs 'sketch'**: Agents sometimes used `scope find` when they already knew the symbol name (should use `scope sketch` or `scope callers`).

### For the Benchmark Harness

1. **Add per-category reporting**: The 9.1% aggregate hides the real story. Report by category.
2. **Track over-navigation**: Count scope commands per task. Flag runs with >5 commands.
3. **Measure fix precision**: For bug fix tasks, count lines changed. Fewer = better.
4. **Run 3-5 reps**: Single-rep results are noisy. The TS Cat-E result (6% worse with scope) might flip with more reps.

### For Fixture Design

1. **Bug fix tasks are the hardest test**: They expose whether Scope adds real value vs just overhead. Keep them in the matrix.
2. **New feature tasks are the easiest win**: They consistently show Scope's value. Good for marketing but don't overweight them.

---

## Appendix: Scope Commands Per Run

| Task | Scope Commands Used |
|---|---|
| TS Cat-A (discovery) | `scope find "retry logic"` (1 command) |
| TS Cat-B (bug fix) | `scope find` × 2, `scope sketch`, `scope callers` (4 commands) |
| TS Cat-C (refactoring) | `scope sketch`, `scope callers` (2 commands) |
| TS Cat-D (new feature) | `scope sketch` × 4 (4 commands) |
| TS Cat-E (exploration) | `scope sketch` × 7, `scope callers` × 1 (9 commands, 1 redundant) |
| CS Cat-A (discovery) | `scope find` × 2 (2 commands) |
| CS Cat-B (bug fix) | `scope sketch` × 4, `scope callers` × 1 (5 commands) |
| CS Cat-C (refactoring) | `scope sketch` × 1, `scope find` × 1 (2 commands) |
| CS Cat-D (new feature) | `scope sketch` × 7 (7 commands) |
| CS Cat-E (exploration) | `scope sketch` × 31, `scope callers` × 1, `scope find` × 1 (33 commands!) |
