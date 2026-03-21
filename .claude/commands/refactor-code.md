Refactor code without changing external behavior.

## Instructions

Read `design-source/project-definition.md` first. The refactoring must NOT change any external behavior — all existing tests must continue to pass.

Ask the developer what code needs refactoring and why if not already provided.

### Process

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

If the refactoring should also change behavior (e.g., fix a bug discovered during refactoring), flag it separately — do not mix behavior changes with structural refactoring.
