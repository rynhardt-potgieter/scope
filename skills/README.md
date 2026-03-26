# Scope Skills for Claude Code

This directory contains a distributable skill that teaches LLM agents how to use Scope effectively — including subagents.

## Installation (3 steps)

### 1. Copy the skill

```bash
# From your project root (where .scope/ lives)
mkdir -p .claude/skills/code-navigation
curl -fsSL https://raw.githubusercontent.com/rynhardt-potgieter/scope/main/skills/code-navigation/SKILL.md \
  > .claude/skills/code-navigation/SKILL.md
```

### 2. Add the CLAUDE.md snippet

Paste this into your project's `CLAUDE.md` (create it if it doesn't exist):

```markdown
## Code Navigation — USE SCOPE FIRST

This project has Scope installed for structural code intelligence.
**Before grepping or reading files, check if scope can answer your question faster.**
Start with `scope map` for a repo overview, then `scope sketch` for specific symbols.
For detailed usage patterns, read `.claude/skills/code-navigation/SKILL.md`.

When dispatching subagents, include: "This project has Scope CLI installed.
Use scope commands (scope sketch, scope refs, scope callers, scope find, scope map)
instead of grep for code navigation."
```

The snippet tells the main session about scope. The last paragraph ensures subagents get scope instructions too.

### 3. Allow scope commands (critical for subagents)

Add `scope` to your project's allowed Bash commands so agents (including subagents) can run scope without permission prompts:

```bash
# In Claude Code, run:
/permissions

# Or manually add to .claude/settings.local.json:
```

```json
{
  "permissions": {
    "allow": [
      "Bash(scope:*)"
    ]
  }
}
```

This single permission covers all scope subcommands (`scope sketch`, `scope refs`, `scope map`, etc.). Without it, every scope command triggers a permission prompt — and in non-interactive subagents, it fails silently.

## How it works

**Main session:** Claude Code reads `CLAUDE.md` at session start and discovers the skill in `.claude/skills/`. When the agent encounters a code navigation task, the skill triggers and provides decision trees, optimal workflows, and anti-patterns.

**Subagents:** The CLAUDE.md snippet tells the orchestrating agent to include scope instructions when dispatching subagents. The `Bash(scope:*)` permission ensures subagents can execute scope commands without being blocked.

**Result:** Both the main session and all subagents use scope for navigation instead of grep. Benchmark data shows this reduces cost by 32% and increases code edits by 67%.

## What the skill teaches agents

- **Check for scope** at session start (`scope status`)
- **Use scope instead of grep** for code navigation
- **Follow optimal workflows** from 54 benchmark runs (discovery, bug fix, refactor, new feature)
- **The 3-command rule** — if you've run 3 scope commands without editing, start editing
- **Anti-patterns** — don't sketch what you'll read, don't run callers AND refs, don't over-navigate
- **Workspace support** — `--workspace` flag for multi-project queries

## Files

```
skills/
  code-navigation/
    SKILL.md          # The skill file — copy to .claude/skills/code-navigation/
  settings.json       # Reference permissions — merge into .claude/settings.local.json
  README.md           # This file
```
