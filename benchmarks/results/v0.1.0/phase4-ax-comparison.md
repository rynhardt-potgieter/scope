# Phase 4: AX Comparison Benchmark — Small Fixtures

**Date:** 2026-03-19
**Scope version:** 0.1.0 (with AX improvements: source line snippets in refs, docstrings in sketch, `scope callers` alias)
**Agent:** Claude Opus 4.6 (1M context) via Claude Code
**Methodology:** 4 parallel agents with worktree isolation, identical task prompts

---

## What Changed Between Runs

### AX Improvements Applied (Phase 1)

| Feature | Before | After |
|---------|--------|-------|
| `scope refs` output | Shows caller name only: `OrderController.checkout` | Shows actual source line: `this.paymentService.processPayment(amount, userId, card)` |
| `scope sketch` output | Method signatures only | Adds docstring above each method (first line of JSDoc/XML comment) |
| `scope callers` command | Did not exist | Alias for `scope refs --kind calls` |
| `scope refs --context N` | Did not exist | Shows N lines of surrounding code per reference |

**Hypothesis:** Source line snippets in refs should reduce file reads — the agent sees the call expression without opening the file.

---

## Task

Identical to Phase 1 baseline:

> Refactor `processPayment` in PaymentService to accept a single `PaymentRequest` object instead of individual `amount`, `userId`, and `card` parameters. Update ALL callers.

**Fixtures:** typescript-api (14 files, 73 symbols), csharp-api (18 files, 91 symbols)
**Ground truth:** 7 callers in each fixture

---

## Results

### TypeScript

| Metric | Run 1: Scope | Run 1: Blind | **Run 2: Scope** | **Run 2: Blind** |
|--------|-------------|-------------|-----------------|-----------------|
| Total tokens | 28,372 | 31,822 | **30,324** | **25,891** |
| Tool uses | 28 | 34 | **23** | **23** |
| Duration | 100s | 126s | **97s** | **85s** |
| Callers found | 7/7 | 7/7 | **7/7** | **7/7** |
| Files modified | 6 | 6 | **6** | **6** |
| Compiles | Yes | Yes | **Yes** | **Yes** |

**Run 2 delta:** Scope used 17% MORE tokens than blind (+4,433). Blind agent was significantly more efficient this run.

### C#

| Metric | Run 1: Scope | Run 1: Blind | **Run 2: Scope** | **Run 2: Blind** |
|--------|-------------|-------------|-----------------|-----------------|
| Total tokens | 28,652 | 31,607 | **28,664** | **30,532** |
| Tool uses | 23 | 20 | **21** | **18** |
| Duration | 85s | 84s | **81s** | **77s** |
| Callers found | 7/7 | 7/7 | **7/7** | **7/7** |
| Files modified | 6 | 6 | **6** | **6** |
| Compiles | Yes | Yes | **Yes** | **Yes** |

**Run 2 delta:** Scope used 6% fewer tokens (-1,868). Consistent with Run 1 (9% fewer).

---

## Agent Behavior Analysis

### What the Scope-Enabled TS Agent Did (Run 2)

1. Ran `scope sketch PaymentService` — got class overview with methods + caller counts + docstrings
2. Ran `scope refs processPayment` — got 7 callers WITH actual source lines (new AX feature)
3. Read `src/payments/types.ts` to check PaymentRequest structure
4. Read `src/payments/service.ts` to understand method body
5. Edited service.ts (signature change)
6. For each caller file: read the file, edited the call site
7. **23 tool uses total**

**Key observation:** The agent still read every caller file before editing, even though the refs output now showed the actual call expression. The source line snippet (`this.paymentService.processPayment(amount, userId, card)`) told the agent WHAT to change, but the agent's editing pattern is "read file → find the line → apply edit" regardless. The snippet didn't eliminate file reads — it just confirmed which files needed editing.

### What the Blind TS Agent Did (Run 2)

1. Read `src/payments/service.ts` — understood the method
2. Read `src/payments/types.ts` — checked PaymentRequest
3. Used Grep to find all `processPayment` call sites across the codebase
4. For each match: read the file, edited the call site
5. **23 tool uses total** — same count as Scope agent

**Key observation:** This agent was unusually efficient. The Grep returned all 7 callers in a single command, and the agent navigated directly to each file without exploratory reads. On a 14-file codebase, Grep is essentially a perfect navigation tool — it finds everything in one pass.

### What the Scope-Enabled C# Agent Did (Run 2)

1. Ran `scope sketch PaymentService` — class overview
2. Ran `scope refs ProcessPayment` — 7 callers with source lines
3. Ran `scope callers ProcessPayment` — (redundant, same as refs --kind calls)
4. Read `PaymentRequest.cs` to check record structure
5. Edited IPaymentService.cs, PaymentService.cs
6. For each caller: read file, edited call site
7. **21 tool uses**

**Key observation:** The agent ran `scope callers` after `scope refs`, which was redundant (callers is a subset of refs). The new `callers` alias didn't save work — it added an extra tool call. However, the refs output with source lines meant the agent could verify completeness without a second grep pass.

### What the Blind C# Agent Did (Run 2)

1. Read `PaymentService.cs` to understand current signature
2. Read `PaymentRequest.cs` to check record structure
3. Used Grep for `ProcessPayment` across codebase
4. Read `IPaymentService.cs` and edited interface
5. For each caller: read file, edited
6. **18 tool uses** — fewest of all 4 agents

**Key observation:** The blind agent was the most efficient in tool use count. It followed a tight grep → read → edit loop with no wasted calls.

---

## Why the AX Improvements Didn't Widen the Gap

### 1. Source line snippets are informational, not actionable

The refs output now shows `this.paymentService.processPayment(amount, userId, card)` instead of `OrderController.checkout`. This tells the agent WHAT the call looks like — but the agent still needs to read the file to make the edit, because:
- It needs the full file context to construct the replacement (imports, variable names, surrounding code)
- Claude Code's Edit tool requires reading the file first
- The snippet is one line — the edit might span multiple lines

**Verdict:** Snippets help the agent confirm it found the right callers, but don't eliminate file reads.

### 2. Docstrings in sketch are irrelevant for refactoring tasks

The sketch now shows `/// Process a payment for the given amount and user.` above method signatures. For a signature refactoring task, the agent doesn't need to know WHAT the method does — it needs to know WHERE it's called. Docstrings help for Category E (discovery) tasks, not Category A (refactoring).

### 3. `scope callers` alias adds redundancy

The C# Scope agent called both `scope refs ProcessPayment` AND `scope callers ProcessPayment` — burning tokens on duplicate information. The alias is ergonomic but agents don't benefit from having two ways to get the same data.

### 4. Small codebase negates navigation advantage

At 14-18 files, a single `grep processPayment` finds everything. There's no navigational complexity. The Scope advantage only appears when:
- Grep returns too many results (noise in large codebases)
- The task requires understanding transitive impact (grep can't trace call chains)
- The agent needs structural context before searching (what does this class depend on?)

---

## Statistical Note

Two runs is not enough to establish significance. LLM non-determinism means each run can vary 20-30% in token usage depending on the agent's chosen navigation path. The TS blind agent was 19% more efficient in Run 2 than Run 1 — that's within normal variance.

**Aggregated across both runs:**

| Condition | Mean tokens (TS) | Mean tokens (C#) |
|-----------|-----------------|-----------------|
| With Scope | 29,348 | 28,658 |
| Without Scope | 28,857 | 31,070 |
| Delta | +1.7% (Scope WORSE) | -7.8% (Scope better) |

On TypeScript, the results are essentially noise. On C#, Scope shows a consistent ~8% advantage across both runs.

---

## Learnings for Phase 5 (Large Fixtures)

### What should change in the Scope-enabled agent prompt

1. **Don't instruct agents to run multiple scope commands upfront.** The Phase 4 agents ran sketch + refs + callers + impact — that's 4 commands before doing anything. On a small codebase, this wastes tokens. Instead: "Run `scope callers processPayment` to get all call sites, then edit each one."

2. **Emphasize that refs output includes the source line.** The agents didn't seem to leverage this — they still read each file before editing. The prompt should say: "The refs output shows the exact line of code at each call site. Use this to plan your edits before reading files."

3. **Skip `scope sketch` for refactoring tasks.** Sketch is for understanding a class you haven't seen. For "change this method signature," the agent already knows the target — go straight to `scope callers`.

### What should show larger gaps on 200+ file fixtures

1. **Grep becomes noisy.** On a 194-file TS codebase with 1,064 symbols, `grep processPayment` will match comments, strings, test descriptions, and type annotations — not just call sites. `scope callers` returns only actual callers.

2. **Navigation cost compounds.** Each grep match requires a file read to verify it's a real caller. On a 14-file codebase, that's 5-6 reads. On a 200-file codebase with grep noise, it could be 15-20 reads.

3. **Transitive impact becomes impossible with grep.** Category D tasks ("make findById async") require tracing callers of callers. Grep finds direct references but not the chain. `scope impact --depth 2` gives the full blast radius.

---

## Raw Data

### Run 2: TypeScript — With Scope (AX)

```
Agent ID:       a3e138d390859872d
Total tokens:   30,324
Tool uses:      23
Duration:       96,793ms
Callers found:  7/7
Files modified: 6
Scope commands: scope sketch PaymentService, scope refs processPayment
```

### Run 2: TypeScript — Without Scope

```
Agent ID:       a19a4b9d02af00af1
Total tokens:   25,891
Tool uses:      23
Duration:       84,549ms
Callers found:  7/7
Files modified: 6
Navigation:     Grep, Read, Glob
```

### Run 2: C# — With Scope (AX)

```
Agent ID:       aa75845610a48d409
Total tokens:   28,664
Tool uses:      21
Duration:       80,930ms
Callers found:  7/7
Files modified: 6
Scope commands: scope sketch, scope refs, scope callers, scope impact
```

### Run 2: C# — Without Scope

```
Agent ID:       a423e11210abcf930
Total tokens:   30,532
Tool uses:      18
Duration:       76,598ms
Callers found:  7/7
Files modified: 6
Navigation:     Grep, Read, Glob
```

---

## Conclusion

On small fixtures, AX improvements did not meaningfully change the token gap. The source line snippet feature works correctly but doesn't eliminate file reads — agents still need full file context to make edits. Docstrings help for understanding, not for refactoring.

The real test is Phase 5 on 200+ file fixtures, where:
- Grep noise increases dramatically
- File reads become expensive (more files to navigate)
- Transitive impact queries become necessary
- `scope callers` gives a precise caller list that grep can't match in one pass

These results are an honest baseline. The tool works; the question is scale.
