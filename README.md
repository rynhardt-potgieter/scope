```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   ███████╗ ██████╗ ██████╗ ██████╗ ███████╗                 │
│   ██╔════╝██╔════╝██╔═══██╗██╔══██╗██╔════╝                 │
│   ███████╗██║     ██║   ██║██████╔╝█████╗                   │
│   ╚════██║██║     ██║   ██║██╔═══╝ ██╔══╝                   │
│   ███████║╚██████╗╚██████╔╝██║     ███████╗                 │
│   ╚══════╝ ╚═════╝ ╚═════╝ ╚═╝     ╚══════╝                 │
│                                                             │
│   Code intelligence for LLM coding agents.                  │
│   Know before you touch.                                    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

[![Rust](https://img.shields.io/badge/built_with-Rust-orange?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-v0.5.3-blue.svg)](https://github.com/rynhardt-potgieter/scope/releases)
[![Build](https://img.shields.io/github/actions/workflow/status/rynhardt-potgieter/scope/ci.yml?label=build)](https://github.com/rynhardt-potgieter/scope/actions)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey)](#installation)
[![Stars](https://img.shields.io/github/stars/rynhardt-potgieter/scope?style=flat)](https://github.com/rynhardt-potgieter/scope/stargazers)
[![Issues](https://img.shields.io/github/issues/rynhardt-potgieter/scope)](https://github.com/rynhardt-potgieter/scope/issues)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

---

## Table of contents

- [What it does](#what-it-does)
- [Supported languages](#supported-languages)
- [Installation](#installation)
- [Quick start](#quick-start)
- [Commands](#commands)
- [How it works](#how-it-works)
- [Configuration](#configuration)
- [CLAUDE.md integration](#claudemd-integration)
- [Building from source](#building-from-source)
- [Benchmark methodology](#benchmark-methodology)
- [Roadmap](#roadmap)
- [License](#license)

---

## What it does

Scope builds a local code intelligence index for any codebase and exposes it through a CLI designed for LLM coding agents. Before an agent edits a function, it can run `scope sketch PaymentService` and get back the class structure, method signatures, caller counts, and dependency surface in approximately 180 tokens -- without reading the 6,000-token source file.

The index is built from tree-sitter AST parsing (fast, error-tolerant, no compiler required), stored in a SQLite dependency graph with FTS5 full-text search, and queried through commands that return structured, agent-readable output. Everything lives in a `.scope/` directory in your project root. No server process, no Docker, no API key required.

Scope integrates with Claude Code, Cursor, Aider, and any other agent that can run a shell command. Add the provided CLAUDE.md snippet to your project and agents will use it automatically.

```
$ scope sketch PaymentService

PaymentService                                    class  src/payments/service.ts:12-89
─────────────────────────────────────────────────────────────────────────────────────
deps:      StripeClient, UserRepository, Logger, PaymentConfig
extends:   BaseService
implements: IPaymentService

methods:
  async  processPayment  (amount: Decimal, userId: string) → Promise<PaymentResult>   [11 callers]
         refundPayment   (txId: string, reason?: string)   → Promise<bool>             [3 callers]
  private validateCard   (card: CardDetails)               → ValidationResult          [internal]
         getTransaction  (id: string)                      → Transaction | null        [2 callers]

fields:
  private readonly  client  : StripeClient
  private           repo    : UserRepository
  private           logger  : Logger

// ~180 tokens  ·  source file is 6,200 tokens
```

---

## Supported languages

### Production-ready

![TypeScript](https://img.shields.io/badge/TypeScript-ready-22863a?style=flat-square&logo=typescript&logoColor=white)
![C#](https://img.shields.io/badge/C%23-ready-22863a?style=flat-square&logo=csharp&logoColor=white)

Both languages have full support: tree-sitter grammar integration, symbol extraction, edge detection (calls, imports, extends, implements), and enriched method modifiers (async, static, private, abstract, virtual, override). C# includes partial class merging across files.

### Planned

![Python](https://img.shields.io/badge/Python-planned-e6a817?style=flat-square&logo=python&logoColor=white)
![Go](https://img.shields.io/badge/Go-planned-e6a817?style=flat-square&logo=go&logoColor=white)
![Java](https://img.shields.io/badge/Java-planned-e6a817?style=flat-square&logo=openjdk&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-planned-e6a817?style=flat-square&logo=rust&logoColor=white)
![Kotlin](https://img.shields.io/badge/Kotlin-planned-6e7681?style=flat-square&logo=kotlin&logoColor=white)
![Ruby](https://img.shields.io/badge/Ruby-planned-6e7681?style=flat-square&logo=ruby&logoColor=white)

Language support means: tree-sitter grammar integrated, symbol extraction tested, and edge detection (calls, imports, extends, implements) working correctly for that language's idioms.

---

## Installation

### curl (Linux and macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/rynhardt-potgieter/scope/main/install.sh | sh
```

### cargo

```bash
cargo install --git https://github.com/rynhardt-potgieter/scope.git scope
```

After installation, verify with:

```bash
scope --version
# scope 0.5.3
```

> **Windows PowerShell note:** The binary is named `scope` -- no conflicts with PowerShell aliases.

---

## Quick start

### 1. Initialise

Run once from your project root. Detects languages and writes a default `.scope/config.toml`.

```bash
scope init
```

```
Initialised .scope/ for project: api
Detected languages: TypeScript
Run 'scope index' to build the index.
```

### 2. Build the index

First run indexes the full codebase. Subsequent runs are incremental -- only changed files are re-indexed.

```bash
scope index
```

```
Indexing 847 files...
  TypeScript   612 files   4,821 symbols
  C#           235 files   1,943 symbols
Built in 12.4s. Index size: 8.2MB
```

### 3. Explore the codebase

Start with the high-level overview, then drill down.

```bash
scope map                                # full repo overview (~500-1000 tokens)
scope entrypoints                        # API controllers, workers, event handlers
scope sketch PaymentService              # structural overview of a class
scope refs processPayment                # find all callers
scope callers processPayment --depth 2   # transitive callers
scope trace processPayment               # entry-point-to-symbol call paths
scope deps PaymentService                # what does it depend on?
scope find "payment retry logic"         # semantic search
scope status                             # is my index fresh?
```

### 4. Keep the index fresh

Line numbers in Scope output reflect the last `scope index` run. If you've edited files since then, re-index before querying:

```bash
scope status                             # check freshness
scope index                              # incremental -- < 1s for a few files
```

Run `scope index` whenever you switch tasks or after a batch of edits. It's cheap.

### 5. Add to CLAUDE.md

Paste the [CLAUDE.md snippet](#claudemd-integration) into your project so agents use Scope automatically.

---

## Commands

| Command | Signature | Description | When to use |
|---|---|---|---|
| `scope init` | `[--json]` | Initialise Scope for a project. Creates `.scope/` with default config, auto-detects languages. | Once per project, before first `scope index`. |
| `scope index` | `[--full] [--json]` | Build or refresh the code index. Incremental by default, `--full` forces complete rebuild. | Once on setup, then after edits during development. |
| `scope map` | `[--limit N] [--json]` | Full repository overview: entry points, core symbols ranked by caller count, architecture summary. ~500-1000 tokens. | First thing in a new codebase. Replaces 5-17 sketch calls for orientation. |
| `scope entrypoints` | `[--json]` | Lists API controllers, workers, and event handlers grouped by type. Symbols with zero incoming call edges. | Understanding the request flow starting points. |
| `scope sketch` | `<symbol> [--json]` | Compressed structural overview: methods with caller counts, dependencies, type signatures, modifiers. ~200 tokens vs ~4,000 for full source. | Before reading source or editing any non-trivial symbol. |
| `scope refs` | `<symbol> [--kind calls\|imports\|extends] [--limit N] [--json]` | All references grouped by kind: call sites, imports, type annotations. Includes source line snippets. | Before changing a function signature or deleting a symbol. |
| `scope callers` | `<symbol> [--depth N] [--context N] [--json]` | Direct callers (depth 1) or transitive callers (depth 2+). Depth 1 shows snippets; depth 2+ groups by level with test file separation. | Before any refactor that changes a public API surface. |
| `scope deps` | `<symbol> [--depth 1-3] [--json]` | What does this symbol depend on? Direct imports, calls, extended classes. Transitive with `--depth`. | Understanding prerequisites before implementing something new. |
| `scope rdeps` | `<symbol> [--depth 1-3] [--json]` | What depends on this symbol? Reverse dependency traversal. | Before deleting or renaming a symbol. |
| `scope impact` | `<symbol> [--depth 1-5] [--json]` | *Deprecated* -- delegates to `scope callers --depth N`. Blast radius analysis. | Use `scope callers --depth N` instead. |
| `scope trace` | `<symbol> [--limit N] [--json]` | Trace call paths from entry points to a symbol. Shows how API endpoints and workers reach a function. | Understanding how a bug is triggered or what code paths exercise a function. |
| `scope find` | `"<query>" [--kind function\|class] [--limit N] [--json]` | Full-text search with BM25 ranking, importance-boosted results. CamelCase and snake_case aware. | Navigating an unfamiliar codebase or finding code by intent. |
| `scope similar` | `<symbol> [--kind function\|class] [--json]` | *Stub* -- find structurally similar symbols. Not yet implemented. | Future: discovering existing implementations. |
| `scope source` | `<symbol> [--json]` | *Stub* -- fetch full source of a symbol. Not yet implemented. | Future: reading implementation after `scope sketch`. |
| `scope status` | `[--json]` | Index health: symbol count, file count, last indexed time, stale files. | Checking whether the index is stale before making range-based edits. |

All commands support `--json` for structured output. Line numbers reflect the state of the index at last run -- use `scope status` to check freshness before range-based edits.

---

## How it works

```
Your codebase
      |
      v
+-----------------------------+
|  tree-sitter parser          |  Fast, incremental, error-tolerant AST parsing.
|  (TypeScript, C#, ...)       |  No compiler required. Extracts symbols, types,
+--------------+--------------+  modifiers, docstrings, line ranges.
               |
       +-------+--------+
       v                v
+----------+    +--------------+
|  SQLite  |    | SQLite FTS5  |  Two complementary indexes:
|  graph   |    |   search     |  SQLite for structural relationships (who calls
|          |    |              |  what, inheritance chains, import graphs).
| symbols  |    | BM25-ranked  |  FTS5 for full-text search by intent -- finds
| + edges  |    | symbol text  |  symbols by what they do, not what they're named.
+----------+    +--------------+  Both embedded on disk -- no server.
       |                |
       +-------+--------+
               v
+-----------------------------+
|  scope query engine         |  Combines structural traversal and text search.
|                             |  Returns labelled, token-efficient output designed
+-----------------------------+  for LLM consumption, not human readability.
```

**Parsing** -- tree-sitter produces a concrete syntax tree for every file. Scope extracts symbols (functions, classes, methods, interfaces, enums, types) with their signatures, type annotations, access modifiers, async status, and docstrings. For C#, partial classes are merged across files before indexing.

**Structural graph** -- symbols are nodes in a SQLite database. Edges represent relationships: `calls`, `imports`, `extends`, `implements`, `instantiates`, `references_type`. Impact analysis uses recursive common table expressions to traverse this graph to arbitrary depth.

**Full-text search** -- symbol names (with CamelCase splitting), signatures, docstrings, caller/callee names, and file path components are indexed in an FTS5 virtual table with porter stemming. Symbols with more callers rank higher via importance-tier boosting. `scope find "payment"` matches `processPayment`, `PaymentService`, and any symbol with "payment" in its docstring. Scores are normalized to 0.00-1.00 and sorted by BM25 relevance.

**Incremental indexing** -- each file's SHA-256 hash is stored alongside its symbols. On re-index, only files whose hash has changed are re-parsed. A single changed file re-indexes in under one second.

---

## Configuration

Scope reads `.scope/config.toml` in the project root. This file is created by `scope init` and is safe to commit to version control.

```toml
[project]
name = "api"
languages = ["typescript", "csharp"]

[index]
# Patterns to exclude from indexing (respects .gitignore by default)
ignore = [
  "node_modules",
  "dist",
  "build",
  ".git",
  "migrations/",
]

# Include test files in refs and impact output
include_tests = true

[embeddings]
# "local" uses SQLite FTS5 -- no API key, works offline
provider = "local"

[output]
# Truncate long reference lists (use --limit to override per command)
max_refs    = 20
# Maximum depth for impact traversal (use --depth to override)
max_depth   = 3
```

---

## CLAUDE.md integration

Add the following to your project's `CLAUDE.md`. Claude Code reads this at the start of every session.

```markdown
## Code Navigation

This project uses [Scope](https://github.com/rynhardt-potgieter/scope) for structural code intelligence.
Start with `scope map` for a repo overview, then `scope sketch` for specific symbols.

**Orientation:**
- `scope map` -- full repo overview: entry points, core symbols, architecture (~500-1000 tokens)
- `scope entrypoints` -- list API controllers, workers, event handlers
- `scope status` -- check index health and freshness

**Before editing a class or function:**
- `scope sketch <symbol>` -- structural overview: methods, deps, modifiers (~200 tokens)
- `scope refs <symbol> [--kind calls|imports|extends|implements|...]` -- all references with file + line
- `scope callers <symbol> [--depth N]` -- direct and transitive callers for blast radius

**Finding code:**
- `scope find "<query>" [--kind function|class|method|interface]` -- full-text search by intent

**Understanding dependencies and flow:**
- `scope deps <symbol> [--depth 1-3]` -- what does this depend on?
- `scope rdeps <symbol> [--depth 1-3]` -- what depends on this?
- `scope trace <symbol>` -- call paths from entry points to target

**Keeping the index fresh:**
- `scope index` -- incremental re-index after edits (< 1s for a few files)
- Line numbers reflect the last index run. Re-index if they look wrong.

Always `scope sketch` before reading full source. Only read source when ready to edit.
```

The same snippet works for Cursor, Aider, and any other agent that reads project instructions from a markdown file. Also available at [`docs/CLAUDE.md.snippet`](docs/CLAUDE.md.snippet).

---

## Building from source

**Prerequisites:** Rust 1.75 or later (`rustup update stable`)

```bash
git clone https://github.com/rynhardt-potgieter/scope.git
cd scope
cargo build --release
# Binary at target/release/scope
```

Run the test suite:

```bash
cargo test                          # 123 tests
cargo clippy -- -D warnings         # zero warnings required
cargo fmt --check                   # formatting check
```

---

## Benchmark methodology

Scope ships with a benchmark harness (`scope-benchmark` v0.6.1) in `benchmarks/runner/` that measures whether it actually reduces token consumption. The harness runs real coding tasks using a 3-arm experiment design, comparing three conditions:

1. **without-scope** -- agent works without Scope CLI (tools disallowed)
2. **with-scope** -- agent has Scope CLI available
3. **with-scope-preloaded** -- agent has Scope CLI available AND `scope map` output baked into CLAUDE.md

Three metrics are measured simultaneously:
- **Input token consumption** across conditions
- **Task correctness** (compilation + tests + caller coverage)
- **Navigation efficiency** (file reads per task)

12 tasks across 6 categories (discovery, bug fix, refactoring, new feature, focused exploration, cross-cutting changes), for both TypeScript and C# fixtures with known dependency graphs. Each task runs 3 reps per condition for statistical reliability.

```bash
# Build the benchmark runner
cd benchmarks/runner
cargo build --release

# Validate a single task across all 3 conditions before committing to a full run
cargo run --release -- test --task ts-cat-a-01 --model sonnet

# Full 3-arm comparison -- all tasks, 3 reps each
cargo run --release -- run --all --conditions 3 --model sonnet --output-dir ../../benchmarks/results/latest

# Manual workflow: prepare work directories, then run tasks manually
cargo run --release -- prepare --all --conditions 3 --output-dir ../../benchmarks/prepared/phase12

# Import results from manual runs
cargo run --release -- import --input results.json --ndjson-dir ndjson/ --output-dir ../../benchmarks/results/latest

# Generate a report from existing results
cargo run --release -- report --input ../../benchmarks/results/latest
```

**Prerequisites:** Claude Code CLI installed, `ANTHROPIC_API_KEY` set, `scope` on PATH, .NET SDK (for C# fixtures), Node.js (for TypeScript fixtures).

Results are committed per release in `benchmarks/results/vX.Y.Z/`. See [`benchmarks/README.md`](benchmarks/README.md) for full documentation.

---

## Roadmap

**v0.1.0 -- v0.5.3 (current)**
- [x] TypeScript and C# symbol extraction with edge detection
- [x] SQLite dependency graph with recursive impact traversal
- [x] Full-text search with FTS5, BM25 ranking, and importance-tier boosting
- [x] `scope init`, `scope index`, `scope sketch`, `scope refs`, `scope callers`, `scope deps`, `scope rdeps`, `scope impact`, `scope find`, `scope status`
- [x] `scope trace` -- entry-point-to-symbol call paths
- [x] `scope entrypoints` -- API controllers, workers, event handlers
- [x] `scope map` -- full repository overview in ~500-1000 tokens
- [x] Enriched sketch output with method modifiers (async, private, static, abstract, virtual, override)
- [x] Enriched FTS5 search with caller/callee indexing and snake_case splitting
- [x] Incremental indexing with SHA-256 hash tracking
- [x] `--json` output on all commands
- [x] Benchmark harness with 12 tasks, 3-arm experiment, correctness verification

**Next**
- [ ] Python, Go, Java, Rust language support
- [ ] `scope index --watch` mode
- [ ] `scope similar`, `scope source` commands (currently stubs)
- [ ] Vector embeddings via local ONNX model (replacing FTS5 for `scope find`)
- [ ] MCP adapter (thin wrapper over the same binary)

**Later**
- [ ] Kotlin and Ruby language support
- [ ] Hosted team index sync
- [ ] CI/CD integration for impact analysis on PRs

---

## License

MIT -- see [LICENSE](LICENSE) for the full text.
