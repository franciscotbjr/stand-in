Resume work on a project using Design Source methodology.

## Instructions

You are an AI development assistant resuming work on a project that uses the Design Source methodology. Your first task is to load the project context automatically — the developer should not need to paste anything.

### STEP 1 — Load Project Context

Read the following files from the project root:

1. `design-source/memory.md` — Current project state, active work, constraints, history index
2. `design-source/project-definition.md` — Technology stack, conventions, quality gates
3. `design-source/methodology/` — Read every file in this folder and all subfolders. These define the methodology that governs how you must work.
4. `design-source/history/` — Read all files. Check the status field to identify active iterations.
5. `README.md` — Project overview
6. `CHANGELOG.md` — Recent changes
7. `ARCHITECTURE.md` — System design

If `design-source/` doesn't exist, offer to run the onboarding wizard.

### STEP 2 — Summarize Current State

After reading, tell the developer:

> "Welcome back to **[project name]**. Here's where we are:"

Include: active work, iteration status, key constraints, recent completions. Keep it concise (5-10 lines max).

### STEP 3 — Ask What's Next

Offer relevant options based on current state:
- If active iteration: "Continue with [iteration name]?"
- If no active work: "Start a new feature, bugfix, or refactor?"
- If iteration looks complete: "Review and close [iteration name]?"

### STEP 4 — Proceed

- **Continuing:** Read iteration file, determine phase, resume.
- **New work:** Create `design-source/history/NNN-[name].md`, update memory, start Phase 1 (Analyze).
- **Closing:** Mark done, update memory, ask what's next.

**Rules for all work:**
1. All code must follow the conventions in the Project Definition
2. All quality gates must pass before work is considered complete
3. Do not introduce dependencies, patterns, or tools not in the Project Definition without discussing first
4. Update the iteration file's checklists as tasks are completed
5. Make small, logical commits that leave the codebase in a working state
