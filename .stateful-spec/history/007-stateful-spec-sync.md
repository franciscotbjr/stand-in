# Iteration 007 — Stateful Spec Sync

- **Type:** chore
- **Status:** done
- **Created:** 2026-04-03
- **Completed:** 2026-04-03
- **Author:** Francisco Tarcizo Bomfim Júnior

## Description

Synced Stateful Spec methodology and operations from upstream repository (https://github.com/franciscotbjr/stateful-spec) latest `main` to bring the local install up to date.

## What Changed

### Methodology (`.stateful-spec/methodology/`)
- **overview.md** — Added Directory Structure section, Getting Started section, expanded agent portability list, operations/ in project memory structure, iteration file usage note
- **phases/03-specify.md** — Added template file references in specification table
- **phases/04-implement.md** — Added template/prompt path references (implementation-plan, test-plan, write-commit-message)
- **phases/05-verify.md** — Added prompt references (review-changes, update-documentation, write-commit-message)

### Operations (`.claude/commands/`)
- **resume-session.md** — Added "Direct-task entry" section for iteration tracking when dev starts with a concrete task; added methodology path clarification for root vs `.stateful-spec/methodology/`
- **save-session.md** — Added Step 2.5 gap-filling logic for retroactive iteration file creation

### New Files
- **AGENTS.md** (project root) — AI agent instructions with operation list, iteration tracking rules, adapted for Claude Code (`/name` commands)

### Unchanged
- `decision-framework.md`, `roles.md`, `01-analyze.md`, `02-plan.md` — already up to date
- `create-technical-spec.md`, `debug-issue.md`, `refactor-code.md`, `review-changes.md`, `update-documentation.md`, `write-commit-message.md`, `write-tests.md` — already up to date

### Preserved
- `.stateful-spec/memory.md` — untouched (updated separately for record-keeping)
- `.stateful-spec/project-definition.md` — untouched
- `.stateful-spec/history/*` — untouched

## Decisions Made

| Decision | Rationale | Date |
|----------|-----------|------|
| Apply on main without branch | Developer accepted risk; changes are low-risk documentation updates | 2026-04-03 |
| Adapt AGENTS.md for Claude Code | Changed Cursor `@name` references to Claude Code `/name` commands to match project's agent setup | 2026-04-03 |

## References

- Source: https://github.com/franciscotbjr/stateful-spec (latest main)
- Update wizard: `prompts/initialization/update-project.md`
