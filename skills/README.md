# Scope Skills for Claude Code

A distributable skill that teaches LLM agents how to use Scope for code navigation.

## Installation

```bash
# 1. Copy the skill (all the detail lives here)
mkdir -p .claude/skills/code-navigation
curl -fsSL https://raw.githubusercontent.com/rynhardt-potgieter/scope/main/skills/code-navigation/SKILL.md \
  > .claude/skills/code-navigation/SKILL.md

# 2. Add the CLAUDE.md snippet (minimal — just tells agents scope exists)
curl -fsSL https://raw.githubusercontent.com/rynhardt-potgieter/scope/main/docs/CLAUDE.md.snippet \
  >> CLAUDE.md

# 3. Allow scope commands (prevents permission prompts in subagents)
# Add to .claude/settings.local.json:
#   { "permissions": { "allow": ["Bash(scope:*)"] } }
```

## How it works

**CLAUDE.md** is minimal — it tells the main session that scope is available and that subagents needing code navigation must be given the `code-navigation` skill. No command lists, no workflows.

**The skill file** carries all the detail:
- Decision tree matching task type to scope command
- Optimal workflows from 54 benchmark runs
- Anti-patterns and the 3-command rule
- Command reference with token costs
- Workspace support

**Why two files?** Subagents don't auto-discover skills. The main session reads CLAUDE.md and knows to pass the skill to subagents that need code navigation. The skill itself has strong trigger descriptions so Claude Code's skill matching fires on any navigation task.

**`Bash(scope:*)`** permission ensures scope commands work without prompts — critical for subagents which can't approve permissions interactively.

## Files

```
skills/
  code-navigation/
    SKILL.md          # The skill — copy to .claude/skills/code-navigation/
  settings.json       # Reference permissions — merge into .claude/settings.local.json
  README.md           # This file
```
