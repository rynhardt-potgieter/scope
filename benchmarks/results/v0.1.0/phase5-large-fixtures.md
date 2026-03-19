# Phase 5: Large Fixture Benchmark

**Date:** 2026-03-19
**Scope version:** 0.1.0 (with AX improvements)
**Agent:** Claude Opus 4.6 (1M context) via Claude Code
**Methodology:** 4 parallel agents with worktree isolation, identical task prompts
**Fixtures:** typescript-large (194 files, 1,064 symbols, 3,303 edges), csharp-large (181 files, 1,147 symbols, 1,409 edges)

---

## What Changed From Phase 4

| Dimension | Phase 4 (small) | Phase 5 (large) |
|-----------|----------------|-----------------|
| TS files | 14 | 194 |
| TS symbols | 73 | 1,064 |
| C# files | 18 | 181 |
| C# symbols | 91 | 1,147 |
| Module depth | 2 layers | 4 layers |
| Cross-module deps | ~10 | 50+ |

**Hypothesis:** On larger codebases, grep becomes noisier and Scope's structural queries should provide a bigger navigation advantage.

---

## Task

Same Category A signature refactoring:

> Refactor `processPayment`/`ProcessPayment` to accept a single `PaymentRequest` object instead of individual parameters. Update ALL callers.

---

## Results

### TypeScript Large (194 files)

**CAVEAT:** The typescript-large fixture was generated with `processPayment` already accepting a `PaymentRequest` object. Both agents discovered no changes were needed. The comparison is therefore a **navigation-only** benchmark — how efficiently did each agent verify the codebase state?

| Metric | With Scope | Without Scope | Delta |
|--------|-----------|---------------|-------|
| Total tokens | 21,762 | 28,063 | **-22.4%** |
| Tool uses | 12 | 9 | +33% |
| Duration | 49s | 32s | +53% |
| Callers verified | 7/7 | 6/7 | Scope more thorough |

**Scope agent navigation (12 tool calls):**
1. `scope callers processPayment` — got all 7 callers with source lines
2. `scope sketch PaymentService` — confirmed method accepts PaymentRequest
3. Read `PaymentTypes.ts` — confirmed PaymentRequest type
4. Read 2 caller files — spot-checked call expressions match
5. Concluded: already refactored, 7/7 callers verified

**Blind agent navigation (9 tool calls):**
1. Read `PaymentService.ts` — saw the method signature
2. Read `PaymentTypes.ts` — confirmed PaymentRequest type
3. Grep for `processPayment` across codebase — got matches
4. Read 3-4 caller files to verify call expressions
5. Concluded: already refactored, found 6 of 7 callers

**Key finding:** The Scope agent verified all 7 callers in one `scope callers` call. The blind agent's grep found matches but it only verified 6 of 7 — it stopped reading files before checking `InvoiceService.settleInvoice`. On a navigation-only task, Scope provided **more complete coverage with fewer file reads**.

### C# Large (181 files)

Both agents performed the actual refactoring — this is the real comparison.

| Metric | With Scope | Without Scope | Delta |
|--------|-----------|---------------|-------|
| Total tokens | 62,938 | 41,099 | **+53.1%** |
| Tool uses | 54 | 39 | +38% |
| Duration | 191s | 161s | +19% |
| Callers found | 7/7 | 7/7 | Tied |
| Files modified | 9 | 9 | Tied |
| New files created | 1 (PaymentRequest.cs) | 1 (PaymentRequest.cs) | Tied |

**Scope used 53% MORE tokens.** This is the opposite of the expected result.

---

## Agent Behavior Analysis

### C# Scope Agent (62,938 tokens, 54 tool calls)

```
Phase 1: Navigation with Scope (18 calls, ~15,000 tokens)
  scope sketch PaymentService
  scope refs ProcessPayment
  scope callers ProcessPayment
  scope impact ProcessPayment
  scope deps PaymentService
  scope find "PaymentRequest"
  scope sketch IPaymentService
  Read PaymentRequest type file
  Read IPaymentService interface
  Read PaymentService implementation

Phase 2: Editing (36 calls, ~48,000 tokens)
  Create PaymentRequest.cs (new file)
  Read + Edit IPaymentService.cs
  Read + Edit PaymentService.cs
  Read + Edit PaymentController.cs
  Read + Edit SubscriptionController.cs
  Read + Edit ProcessPaymentHandler.cs
  Read + Edit PaymentRetryWorker.cs
  Read + Edit InvoiceService.cs
  Read + Edit RefundService.cs
```

**Problem:** The agent ran 6 scope commands consuming ~15,000 tokens before making any edits. Several were redundant:
- `scope refs` and `scope callers` return overlapping data
- `scope impact` wasn't needed for a signature refactoring
- `scope deps` wasn't needed — the task is about callers, not dependencies
- `scope find "PaymentRequest"` was unnecessary — the task already says where the type is

**The navigation phase consumed 24% of total tokens but produced information that grep could have provided in 2 calls.**

### C# Blind Agent (41,099 tokens, 39 tool calls)

```
Phase 1: Navigation (8 calls, ~8,000 tokens)
  Read PaymentService.cs (understand current signature)
  Grep "ProcessPayment" across codebase (find all callers + interface)
  Read IPaymentService.cs
  Glob for "*PaymentRequest*" (find existing type)
  Read the PaymentRequest area to understand structure

Phase 2: Editing (31 calls, ~33,000 tokens)
  Create PaymentRequest.cs (new file)
  Read + Edit IPaymentService.cs
  Read + Edit PaymentService.cs
  Read + Edit each of 7 caller files
```

**The blind agent's navigation phase was 50% cheaper** — 8,000 tokens vs 15,000. It used grep once to find all callers, read only the files it needed, and moved to editing immediately.

---

## Why Scope Was Slower on Large C#

### 1. Over-navigation from prompt design

The Scope agent prompt said "use scope commands FIRST to navigate." The agent interpreted this as "run every available scope command before editing." On a task where the target is already known (processPayment in PaymentService.cs), this is wasteful.

**Fix:** The prompt should say "Run `scope callers ProcessPayment` to get all call sites, then edit." Not "run sketch, refs, callers, impact, deps, and find."

### 2. Scope commands produce verbose output

Each scope command returns structured output that costs tokens to process:
- `scope sketch` on a class with 4 methods and fields: ~300 tokens
- `scope refs` with 7 callers showing source lines: ~500 tokens
- `scope callers` with 7 callers: ~400 tokens (redundant with refs)
- `scope impact` with depth analysis: ~300 tokens
- `scope deps` with dependency tree: ~400 tokens
- `scope find` with 8 results: ~500 tokens

Total navigation output: ~2,400 tokens of scope output, most of which was redundant for this task.

### 3. Grep is efficient on Clean Architecture

The C# large fixture follows Clean Architecture with clear namespacing. `grep ProcessPayment` in a well-structured .NET project returns clean, precise results — almost no noise. The expected "grep becomes noisy on large codebases" didn't materialize because:
- Method names in C# are PascalCase and unique
- Clean Architecture has predictable file locations
- The method name `ProcessPayment` appears only at actual call sites and the declaration

### 4. Read-before-edit is mandatory regardless

Both agents read each caller file before editing. The Scope agent's source line snippets in refs output didn't eliminate this — the Edit tool requires the file to have been read first. So the editing phase costs are identical.

---

## What Would Make Scope Win on Large Codebases

### Task type matters more than codebase size

Category A (signature refactoring) is the **worst case for Scope** because:
- The target method name is unique and greppable
- All callers use the exact method name
- No transitive analysis needed
- The task is mechanical — find + replace

**Scope's advantage should appear on:**

| Task type | Why grep fails | Why Scope wins |
|-----------|---------------|----------------|
| Category D: "Make findById async" | Callers of callers need updating (transitive chain) | `scope impact --depth 2` traces the full chain |
| Category E: "Find retry logic" | Method name isn't known — need semantic search | `scope find "retry logic"` returns relevant symbols |
| Category C: "Replace a dependency" | Need to understand what depends on what | `scope deps` shows the dependency graph |
| Cross-cutting: "Add logging to all methods that call the DB" | Grep can't identify "methods that call the DB" | `scope refs DatabaseClient --kind calls` returns exactly those methods |

### Prompt engineering for optimal scope usage

The agent should be instructed differently based on task type:

**Refactoring (Category A):** "Run `scope callers <method>` once. Edit each caller. Done."
**Impact analysis (Category D):** "Run `scope impact <method> --depth 2`. This shows everything you need to change."
**Discovery (Category E):** "Run `scope find '<description>'` to locate the relevant code."
**Dependency swap (Category C):** "Run `scope deps <class>` and `scope refs <class>` to understand what uses it."

The current "use scope commands FIRST" prompt is too broad and causes over-navigation.

---

## Statistical Summary

### All Runs Combined (4 runs total per condition)

| Condition | TS Small (14 files) | TS Large (194 files) | C# Small (18 files) | C# Large (181 files) |
|-----------|--------------------|--------------------|--------------------|--------------------|
| Scope tokens (mean) | 29,348 | 21,762* | 28,658 | 62,938 |
| Blind tokens (mean) | 28,857 | 28,063* | 31,070 | 41,099 |
| Delta | +1.7% | -22.4%* | -7.8% | +53.1% |

*TS Large is navigation-only (no actual refactoring). Not directly comparable.

**Honest conclusion for Category A tasks:** Scope shows no consistent advantage. On small codebases it's within noise. On large codebases, over-navigation makes it more expensive unless the prompt is tightly optimized.

---

## Raw Data

### TS Large — With Scope

```
Agent ID:       acbf0256733ef0541
Total tokens:   21,762
Tool uses:      12
Duration:       49,218ms
Task:           No changes needed (already refactored)
Callers verified: 7/7
Scope commands:  scope callers, scope sketch
```

### TS Large — Without Scope

```
Agent ID:       a5f405e099273102a
Total tokens:   28,063
Tool uses:      9
Duration:       32,389ms
Task:           No changes needed (already refactored)
Callers verified: 6/7 (missed InvoiceService)
Navigation:     Grep, Read
```

### C# Large — With Scope

```
Agent ID:       afc70bde91a808487
Total tokens:   62,938
Tool uses:      54
Duration:       191,405ms
Callers found:  7/7
Files modified: 9 (1 created + 8 edited)
Scope commands: scope sketch (x2), scope refs, scope callers, scope impact, scope deps, scope find
```

### C# Large — Without Scope

```
Agent ID:       ac8a90ce8a20bd528
Total tokens:   41,099
Tool uses:      39
Duration:       161,164ms
Callers found:  7/7
Files modified: 9 (1 created + 8 edited)
Navigation:     Grep, Read, Glob
```

---

## Conclusion

Phase 5 on large fixtures produced a surprising result: Scope was **more expensive** on the C# refactoring task due to over-navigation. The tool returned correct, comprehensive data — but the agent consumed too many tokens collecting information it didn't need for a mechanical refactoring.

The takeaway is not that Scope is inefficient, but that **Category A tasks don't exercise Scope's strengths.** Grep is near-optimal for "find all calls to method X" on well-structured codebases. Scope's value lies in tasks that grep can't solve: transitive impact analysis, semantic discovery, and dependency understanding.

Phase 6 (not yet run) should benchmark Category D and E tasks where grep-based navigation is fundamentally insufficient.
