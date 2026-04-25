---
description: Self-review code changes before committing
agent: plan
---

Please review the following code changes as if you were a code reviewer. Be thorough but constructive.

Review the changes against these criteria:

1. Correctness — Does the code do what the spec says? Are there logic errors?
2. Completeness — Are all acceptance criteria from the spec addressed? Anything missing?
3. Convention Compliance — Does the code follow the Project Definition's naming, style, and patterns?
4. Test Coverage — Are there tests for happy path, edge cases, and error cases? Are any scenarios missing?
5. Security — Any exposed secrets, unsafe inputs, missing validation, or injection risks?
6. Performance — Any obvious performance issues?
7. Cleanliness — Any debug code, TODO comments, commented-out code, or temporary hacks left behind?
8. Scope — Does the change stay within the spec's boundaries? Any scope creep?

For each issue found, provide:
- Severity: Critical / Warning / Suggestion
- Location: File and line (if applicable)
- Description: What the issue is
- Recommendation: How to fix it

If the changes look good, say so — don't invent issues.

First, run `git diff` or `git diff --cached` to see the changes, then review them.
