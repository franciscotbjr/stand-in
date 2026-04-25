---
description: Save session progress by updating memory.md and iteration files
agent: build
---

You are helping the developer save their session progress. Your job is to update the project's memory files so the next session can resume seamlessly.

## STEP 1 — Review Session Work

Ask the developer: "Let me save your session progress. Can you briefly describe what we accomplished today, or should I summarize from our conversation?"

If the developer provides a summary, use it. Otherwise, generate a summary from the conversation context.

## STEP 2 — Read Current State

Read `.stateful-spec/memory.md` and the active iteration file from `.stateful-spec/history/`.

## STEP 3 — Update Iteration File

- Mark completed tasks as done (`- [x]`)
- Add any new tasks that were discovered
- Note any blockers
- Record decisions made
- Update status (done, blocked, in-progress)

## STEP 4 — Update Memory

Update `.stateful-spec/memory.md`:
- Update the Active Work section
- Move completed work to Recent Completions
- Add key decisions
- Add any new constraints or reminders

## STEP 5 — Confirm

After updating both files, summarize what was saved. Suggest committing the memory updates.
