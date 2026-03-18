# Scope (`sc`)

Code intelligence CLI for LLM coding agents. Gives structural queries -- sketches, references, dependency graphs, impact analysis, semantic search -- without loading full source files.

## Why

LLM coding agents waste tokens reading entire files to understand code structure. Scope builds a local index and answers structural questions in 100-300 tokens instead of 2,000-8,000.

## Install

### From binary (recommended)

```sh
curl -fsSL https://raw.githubusercontent.com/Rayne182/scope/main/install.sh | sh
```

### From source

```sh
cargo install --git https://github.com/Rayne182/scope.git
```

## Quick Start

```sh
sc init                          # initialise in your project
sc index                         # build the index
sc sketch PaymentService         # structural overview
sc refs processPayment           # find all callers
sc deps PaymentService           # what does it depend on?
sc impact processPayment         # blast radius analysis
sc find "payment retry logic"    # semantic search
```

## Commands

| Command | Description |
|---------|-------------|
| `sc init` | Initialise Scope for a project, detect languages, write config |
| `sc index` | Build or refresh the code index (incremental by default) |
| `sc sketch <symbol>` | Structural overview of a class, method, interface, or file |
| `sc refs <symbol>` | Find all references to a symbol across the codebase |
| `sc deps <symbol>` | Show what a symbol depends on |
| `sc rdeps <symbol>` | Show what depends on a symbol (reverse dependencies) |
| `sc impact <symbol>` | Blast radius analysis -- transitive reverse dependency traversal |
| `sc find "<query>"` | Semantic search by intent using FTS5 |
| `sc status` | Index health check -- symbol count, staleness, last index time |

All commands support `--json` for programmatic output.

## Add to Your CLAUDE.md

Copy the following block into your project's `CLAUDE.md` so Claude Code automatically uses Scope when navigating your codebase. The full snippet is also available at [`docs/CLAUDE.md.snippet`](docs/CLAUDE.md.snippet).

```markdown
## Code Navigation

This project uses `Scope` for structural code navigation. Use it before reading or
editing any non-trivial code.

**Before editing a class or function:**
- `sc sketch <ClassName>` -- get structural overview without reading full source
- `sc refs <symbol>` -- find all callers before changing a signature
- `sc impact <symbol>` -- check blast radius before any refactor

**Finding code:**
- `sc find "<description>"` -- find relevant code by intent
- `sc similar <symbol>` -- find similar implementations

**Understanding dependencies:**
- `sc deps <symbol>` -- what does this depend on?
- `sc rdeps <symbol>` -- what depends on this?

**Reading source:**
- `sc source <symbol>` -- fetch full source of a specific symbol

Always run `sc sketch` before `sc source`. Only call `sc source` when you're
ready to read or edit the implementation.

Run `sc --help` for full usage.
```

## Supported Languages

- TypeScript / JavaScript
- C#

More coming: Python, Go, Java, Rust

## Performance

These are acceptance criteria, not aspirational targets.

| Operation | Target |
|-----------|--------|
| `sc sketch <symbol>` | < 100ms |
| `sc refs <symbol>` | < 100ms |
| `sc deps <symbol>` | < 100ms |
| `sc rdeps <symbol>` | < 100ms |
| `sc impact <symbol>` | < 500ms |
| `sc find "<query>"` | < 500ms |
| `sc status` | < 50ms |
| Incremental index (1 file) | < 1s |
| Incremental index (10 files) | < 5s |
| Full index (10k files) | < 60s |

## License

MIT
