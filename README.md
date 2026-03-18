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
cargo install --git https://github.com/Rayne182/scope.git scope
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

## Stale Line Numbers

Line numbers in Scope output reflect the index state at the last `sc index` run. If files have been edited since then, line numbers may be off. This matters because agents use these line numbers for targeted edits.

**The workflow:**

1. Run `sc status` to check freshness before a session
2. If files have changed, run `sc index` to refresh (incremental -- takes < 1s for a few files)
3. After making edits during a session, `sc index` again before querying

In practice: run `sc index` whenever you switch tasks or after a batch of edits. It's cheap. The incremental indexer only re-parses changed files.

**For the CLAUDE.md snippet below**, consider adding: *"Run `sc index` before querying if you've made edits since the last index."*

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

**Keeping the index fresh:**
- Run `sc status` to check if the index is stale
- Run `sc index` after making edits -- it's incremental and takes < 1s for a few files
- Line numbers in Scope output reflect the last index run. If they seem off, re-index first.

Run `sc --help` for full usage.
```

The full snippet is also at [`docs/CLAUDE.md.snippet`](docs/CLAUDE.md.snippet).

---

## Built With

```
                 ┌─────────────────────────────────────────┐
                 │               sc binary                 │
                 │           (Rust, single binary)         │
                 └──────┬──────────┬──────────┬────────────┘
                        │          │          │
              ┌─────────▼──┐ ┌────▼─────┐ ┌──▼───────────┐
              │ tree-sitter │ │  SQLite  │ │ SQLite FTS5  │
              │   (parser)  │ │ (graph)  │ │   (search)   │
              └─────────────┘ └──────────┘ └──────────────┘
```

| Layer | Technology | Role |
|-------|-----------|------|
| **Language** | Rust | Single statically-linked binary. No runtime, no VM, no interpreter. `curl \| sh` and you're done. |
| **CLI** | [clap](https://crates.io/crates/clap) | Derive-based argument parsing with auto-generated `--help` written for LLM readers. |
| **Parsing** | [tree-sitter](https://tree-sitter.github.io/) | Incremental, error-tolerant AST parsing. Extracts symbols and relationships from source using `.scm` query files per language. |
| **Graph storage** | [SQLite](https://sqlite.org/) via [rusqlite](https://crates.io/crates/rusqlite) | Embedded dependency graph. Symbols and edges stored in tables, queried with recursive CTEs for transitive analysis (impact, deps). WAL mode, 64MB cache. |
| **Semantic search** | SQLite FTS5 | Full-text search over symbol names, signatures, and docstrings. BM25 ranking with porter stemming. CamelCase splitting so `processPayment` matches a search for "payment". |
| **File watching** | SHA-256 hashing | Incremental indexing compares file hashes against stored state. Only changed files are re-parsed. |
| **Output** | Human-readable + JSON | Every command supports `--json` for programmatic consumption. Human output designed for LLM token efficiency. |

### Design principles

- **No server, no network, no Docker.** A binary on disk. Works with any agent that can run a shell command.
- **Everything in `.scope/`.** The entire index lives in the project directory. No global state, no home directory pollution.
- **Offline-only.** Zero network calls at runtime. No telemetry, no updates, no API keys (unless you opt into an external embedding provider later).
- **Output is the API.** LLMs infer the data model from output format. Every output change is treated as a breaking change.

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

## Benchmarks

Scope ships with a benchmark harness that measures whether it actually reduces token consumption for coding agents. The harness runs real coding tasks with and without Scope enabled, comparing token usage, correctness, and navigation efficiency.

### What gets measured

Three metrics, always reported together:

1. **Input token consumption** -- total tokens the agent consumed across the task
2. **Task correctness** -- compilation pass, tests pass, caller coverage score
3. **Navigation efficiency** -- number of full source file reads (fewer = better)

### Task suite

20 tasks across 5 categories, for both TypeScript and C#:

| Category | What it tests | Example |
|----------|--------------|---------|
| Signature refactoring | Find and update all callers | Refactor `processPayment` to accept a `PaymentRequest` object |
| Cross-cutting changes | Breadth of navigation | Add structured logging to all public methods in a service |
| Dependency understanding | Comprehend what a class needs | Replace `PaymentProcessor` with a new `PaymentGateway` |
| Impact-aware refactoring | Get the blast radius right | Make `findById` async and update everything downstream |
| Discovery | Find code by intent | Find where payment retry logic lives and add exponential backoff |

### Fixtures

Two benchmark fixtures with known, deterministic dependency graphs:

- `benchmarks/fixtures/typescript-api/` -- 20-symbol TypeScript project. `processPayment` has exactly 7 callers across 4 files.
- `benchmarks/fixtures/csharp-api/` -- 20-symbol .NET 8 Web API. Same ground truth structure, C# idioms (interfaces, DI, async/await).

### Running the benchmarks

```sh
# Build the runner
cd benchmarks/runner
cargo build --release

# Run all tasks, comparing Scope-enabled vs baseline
cargo run --release -- run --all --compare --reps 5

# Run a single task
cargo run --release -- run --task ts-cat-a-01 --compare

# Run only TypeScript tasks
cargo run --release -- run --language typescript --compare

# Run only C# tasks
cargo run --release -- run --language csharp --compare

# Generate a report from existing results
cargo run --release -- report --input ../../benchmarks/results/v0.1.0/ --output markdown
```

**Prerequisites:** The runner invokes Claude Code (`claude --print`) to execute tasks. You need:
- Claude Code installed (`npm install -g @anthropic-ai/claude-code`)
- `ANTHROPIC_API_KEY` set in your environment
- The Scope binary (`sc`) on your PATH

### Results format

Results are committed per release in `benchmarks/results/vX.Y.Z/`:

```
benchmarks/results/v0.1.0/
  full_results.json      # raw data -- every run, every metric
  summary.md             # human-readable tables for release notes
  environment.json       # machine spec, tool versions, rep count
```

### Adding your own tasks

Create a TOML file in `benchmarks/tasks/<language>/`:

```toml
[task]
id = "ts-custom-01"
category = "signature-refactoring"
language = "typescript"
corpus = "fixture"
description = "Your task description"

[prompt]
text = """
The exact prompt sent to the coding agent.
"""

[target]
symbol = "PaymentService.processPayment"
file = "src/payments/service.ts"

[correctness]
require_compilation = true
require_tests_pass = true
require_caller_coverage = true
caller_coverage_threshold = 1.0

[scope]
expected_commands = ["sc refs", "sc sketch"]
```

Then run it: `cargo run -- run --task ts-custom-01 --compare`

---

## License

MIT
