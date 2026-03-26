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
[![Version](https://img.shields.io/badge/version-v0.7.3-blue.svg)](https://github.com/rynhardt-potgieter/scope/releases)
[![Build](https://img.shields.io/badge/build-passing-22863a)](https://github.com/rynhardt-potgieter/scope/actions)
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
- [Watch mode](#watch-mode)
- [Workspaces](#workspaces)
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
![Python](https://img.shields.io/badge/Python-ready-22863a?style=flat-square&logo=python&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-ready-22863a?style=flat-square&logo=rust&logoColor=white)

All four languages have full support: tree-sitter grammar integration, symbol extraction, edge detection (calls, imports, extends, implements), and enriched metadata (async, static, private, abstract, decorators, visibility). C# includes partial class merging; Python includes decorator and docstring extraction; Rust includes impl block method association and visibility modifiers.

### Planned

![Go](https://img.shields.io/badge/Go-planned-e6a817?style=flat-square&logo=go&logoColor=white)
![Java](https://img.shields.io/badge/Java-planned-e6a817?style=flat-square&logo=openjdk&logoColor=white)
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
# scope 0.7.3
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
scope index --watch                      # auto re-index on file changes
```

`scope index --watch` monitors your project for file changes and re-indexes automatically with a 300ms debounce. Run it in a terminal tab during development -- agents never see stale line numbers.

### 5. Add to CLAUDE.md

Paste the [CLAUDE.md snippet](#claudemd-integration) into your project so agents use Scope automatically.

---

## Commands

| Command | Signature | Description | When to use |
|---|---|---|---|
| `scope init` | `[--json]` | Initialise Scope for a project. Creates `.scope/` with default config, auto-detects languages. | Once per project, before first `scope index`. |
| `scope index` | `[--full] [--watch] [--json]` | Build or refresh the code index. Incremental by default, `--full` forces rebuild, `--watch` auto re-indexes on file changes. | Once on setup, then `--watch` during development or manual `scope index` after edits. |
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
| `scope workspace init` | `[--name NAME]` | Discover projects with `.scope/` in subdirectories and create `scope-workspace.toml`. | Once per workspace, after running `scope init` in each project. |
| `scope workspace list` | `[--json]` | Show all workspace members with index status, symbol counts, and freshness. | Checking workspace health before cross-project queries. |
| `scope workspace index` | `[--full] [--watch] [--json]` | Index all workspace members. With `--watch`, starts a file watcher per member. | Initial setup, batch refresh, or continuous watching of all projects. |

### Global flags

| Flag | Description |
|---|---|
| `--workspace` | Query across all workspace members. Requires `scope-workspace.toml`. Works with: `status`, `map`, `refs`, `find`, `entrypoints`. |
| `--project <name>` | Target a specific workspace member by name. Overrides CWD-based project detection. |
| `--verbose` | Enable debug output to stderr. |
| `--json` | Output structured JSON instead of human-readable text. Supported on all commands. |

All commands support `--json` for structured output. Line numbers reflect the state of the index at last run -- use `scope status` to check freshness before range-based edits.

---

## Watch mode

`scope index --watch` monitors your project for file changes and automatically re-indexes when source files are modified. It's designed to run in a background terminal tab during development so the index is always fresh.

```bash
scope index --watch
```

```
Initial index: 0.8s. 4,821 symbols, 12,456 edges.
Watching for changes... (Ctrl+C to stop)
Re-indexed 2 files (34ms) — 2 symbols updated
Re-indexed 1 file (12ms) — 0 symbols updated
```

### How it works

- Uses the `notify` crate for cross-platform file system events (inotify on Linux, FSEvents on macOS, ReadDirectoryChangesW on Windows)
- Debounces rapid changes with a 300ms window -- saving 5 files in quick succession triggers one re-index, not five
- Respects `.gitignore` and `config.index.ignore` patterns -- `node_modules/`, `dist/`, etc. are never watched
- Only reacts to files with supported extensions (`.ts`, `.tsx`, `.cs`, etc.)
- A lock file (`.scope/.watch.lock`) prevents multiple watchers on the same project

### NDJSON output

For programmatic consumers, `--watch --json` emits newline-delimited JSON to stdout:

```bash
scope index --watch --json
```

```json
{"event":"start","project":"api","languages":["typescript"],"timestamp":"2026-03-24T10:30:00Z"}
{"event":"reindex","files_changed":2,"symbols_added":3,"duration_ms":34,"timestamp":"2026-03-24T10:30:05Z"}
{"event":"stop","total_reindexes":42,"uptime_seconds":3600,"timestamp":"2026-03-24T11:30:00Z"}
```

### Concurrent access

Watch mode writes to `graph.db` on each re-index. Other `scope` commands (sketch, refs, map, etc.) can read concurrently thanks to SQLite WAL mode and a 5-second busy timeout. You don't need to stop the watcher to query.

---

## Workspaces

Workspaces let you query across multiple Scope projects as a single unit. This is useful for monorepos, multi-repo setups, and polyglot projects where different parts of the codebase live in separate directories.

### The model

Each project keeps its own independent `.scope/` index. A `scope-workspace.toml` manifest at the workspace root ties them together. When you pass `--workspace` to a command, Scope opens all member databases and fans out the query, merging results with project labels.

```
~/repos/
  scope-workspace.toml              ← workspace manifest
  api-gateway/
    .scope/                         ← Go index
    src/
  user-service/
    .scope/                         ← Rust index
    src/
  web-app/
    .scope/                         ← TypeScript index
    src/
```

**Key properties:**
- Workspace is **opt-in** -- single-project behavior is unchanged without `--workspace`
- Each project index is **independent** -- you can build, refresh, and query them separately
- Cross-project queries **merge results** from all members with project-name labels
- No cross-project edge detection -- Scope doesn't know that your TypeScript frontend calls your Go backend. Each project's call graph is self-contained.

### Setup

```bash
# 1. Initialise and index each project individually
cd ~/repos/api-gateway && scope init && scope index
cd ~/repos/user-service && scope init && scope index
cd ~/repos/web-app && scope init && scope index

# 2. Create the workspace manifest from the parent directory
cd ~/repos
scope workspace init
```

```
Found 3 projects:
  api-gateway     (go)
  user-service    (rust)
  web-app         (typescript)
Wrote scope-workspace.toml
```

Alternatively, index all members at once after creating the manifest:

```bash
scope workspace init
scope workspace index
```

### Manifest format

`scope-workspace.toml` is a simple TOML file:

```toml
[workspace]
name = "my-platform"

[[workspace.members]]
path = "api-gateway"

[[workspace.members]]
path = "user-service"

[[workspace.members]]
path = "web-app"
name = "frontend"          # optional -- defaults to directory name
```

You can edit this file manually to add or remove members, rename projects, or reorder entries.

### Querying across projects

Add `--workspace` to any supported command:

```bash
# Overview of the entire workspace
scope map --workspace

# Find who calls PaymentService across all projects
scope refs PaymentService --workspace

# Search for authentication code everywhere
scope find "authentication" --workspace

# List all entry points across all projects
scope entrypoints --workspace

# Check health of all member indexes
scope status --workspace
```

Results are tagged with project names so you can tell which project each symbol belongs to:

```
PaymentService (api-gateway)    class  src/payments/service.go:12-89
PaymentService (web-app)        class  src/types/payment.ts:5-22
```

### Targeting a specific member

Use `--project <name>` to query a specific workspace member without `cd`-ing into it:

```bash
scope sketch PaymentService --project api-gateway
scope refs processPayment --project web-app
```

### Watch mode with workspaces

Watch all workspace members with a single command:

```bash
scope workspace index --watch
```

```
[api-gateway] Watcher started (PID 12345)
[user-service] Watcher started (PID 12346)
[web-app] Watcher started (PID 12347)

Watching 3 members (Ctrl+C to stop all)...
```

This spawns one `scope index --watch` process per member. Each watcher is independent — a file change in `web-app/` only re-indexes that project. Press Ctrl+C to stop all watchers.

You can also watch individual projects manually:

```bash
cd ~/repos/web-app && scope index --watch
```

### Limitations

- **No cross-project edges.** Scope doesn't detect that `web-app` calls `api-gateway` via HTTP. Each project's dependency graph is self-contained. Cross-project references are a planned future feature.
- **Commands that need source access** (`sketch`, `deps`, `trace`, `callers`) work on one project at a time. Use `--project <name>` to target a member, or `cd` into the project directory.
- **Nested projects are detected.** If project A contains project B's directory, the file walker skips B's files to avoid double-indexing. Each project only indexes its own source tree.

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

## Agent integration

### Setup

```bash
# 1. Copy the skill (teaches agents HOW to use scope)
mkdir -p .claude/skills/code-navigation
curl -fsSL https://raw.githubusercontent.com/rynhardt-potgieter/scope/main/skills/code-navigation/SKILL.md \
  > .claude/skills/code-navigation/SKILL.md

# 2. Add the CLAUDE.md snippet (tells agents scope EXISTS)
curl -fsSL https://raw.githubusercontent.com/rynhardt-potgieter/scope/main/docs/CLAUDE.md.snippet \
  >> CLAUDE.md

# 3. Allow scope commands (so agents don't get blocked by permissions)
# In Claude Code, run: /permissions
# Or add to .claude/settings.local.json:
#   { "permissions": { "allow": ["Bash(scope:*)"] } }
```

**How it works:**
- **CLAUDE.md** tells the main session that scope is available and that subagents needing code navigation must be given the `code-navigation` skill
- **The skill file** contains all the detail — decision trees, command reference, optimal workflows from 54 benchmark runs, anti-patterns, and the 3-command rule
- **`Bash(scope:*)`** permission ensures scope commands work without prompts in both main sessions and subagents

See [`skills/README.md`](skills/README.md) for full details. The snippet also works with Cursor, Aider, and any other agent that reads project instructions.

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

**v0.1.0 -- v0.7.3 (current)**
- [x] TypeScript and C# symbol extraction with edge detection
- [x] SQLite dependency graph with recursive impact traversal
- [x] Full-text search with FTS5, BM25 ranking, and importance-tier boosting
- [x] 15 commands: `init`, `index`, `sketch`, `refs`, `callers`, `deps`, `rdeps`, `impact`, `find`, `trace`, `entrypoints`, `map`, `status`, `similar` (stub), `source` (stub)
- [x] `scope index --watch` -- auto re-index on file changes with notify crate
- [x] Multi-project workspaces -- `scope workspace init/list/index`, `--workspace` flag on 5 commands, `--project <name>` targeting, workspace-level `--watch`
- [x] `WorkspaceGraph` federated query facade with symbol ID namespacing
- [x] `LanguagePlugin` trait for pluggable language support (parser refactor)
- [x] Python language support -- decorator metadata, docstring extraction, access inference
- [x] Rust language support -- structs, enums, traits, visibility modifiers, dogfooding
- [x] Enriched output with method modifiers, CamelCase/snake_case-aware FTS5
- [x] `--json` output on all commands
- [x] Benchmark harness with 12 tasks, 3-arm experiment, correctness verification

**Next**
- [ ] Go and Java language support
- [ ] `scope similar`, `scope source` commands (currently stubs)
- [ ] Vector embeddings via local ONNX model (replacing FTS5 for `scope find`)
- [ ] Cross-project edge detection via `scope link`
- [ ] MCP adapter (thin wrapper over the same binary)

**Later**
- [ ] Kotlin and Ruby language support
- [ ] Hosted team index sync
- [ ] CI/CD integration for impact analysis on PRs

---

## License

MIT -- see [LICENSE](LICENSE) for the full text.
