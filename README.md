```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   ███████╗ ██████╗ ██████╗ ██████╗ ███████╗                │
│   ██╔════╝██╔════╝██╔═══██╗██╔══██╗██╔════╝                │
│   ███████╗██║     ██║   ██║██████╔╝█████╗                  │
│   ╚════██║██║     ██║   ██║██╔═══╝ ██╔══╝                  │
│   ███████║╚██████╗╚██████╔╝██║     ███████╗                │
│   ╚══════╝ ╚═════╝ ╚═════╝ ╚═╝     ╚══════╝               │
│                                                             │
│   Code intelligence for LLM coding agents.                  │
│   Know before you touch.                                    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

[![Rust](https://img.shields.io/badge/built_with-Rust-orange?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-v0.1.0-blue.svg)](https://github.com/Rayne182/scope/releases)
[![Build](https://img.shields.io/github/actions/workflow/status/Rayne182/scope/ci.yml?label=build)](https://github.com/Rayne182/scope/actions)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey)](#installation)
[![Stars](https://img.shields.io/github/stars/Rayne182/scope?style=flat)](https://github.com/Rayne182/scope/stargazers)
[![Issues](https://img.shields.io/github/issues/Rayne182/scope)](https://github.com/Rayne182/scope/issues)
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
  processPayment  (amount: Decimal, userId: string) → Promise<PaymentResult>   [11 callers]
  refundPayment   (txId: string, reason?: string)   → Promise<bool>             [3 callers]
  validateCard    (card: CardDetails)               → ValidationResult          [internal]
  getTransaction  (id: string)                      → Transaction | null        [2 callers]

fields:
  private readonly  client  : StripeClient
  private           repo    : UserRepository
  private           logger  : Logger

// ~180 tokens  ·  source file is 6,200 tokens
```

---

## Supported languages

### Phase 0 -- v0.1.0 (current)

![TypeScript](https://img.shields.io/badge/TypeScript-ready-22863a?style=flat-square&logo=typescript&logoColor=white)
![C#](https://img.shields.io/badge/C%23-ready-22863a?style=flat-square&logo=csharp&logoColor=white)

### Phase 1 -- v0.2.0

![Python](https://img.shields.io/badge/Python-v0.2-e6a817?style=flat-square&logo=python&logoColor=white)
![Go](https://img.shields.io/badge/Go-v0.2-e6a817?style=flat-square&logo=go&logoColor=white)
![Java](https://img.shields.io/badge/Java-v0.2-e6a817?style=flat-square&logo=openjdk&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-v0.2-e6a817?style=flat-square&logo=rust&logoColor=white)

### Planned

![Kotlin](https://img.shields.io/badge/Kotlin-planned-6e7681?style=flat-square&logo=kotlin&logoColor=white)
![Ruby](https://img.shields.io/badge/Ruby-planned-6e7681?style=flat-square&logo=ruby&logoColor=white)

Language support means: tree-sitter grammar integrated, symbol extraction tested, and edge detection (calls, imports, extends, implements) working correctly for that language's idioms. C# includes partial class merging across files.

---

## Installation

### curl (Linux and macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/Rayne182/scope/main/install.sh | sh
```

### cargo

```bash
cargo install --git https://github.com/Rayne182/scope.git scope
```

After installation, verify with:

```bash
scope --version
# scope 0.1.0
```

> **Windows PowerShell note:** The binary is named `scope` — no conflicts with PowerShell aliases.

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

### 3. Query your codebase

```bash
scope sketch PaymentService              # structural overview
scope refs processPayment                # find all callers
scope deps PaymentService                # what does it depend on?
scope impact processPayment              # blast radius analysis
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
| `scope sketch` | `<symbol> [--json]` | Compressed structural overview: methods with caller counts, dependencies, type signatures, modifiers. ~200 tokens vs ~4,000 for full source. | Before reading source or editing any non-trivial symbol. |
| `scope refs` | `<symbol> [--kind calls\|imports\|extends] [--limit N]` | All references grouped by kind: call sites, instantiations, imports, type annotations. Includes file path and line number for each. | Before changing a function signature or deleting a symbol. |
| `scope impact` | `<symbol> [--depth 1-5]` | Transitive reverse dependency traversal. Shows direct callers, second-degree dependents, and affected test files. Ranked by proximity. | Before any refactor that changes a public API surface. |
| `scope find` | `"<intent>" [--kind function\|class] [--limit N]` | Full-text search with BM25 ranking. CamelCase-aware so `processPayment` matches "payment". | Navigating an unfamiliar codebase or finding code by intent. |
| `scope deps` | `<symbol> [--depth 1-3]` | What does this symbol depend on? Direct imports, called functions, extended classes. Transitive with `--depth`. | Understanding what must exist before implementing something new. |
| `scope status` | `[--json]` | Index health: symbol count, file count, last indexed time. | Checking whether the index is stale before making range-based edits. |
| `scope index` | `[--full]` | Build or refresh the code index. Incremental by default. `--full` forces a complete rebuild. | Once on project setup, then after edits during active development. |

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
|  scope query engine             |  Combines structural traversal and text search.
|                              |  Returns labelled, token-efficient output designed
+-----------------------------+  for LLM consumption, not human readability.
```

**Parsing** -- tree-sitter produces a concrete syntax tree for every file. Scope extracts symbols (functions, classes, methods, interfaces, enums, types) with their signatures, type annotations, access modifiers, async status, and docstrings. For C#, partial classes are merged across files before indexing.

**Structural graph** -- symbols are nodes in a SQLite database. Edges represent relationships: `calls`, `imports`, `extends`, `implements`, `instantiates`, `references_type`. Impact analysis uses recursive common table expressions to traverse this graph to arbitrary depth.

**Full-text search** -- symbol names (with CamelCase splitting), signatures, and docstrings are indexed in an FTS5 virtual table with porter stemming. `scope find "payment"` matches `processPayment`, `PaymentService`, and any symbol with "payment" in its docstring. Scores are normalized to 0.00-1.00 and sorted by BM25 relevance.

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
## Code navigation

This project uses Scope for structural code navigation. Use it before editing.

**Before editing a class or function:**
- `scope sketch <ClassName>` -- structural overview without reading full source
- `scope refs <symbol>` -- find all callers before changing a signature
- `scope impact <symbol>` -- check blast radius before any refactor

**Finding code:**
- `scope find "<description>"` -- find relevant code by intent

**Understanding dependencies:**
- `scope deps <symbol>` -- what does this depend on?

**Keeping the index fresh:**
- Run `scope status` to check if the index is stale
- Run `scope index` after making edits -- incremental, takes < 1s for a few files
- Line numbers in output reflect the last index run. If they look wrong, re-index first.

Always run `scope sketch` before reading full source. Only read implementation when
you know exactly which file and line range you need.
```

The same snippet works for Cursor, Aider, and any other agent that reads project instructions from a markdown file. Also available at [`docs/CLAUDE.md.snippet`](docs/CLAUDE.md.snippet).

---

## Building from source

**Prerequisites:** Rust 1.75 or later (`rustup update stable`)

```bash
git clone https://github.com/Rayne182/scope.git
cd scope
cargo build --release
# Binary at target/release/sc
```

Run the test suite:

```bash
cargo test                          # 84 tests
cargo clippy -- -D warnings         # zero warnings required
cargo fmt --check                   # formatting check
```

---

## Benchmark methodology

Scope ships with a benchmark harness in `benchmarks/runner/` that measures whether it actually reduces token consumption. The harness runs real coding tasks with and without Scope enabled, comparing three metrics: input token consumption, task correctness (compilation + tests + caller coverage), and navigation efficiency (file reads per task).

20 tasks across 5 categories (signature refactoring, cross-cutting changes, dependency understanding, impact-aware refactoring, discovery), for both TypeScript and C# fixtures with known dependency graphs.

```bash
# Build and run
cd benchmarks/runner
cargo build --release
cargo run --release -- run --all --compare --reps 5

# Single task
cargo run --release -- run --task ts-cat-a-01 --compare
```

**Prerequisites:** Claude Code installed, `ANTHROPIC_API_KEY` set, `sc` on PATH.

Results are committed per release in `benchmarks/results/vX.Y.Z/`. Full methodology and caveats documented in `benchmarks/results/v0.1.0/summary.md`.

See the [benchmark spec](benchmarks/) for task definitions and how to add your own.

---

## Roadmap

**v0.1.0 -- current**
- [x] TypeScript and C# symbol extraction with edge detection
- [x] SQLite dependency graph with recursive impact traversal
- [x] Full-text search with FTS5 and BM25 ranking
- [x] `scope sketch`, `scope refs`, `scope impact`, `scope find`, `scope deps`, `scope status`, `scope index`
- [x] Incremental indexing with SHA-256 hash tracking
- [x] `--json` output on all commands
- [x] Benchmark harness with 20 tasks across TypeScript and C# fixtures

**v0.2.0**
- [ ] Python, Go, Java, Rust language support
- [ ] `scope index --watch` mode
- [ ] `scope rdeps`, `scope similar`, `scope source` commands
- [ ] Vector embeddings via local ONNX model (replacing FTS5 for `scope find`)
- [ ] Expanded benchmark suite (30 tasks)

**v0.3.0**
- [ ] MCP adapter (thin wrapper over the same binary)
- [ ] Kotlin and Ruby language support

**Later**
- [ ] Hosted team index sync
- [ ] CI/CD integration for impact analysis on PRs

---

## License

MIT -- see [LICENSE](LICENSE) for the full text.
