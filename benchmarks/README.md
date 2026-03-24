# Scope Benchmark Harness

Measures whether Scope CLI actually reduces token consumption and improves task correctness for LLM coding agents.

## What it measures

The harness runs real coding tasks using a **3-arm experiment** design:

| Condition | Description |
|-----------|-------------|
| **without-scope** | Agent works without Scope CLI (tools disallowed via `Bash(scope:*)` glob) |
| **with-scope** | Agent has Scope CLI available |
| **with-scope-preloaded** | Agent has Scope CLI AND `scope map` output baked into CLAUDE.md |

Three metrics are captured simultaneously:
1. **Input token consumption** -- the primary efficiency metric
2. **Task correctness** -- compilation, tests, and caller coverage verification
3. **Navigation efficiency** -- file reads per task

12 tasks across 6 categories, for both TypeScript and C# codebases with known dependency graphs.

## Prerequisites

- **Anthropic API key** — get one at [console.anthropic.com](https://console.anthropic.com)
- **Claude CLI** — `npm install -g @anthropic-ai/claude-code`
- **Scope CLI** — `cargo install --path .` from the repo root
- **Node.js** — for TypeScript fixture compilation (`npx tsc`)
- **.NET SDK** — for C# fixture compilation (`dotnet build`)
- **Rust toolchain** — to build the benchmark runner

## Setup

```bash
# 1. Build the benchmark runner
cd benchmarks/runner
cargo build

# 2. Install scope globally (needed for with-scope and preloaded conditions)
cd ../..
cargo install --path .
scope --version    # Should print scope 0.5.3+

# 3. Set your API key
```

**Linux/macOS:**
```bash
export ANTHROPIC_API_KEY="sk-ant-..."
```

**Windows PowerShell:**
```powershell
$env:ANTHROPIC_API_KEY = "sk-ant-..."
```

To persist the key across sessions (Windows):
```powershell
[System.Environment]::SetEnvironmentVariable('ANTHROPIC_API_KEY', 'sk-ant-...', 'User')
# Restart terminal after this
```

```bash
# 4. Verify everything
claude --version   # Should print 2.x.x
scope --version    # Should print scope 0.5.3+
cd benchmarks/runner
cargo run -- --version  # Should print benchmark 0.6.6+
```

## Quick start

### 1. Test with Haiku first (always)

Validates telemetry capture before spending real money. Costs ~$0.10:

```bash
cd benchmarks/runner
cargo run -- test --task ts-cat-a-01 --model haiku
```

Runs 1 task across all 3 conditions (3 agent invocations). Prints PASS/FAIL per condition, validating that tokens, actions, file reads, scope commands, and NDJSON streams are all captured. **Fix any failures before proceeding to a full run.**

Expected output for each condition:
- `Input tokens: > 0`
- `Output tokens: > 0`
- `Actions: > 0`
- `File reads: > 0`
- `Scope commands: [scope find]` (for with-scope conditions)
- `Telemetry: VALID`

### 2. Run

Full automated benchmark:

```bash
# 3-arm experiment, all tasks, 3 reps each (108 agent invocations)
cargo run -- run --all --conditions 3 --model sonnet --output-dir ../../benchmarks/results/latest
```

### 3. Report

Generate a summary from existing results:

```bash
cargo run -- report --input ../../benchmarks/results/latest
```

## Full command reference

### `benchmark test`

Single-task validation across all 3 conditions (1 rep each).

```bash
benchmark test --task ts-cat-a-01 --model sonnet
benchmark test --language typescript --model opus
```

| Flag | Description |
|------|-------------|
| `--task <id>` | Task ID to test (default: ts-cat-a-01) |
| `--language <lang>` | Pick first task from this language |
| `--model <name>` | Model for agent runs (required): sonnet, opus, haiku |

### `benchmark run`

Full automated benchmark with configurable conditions and repetitions.

```bash
# 3-arm, all tasks
benchmark run --all --conditions 3 --model sonnet --output-dir ./results

# 2-arm comparison
benchmark run --all --compare --model sonnet

# Single task
benchmark run --task ts-cat-a-01 --compare --model sonnet --reps 5

# Save NDJSON for post-hoc analysis
benchmark run --all --conditions 3 --model sonnet --save-ndjson ./ndjson/
```

| Flag | Description | Default |
|------|-------------|---------|
| `--all` | Run all 12 tasks | - |
| `--task <id>` | Run a single task | - |
| `--language <lang>` | Run all tasks for a language | - |
| `--category <cat>` | Run all tasks in a category | - |
| `--reps N` | Repetitions per task per condition | 3 |
| `--compare` | Run with-scope + without-scope (2-arm) | - |
| `--conditions 3` | Run all 3 arms | 1 |
| `--no-scope` | Run baseline only | - |
| `--scope-only` | Run with-scope only | - |
| `--model <name>` | Model for agent runs | - |
| `--save-ndjson <dir>` | Save raw NDJSON streams | - |
| `--output-dir <dir>` | Where results are saved | `benchmarks/results/latest/` |

Always writes `full_results.json`, `summary.md`, and `environment.json`.

### `benchmark prepare`

Sets up isolated work directories for manual runs. Does NOT require an API key.

```bash
# 3-arm, all tasks
benchmark prepare --all --conditions 3 --output-dir ../../benchmarks/prepared/phase12

# 2-arm comparison
benchmark prepare --all --compare --output-dir ../../benchmarks/prepared/phase12
```

| Flag | Description | Default |
|------|-------------|---------|
| `--all` | Prepare all 12 tasks | - |
| `--task <id>` | Prepare a single task | - |
| `--language <lang>` | Prepare all tasks for a language | - |
| `--compare` | Prepare with-scope + without-scope (2-arm) | - |
| `--conditions 3` | Prepare all 3 arms | 1 |
| `--output-dir <dir>` | Where work dirs are created | `benchmarks/prepared/` |

Outputs a `manifest.json` with task prompts and work directory paths. Automatically verifies fixture integrity via SHA256 manifests before copying.

### `benchmark import`

Ingests manually captured results for analysis.

```bash
benchmark import --input results.json --ndjson-dir ./ndjson/ --output-dir ../../benchmarks/results/latest
```

| Flag | Description | Default |
|------|-------------|---------|
| `--input <path>` | JSON file with manually captured results | Required |
| `--ndjson-dir <dir>` | Directory with NDJSON files (`<task_id>-<condition>.ndjson`) | - |
| `--output <format>` | Report format: `json` or `markdown` | `markdown` |
| `--output-dir <dir>` | Where results are saved | - |

### `benchmark manifest`

Generate or verify fixture integrity manifests.

```bash
# Generate manifests (after intentional fixture changes)
benchmark manifest --generate

# Verify all fixtures
benchmark manifest --verify

# Verify a single fixture
benchmark manifest --verify --fixture ../../benchmarks/fixtures/typescript-large
```

| Flag | Description |
|------|-------------|
| `--generate` | Create manifest from current fixture state |
| `--verify` | Check fixtures against stored manifests |
| `--fixture <path>` | Path to a specific fixture directory |

### `benchmark verify`

Verify correctness of a completed work directory.

```bash
# Human-readable output
benchmark verify --dir ./prepared/ts-cat-b-01-with-scope

# JSON output
benchmark verify --dir ./prepared/ts-cat-b-01-with-scope --json

# Specify task ID explicitly
benchmark verify --dir /path/to/work/dir --task ts-cat-b-01
```

| Flag | Description |
|------|-------------|
| `--dir <path>` | Path to completed work directory (required) |
| `--task <id>` | Task ID (auto-detected from directory name if omitted) |
| `--json` | Output as JSON instead of human-readable |

### `benchmark report`

Generate a report from existing results.

```bash
benchmark report --input ../../benchmarks/results/latest
benchmark report --input ../../benchmarks/results/latest --output json
```

| Flag | Description | Default |
|------|-------------|---------|
| `--input <path>` | Path to results file or directory (required) | - |
| `--output <format>` | Report format: `json` or `markdown` | `markdown` |

## Output directory structure

```
benchmarks/
  results/
    vX.Y.Z/
      phaseNN/
        full_results.json      # All raw run data
        summary.md             # Human-readable comparison report
        environment.json       # Machine specs, tool versions
        ndjson/                # Raw NDJSON streams (if saved)
  prepared/
    phaseNN/
      manifest.json            # Task prompts and work dir paths
      ts-cat-a-01-with-scope/  # Isolated work directory
      ts-cat-a-01-without-scope/
      ...
      ndjson/                  # For saving NDJSON during manual runs
  fixtures/
    typescript-api/            # Small TS fixture (~20 files)
    typescript-large/          # Enterprise TS fixture (~194 files)
    csharp-api/                # Small C# fixture (~20 files)
    csharp-large/              # Enterprise C# fixture (~182 files)
  tasks/
    typescript/                # Task TOML definitions
    csharp/
```

## Task categories

| Category | What it tests | Example |
|----------|--------------|---------|
| **Cat-A** (Discovery) | Finding code by behavior description, not by name | "Find the code that handles charge decline" |
| **Cat-B** (Bug Fix) | Diagnosing and fixing a planted bug | Swallowed error, data integrity violation |
| **Cat-C** (Refactoring) | Extracting patterns from existing code | Strategy pattern, validator extraction |
| **Cat-D** (New Feature) | Creating new code that integrates with existing | New service that calls existing dependencies |
| **Cat-E** (Focused Exploration) | Understanding a specific flow for debugging | "Explain the payment flow" |
| **Cat-F** (Cross-cutting) | Changes that touch many files | Add structured logging to all catch blocks |

Each category has one TypeScript and one C# task (12 tasks total).

## Fixtures

Two sizes per language:

- **`*-api`** -- Small (~20 files). For quick iteration and debugging.
- **`*-large`** -- Enterprise-scale (~190 files). Clean Architecture, CQRS, DI, cross-cutting concerns. Used for actual benchmarks.

Each fixture includes:
- `CLAUDE.md.with-scope` / `CLAUDE.md.without-scope` -- swapped per condition
- `CLAUDE.md.with-scope-preloaded` -- template with `{{SCOPE_MAP_OUTPUT}}` placeholder
- `.scope/` -- pre-built Scope index
- `.fixture-manifest.sha256` -- SHA256 integrity hashes

Fixtures must not be modified without regenerating manifests (`benchmark manifest --generate`). The `prepare` command rejects fixtures with stale manifests.

## Cost estimates

Rough estimates per full run (12 tasks x 3 conditions x 3 reps = 108 agent invocations):

| Model | Est. cost per full run | Notes |
|-------|----------------------|-------|
| Haiku | ~$5-15 | Cheapest, good for methodology validation |
| Sonnet | ~$30-80 | Best cost/quality tradeoff |
| Opus | ~$150-400 | Most capable, highest cost |

Use `benchmark test` first (3 invocations, ~1/36th of a full run cost) to validate your setup.
