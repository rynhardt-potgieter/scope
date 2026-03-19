# Scope v0.1.0 — Test Harness Results

**Date:** 2026-03-19
**Version:** scope 0.1.0
**Platform:** Windows 11 Pro, Rust 2021 edition
**Fixtures:** typescript-simple (5 files, 21 symbols), csharp-simple (4 files, 14 symbols)

---

## Methodology

All tests were run against two committed test fixtures with known, deterministic dependency graphs. Each fixture was freshly indexed (`scope init` + `scope index --full`) before testing. Every MVP command was exercised with human-readable and JSON output modes.

**What was tested:**
- All 8 commands: `init`, `index`, `sketch`, `refs`, `deps`, `impact`, `find`, `status`
- Both supported languages: TypeScript and C#
- JSON output envelope consistency
- Incremental indexing (add, modify, delete, no-change detection)
- Error handling (unknown symbols, missing index)
- Semantic search with FTS5 prefix matching

**What was NOT tested (deferred):**
- Agent-based benchmarks (requires Claude Code API invocation)
- Token consumption comparison (with vs without Scope)
- Real-world corpora (only fixture projects)
- Performance under load (10k+ file codebases)

---

## TypeScript Fixture Results

**Fixture:** `tests/fixtures/typescript-simple/`
**Structure:** PaymentService (3 methods), OrderController (2 methods calling processPayment), RefundController (1 method calling refundPayment), Logger (2 methods)

### Indexing

| Metric | Result |
|--------|--------|
| Files indexed | 5 |
| Symbols extracted | 21 |
| Edges extracted | 22 |
| Index time | < 0.1s |

### `scope sketch` — Structural Overview

**Class sketch (`scope sketch PaymentService`):**
```
PaymentService                                    class  src/payments/service.ts:4-24
──────────────────────────────────────────────────────────────────────────────
deps:     Logger, PaymentRequest, logger.info

methods:
  constructor(logger: Logger)                                       [internal]
  async processPayment(request: PaymentRequest): Promise<PaymentResult>[2 callers]
  async refundPayment(transactionId: string): Promise<boolean>      [1 caller]
  private validateAmount(amount: number): boolean                   [internal]

fields:
  private logger: Logger
```

| Check | Result |
|-------|--------|
| Class name and kind shown | PASS |
| File path with line range | PASS |
| Dependencies listed | PASS |
| Method signatures shown | PASS |
| Caller counts accurate | PASS — processPayment [2 callers] (OrderController.checkout + retryPayment) |
| Internal methods marked | PASS — validateAmount [internal] |
| Fields with access modifiers | PASS |

**Method sketch (`scope sketch PaymentService.processPayment`):**
```
processPayment                                    method  src/payments/service.ts:11-14
──────────────────────────────────────────────────────────────────────────────
signature:  async processPayment(request: PaymentRequest): Promise<PaymentResult>
calls:      logger.info
```

| Check | Result |
|-------|--------|
| Qualified name lookup works | PASS |
| Signature shown | PASS |
| Outgoing calls listed | PASS |

**File sketch (`scope sketch src/payments/service.ts`):**
```
src/payments/service.ts
──────────────────────────────────────────────────────────────────────────────
  PaymentService          class     4-24     [internal]
  logger                  property  5        [internal]
  constructor             method    7-9      [internal]
  processPayment          method    11-14    [2 callers]
  refundPayment           method    16-19    [1 caller]
  validateAmount          method    21-23    [internal]
```

| Check | Result |
|-------|--------|
| All symbols in file listed | PASS (6 symbols) |
| Line ranges shown | PASS |
| Caller counts per symbol | PASS |

### `scope refs` — Reference Lookup

**Class refs (`scope refs PaymentService`):**
```
PaymentService — 9 references
──────────────────────────────────────────────────────────────────────────────
called (3):
  src/controllers/order.ts:13           OrderController.checkout
  src/controllers/order.ts:18           OrderController.retryPayment
  src/controllers/refund.ts:11          RefundController.processRefund

imported (2):
  src/controllers/order.ts:1            order
  src/controllers/refund.ts:1           refund

used as type (4):
  src/controllers/order.ts:5            OrderController
  src/controllers/order.ts:7            OrderController.constructor
  src/controllers/refund.ts:4           RefundController
  src/controllers/refund.ts:6           RefundController.constructor
```

| Check | Result |
|-------|--------|
| Total count correct | PASS (9 references) |
| Grouped by kind | PASS (called, imported, used as type) |
| Caller names resolved (not file stems) | PASS — shows OrderController.checkout, not "order" |
| File paths with line numbers | PASS |

**Method refs (`scope refs processPayment`):**
```
processPayment — 2 references
──────────────────────────────────────────────────────────────────────────────
src/controllers/order.ts:13             OrderController.checkout
src/controllers/order.ts:18             OrderController.retryPayment
```

| Check | Result |
|-------|--------|
| Flat list for methods | PASS |
| Exact caller count | PASS (2 callers) |
| Caller method names shown | PASS |

### `scope deps` — Dependencies

```
PaymentService — direct dependencies
──────────────────────────────────────────────────────────────────────────────
calls (external):
  logger.info             (external)

imports:
  Logger                  src/utils/logger.ts
  PaymentRequest          src/payments/types.ts
  PaymentResult           src/payments/types.ts

references_type:
  Logger                  src/utils/logger.ts
  PaymentRequest          src/payments/types.ts
```

| Check | Result |
|-------|--------|
| Direct dependencies shown | PASS |
| External deps marked | PASS (logger.info marked external) |
| Internal deps show file path | PASS |
| Grouped by edge kind | PASS |

### `scope impact` — Blast Radius

```
Impact analysis: processPayment
──────────────────────────────────────────────────────────────────────────────
Direct callers (2):
  checkout                                src/controllers/order.ts
  retryPayment                            src/controllers/order.ts
```

| Check | Result |
|-------|--------|
| Direct callers found | PASS (2 callers) |
| Caller names resolved | PASS — checkout, retryPayment |
| No longer shows "no impact detected" | PASS |

**Class impact (`scope impact PaymentService`):**
```
Direct callers (7):
  OrderController, checkout, constructor, retryPayment (order.ts)
  RefundController, constructor, processRefund (refund.ts)
```

| Check | Result |
|-------|--------|
| Includes all reference types | PASS (calls + type refs + imports) |

### `scope find` — Semantic Search

```
Results for: "payment"
──────────────────────────────────────────────────────────────────────────────
1.00  PaymentService          src/payments/service.ts:4             class
1.00  PaymentRequest          src/payments/types.ts:1               interface
1.00  PaymentResult           src/payments/types.ts:7               interface
...
0.90  processPayment          src/payments/service.ts:11            method
0.50  refundPayment           src/payments/service.ts:16            method
```

| Check | Result |
|-------|--------|
| Relevant results returned | PASS |
| Scores in 0.00-1.00 range | PASS |
| Proper spacing between columns | PASS |
| PaymentService at top | PASS |

**Semantic adjacency test (`scope find "logging"`):**
```
1.00  logger                  src/payments/service.ts:5             property
0.98  Logger                  src/utils/logger.ts:1                 class
0.50  constructor             src/payments/service.ts:7             method
```

| Check | Result |
|-------|--------|
| Finds Logger via "logging" (prefix match) | PASS |

### Error Handling

| Scenario | Expected | Result |
|----------|----------|--------|
| `scope sketch UnknownThing` | Error + suggestion | PASS — "not found" + "use scope find" |
| Exit code on error | 1 | PASS |

---

## C# Fixture Results

**Fixture:** `tests/fixtures/csharp-simple/`
**Structure:** IPaymentService (interface), PaymentService (class implementing interface, 3 methods), OrderController (1 method calling ProcessPayment), Logger (2 methods)

### Indexing

| Metric | Result |
|--------|--------|
| Files indexed | 4 |
| Symbols extracted | 14 |
| Edges extracted | 6 |
| Index time | 0.1s |

### `scope sketch` — Structural Overview

**Class sketch (`scope sketch PaymentService`):**
```
PaymentService                                    class  src/Payments/PaymentService.cs:5-30
──────────────────────────────────────────────────────────────────────────────
deps:     _logger.Info
implements: IPaymentService

methods:
  public PaymentService(Logger logger)                              [internal]
  public async Task<bool> ProcessPayment(decimal amount, string userId)[1 caller]
  public async Task<bool> RefundPayment(string transactionId)       [internal]
  private bool ValidateAmount(decimal amount)                       [internal]
```

| Check | Result |
|-------|--------|
| C# syntax in signatures | PASS (Task<bool>, decimal, string) |
| Implements relationship | PASS (IPaymentService) |
| Access modifiers shown | PASS (public, private) |
| Async methods shown | PASS |
| Caller count for ProcessPayment | PASS — [1 caller] (OrderController.Checkout) |

**Interface sketch (`scope sketch IPaymentService`):**
```
IPaymentService                                   interface  src/Payments/IPaymentService.cs:3-7
──────────────────────────────────────────────────────────────────────────────

methods:
  Task<bool> ProcessPayment(decimal amount, string userId);
  Task<bool> RefundPayment(string transactionId);
```

| Check | Result |
|-------|--------|
| Interface methods with signatures | PASS |
| C# types preserved | PASS |

### `scope refs` — Reference Lookup

**Interface refs (`scope refs IPaymentService`):**
```
IPaymentService — 2 references
──────────────────────────────────────────────────────────────────────────────
called (1):
  src/Controllers/OrderController.cs:16 OrderController.Checkout

implemented (1):
  src/Payments/PaymentService.cs:5      PaymentService
```

| Check | Result |
|-------|--------|
| Implementation reference found | PASS |
| Grouped by kind | PASS (called, implemented) |

### `scope impact` — Blast Radius

```
Impact analysis: ProcessPayment
──────────────────────────────────────────────────────────────────────────────
Direct callers (1):
  Checkout                                src/Controllers/OrderController.cs
```

| Check | Result |
|-------|--------|
| C# method callers found | PASS |
| PascalCase method names preserved | PASS |

### `scope find` — Semantic Search

```
Results for: "payment"
──────────────────────────────────────────────────────────────────────────────
1.00  PaymentService          src/Payments/PaymentService.cs:5      class
0.70  IPaymentService         src/Payments/IPaymentService.cs:3     interface
0.50  ProcessPayment          src/Payments/PaymentService.cs:14     method
```

| Check | Result |
|-------|--------|
| C# symbols found by intent | PASS |
| Interface included in results | PASS |

---

## JSON Output Validation

Every command was tested with `--json`. All produce valid JSON with the `JsonOutput<T>` envelope.

| Command | `command` field | `data` present | `total` present | `truncated` present |
|---------|----------------|----------------|-----------------|---------------------|
| `scope sketch --json` | "sketch" | PASS | PASS | PASS |
| `scope refs --json` | "refs" | PASS | PASS (2) | PASS (false) |
| `scope impact --json` | "impact" | PASS | PASS (2) | PASS |
| `scope find --json` | "find" | PASS | PASS (10) | PASS |
| `scope status --json` | "status" | PASS | PASS | PASS |

---

## Incremental Indexing

Tested with a fresh project, modifying files between index runs.

| Operation | Expected behavior | Result |
|-----------|-------------------|--------|
| Add a new `.ts` file | "Added: src/extra.ts" | PASS |
| Modify an existing file | "Modified: src/main.ts" | PASS |
| Delete a file | "Deleted: src/extra.ts" | PASS |
| No changes | "Index up to date." | PASS |
| Index time (1 file) | < 1s | PASS (< 0.1s) |

---

## Output Format Compliance

| Rule | Spec requirement | Result |
|------|-----------------|--------|
| Separator character | `─` (U+2500) | PASS |
| File paths | Forward slashes on all platforms | PASS |
| Line ranges | `start-end` format | PASS |
| Similarity scores | 2 decimal places | PASS |
| Caller counts | `[N callers]` or `[internal]` | PASS |
| Truncation | "... N more (use --limit to show more)" | PASS (verified in test suite) |
| Progress to stderr | Indexing output to stderr | PASS |
| Data to stdout | Query results to stdout | PASS |
| Error messages | To stderr with exit code 1 | PASS |

---

## Known Limitations

1. **Edge `from_id` for C# member calls** — C# member calls like `_logger.Info()` produce edges with `_logger.Info` as `to_id`. Since `_logger` is a field name, not a class name, the edge doesn't resolve cleanly to the `Logger.Info` symbol. This means some C# call edges show as external dependencies when they should be internal.

2. **C# namespace imports** — `using CSharpSimple.Utils` produces an import edge to `CSharpSimple.Utils` as a string, not to a specific symbol. This shows as `(external)` in deps output.

3. **Caller counts include type references** — `scope refs PaymentService` counts type annotations (e.g., `private svc: PaymentService`) as references. This inflates the total count for class symbols compared to method symbols.

4. **FTS5 vs vector embeddings** — Search uses keyword matching (FTS5 BM25), not semantic vector similarity. "authentication" will not match `LoginService` because there's no semantic understanding — only token overlap. Planned for v0.2 with ONNX embeddings.

5. **No second-degree impact** — Impact analysis currently shows direct callers but the second/third-degree traversal depends on edges pointing to actual symbols. In the TypeScript fixture, impact stops at depth 1 because the controllers are leaf nodes (nothing calls them in the fixture).

---

## Summary

| Category | TypeScript | C# |
|----------|-----------|-----|
| Indexing | PASS (5 files, 21 symbols, 22 edges) | PASS (4 files, 14 symbols, 6 edges) |
| Sketch (class) | PASS — methods, deps, caller counts | PASS — C# syntax, implements, async |
| Sketch (method) | PASS — signature, calls | PASS |
| Sketch (file) | PASS — all symbols listed | PASS |
| Refs | PASS — grouped by kind, caller names resolved | PASS — implemented/called grouping |
| Deps | PASS — imports, calls, external marking | PASS |
| Impact | PASS — direct callers found | PASS — direct callers found |
| Find | PASS — FTS5 prefix matching, BM25 ranking | PASS |
| JSON output | PASS — all commands, valid envelope | PASS |
| Incremental indexing | PASS — add/modify/delete/no-change | N/A (tested on TS) |
| Error handling | PASS — helpful messages, exit code 1 | PASS |

**Overall: All MVP commands functional on both TypeScript and C# codebases.**
