# Scope

**Give your coding agent structural awareness instead of dumping entire files into context.**

Scope (`sc`) is a CLI that builds a local code intelligence index and exposes it through fast, token-efficient queries. Instead of an agent reading 8,000 tokens of source to understand a class, Scope returns a 200-token structural sketch with dependencies, callers, and method signatures.

```
$ sc sketch PaymentService

PaymentService                                    class  src/payments/service.ts:12
─────────────────────────────────────────────────────────────────────────────────
deps:     StripeClient, UserRepository, Logger, PaymentConfig
extends:  BaseService
implements: IPaymentService

methods:
  processPayment(amount: Decimal, userId: string) → PaymentResult       [11 callers]
  refundPayment(transactionId: string, reason?: string) → Promise<bool>  [3 callers]
  validateCard(card: CardDetails) → ValidationResult                     [internal]
  getTransaction(id: string) → Transaction | null                        [2 callers]

fields:
  private client: StripeClient
  private repo: UserRepository
  private logger: Logger
```

The agent now knows what `PaymentService` depends on, which methods are heavily called, and which are internal -- without reading a single line of implementation.

---

## The Problem

LLM coding agents navigate codebases by reading files. Every file read costs tokens. A typical agent task -- "change the signature of `processPayment`" -- might require reading 6-10 files just to find all callers, understand the dependency graph, and assess blast radius. That's 20,000-50,000 tokens of context burned on navigation, not on the actual edit.

Scope gives agents the same structural understanding in a fraction of the tokens:

| Without Scope | With Scope |
|---------------|------------|
| Read `service.ts` (4,200 tokens) | `sc sketch PaymentService` (200 tokens) |
| Read `order.ts` (3,100 tokens) | `sc refs processPayment` (150 tokens) |
| Read `subscription.ts` (2,800 tokens) | `sc impact processPayment` (180 tokens) |
| Grep for callers (1,500 tokens) | |
| **~11,600 tokens** | **~530 tokens** |

That's a 95% reduction on navigation. The agent still reads full source when it's ready to edit -- but only the exact files it needs, informed by Scope's structural queries.

---

## Install

**Binary (recommended):**

```sh
curl -fsSL https://raw.githubusercontent.com/Rayne182/scope/main/install.sh | sh
```

**From source:**

```sh
cargo install --git https://github.com/Rayne182/scope.git
```

---

## 60-Second Quick Start

```sh
cd your-project
sc init              # detects languages, creates .scope/
sc index             # builds the index (incremental after first run)
```

Now query:

```sh
sc sketch UserService               # what does this class look like?
sc refs processPayment               # who calls this? (before changing a signature)
sc deps PaymentService               # what does this depend on?
sc impact processPayment             # what breaks if I change this?
sc find "payment retry logic"        # find code by intent, not by name
sc status                            # is my index fresh?
```

Every command supports `--json` for programmatic consumption.

---

## Commands

### `sc sketch` -- Structural overview

The most important command. Returns the shape of a symbol without loading source.

```sh
sc sketch PaymentService                 # class overview
sc sketch PaymentService.processPayment  # method detail
sc sketch src/payments/service.ts        # file-level summary
```

Method detail shows the call graph in both directions:

```
processPayment                        method  src/payments/service.ts:34-67
──────────────────────────────────────────────────────────────────────────
signature:  (amount: Decimal, userId: string) → PaymentResult
calls:      validateCard, repo.findUser, client.charge, logger.info
called by:  OrderController.checkout [x3], SubscriptionService.renew [x8]
```

### `sc refs` -- Find all references

The thing to run before changing any signature. Shows every call site across the codebase.

```sh
sc refs processPayment               # all 11 callers
sc refs PaymentService --kind calls   # filter by reference kind
sc refs PaymentService --limit 5      # truncate output
```

For classes, refs are grouped by kind (instantiated, extended, imported, used as type).

### `sc deps` -- Dependencies

What does this symbol depend on?

```sh
sc deps PaymentService               # direct dependencies
sc deps PaymentService --depth 2     # transitive dependencies
sc deps src/payments/service.ts      # file-level dependencies
```

### `sc impact` -- Blast radius

Transitive reverse dependency traversal. Shows what breaks if you change something.

```sh
sc impact processPayment             # who's affected?
sc impact PaymentConfig              # what if config shape changes?
sc impact src/types/payment.ts       # impact of changing a types file
```

Groups results by depth: direct callers, second-degree, third-degree. Test files shown separately.

### `sc find` -- Semantic search

Find code by what it does, not what it's called.

```sh
sc find "payment retry logic"
sc find "handles authentication errors" --kind method
```

### `sc status` -- Index health

```sh
sc status
# Index status: up to date
#   Symbols:    6,764
#   Files:      847
#   Last index: 2 minutes ago
```

---

## Add to Your Project's CLAUDE.md

Paste this into your `CLAUDE.md` so Claude Code uses Scope automatically:

```markdown
## Code Navigation

This project uses Scope for structural code navigation. Use it before reading or
editing any non-trivial code.

**Before editing a class or function:**
- `sc sketch <ClassName>` -- structural overview without reading full source
- `sc refs <symbol>` -- find all callers before changing a signature
- `sc impact <symbol>` -- check blast radius before any refactor

**Finding code:**
- `sc find "<description>"` -- find relevant code by intent

**Understanding dependencies:**
- `sc deps <symbol>` -- what does this depend on?

Always run `sc sketch` before reading full source. Only read implementation when
you know exactly which file and line range you need.

Run `sc --help` for full usage.
```

The full snippet is also at [`docs/CLAUDE.md.snippet`](docs/CLAUDE.md.snippet).

---

## How It Works

Scope builds a local index in `.scope/` using:

- **tree-sitter** for AST parsing -- extracts symbols (classes, methods, interfaces, functions) and relationships (calls, imports, extends, implements)
- **SQLite** for the dependency graph -- symbols and edges, queried with recursive CTEs for transitive analysis
- **SQLite FTS5** for semantic search -- indexes symbol names, signatures, and docstrings with BM25 ranking

Indexing is incremental by default. After the first full build, `sc index` only re-processes changed files using SHA-256 hash comparison. Re-indexing a single file completes in under a second.

Everything runs locally. No network calls, no server process, no API keys required.

---

## Supported Languages

| Language | Status |
|----------|--------|
| TypeScript / JavaScript | Supported |
| C# | Supported |
| Python | Planned |
| Go | Planned |
| Java | Planned |
| Rust | Planned |

---

## Performance

These are acceptance criteria, not aspirational targets. A command that misses its target is a bug.

| Operation | Target |
|-----------|--------|
| `sc sketch` | < 100ms |
| `sc refs` | < 100ms |
| `sc deps` | < 100ms |
| `sc impact` | < 500ms |
| `sc find` | < 500ms |
| `sc status` | < 50ms |
| Incremental index (1 file) | < 1s |
| Full index (10k files) | < 60s |

---

## License

MIT
