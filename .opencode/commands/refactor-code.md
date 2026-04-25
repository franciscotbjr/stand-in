---
description: Safely restructure code without changing behavior
agent: build
---

I need to refactor the following code. The refactoring must NOT change any external behavior — all existing tests must continue to pass.

## Instructions
1. Analyze the current code and identify the specific problems
2. Propose a refactoring plan with ordered steps — each step should be independently committable
3. For each step, explain what changes and why
4. Implement the refactoring step by step
5. After each step, confirm that:
   - External behavior is unchanged
   - All existing tests should still pass
   - The code follows the Project Definition's conventions
6. If new tests are needed to cover the refactored code, write them
7. Do NOT change any public API, interface, or behavior unless explicitly discussed

If you think the refactoring should also change behavior (e.g., fix a bug discovered during refactoring), flag it separately — do not mix behavior changes with structural refactoring.

Ask the developer for:
- The code to refactor (file references)
- The reason for refactoring
- The desired outcome (if known)
