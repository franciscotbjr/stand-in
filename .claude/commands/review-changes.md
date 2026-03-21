Review code changes as a code reviewer before committing or creating a PR.

## Instructions

Read `design-source/project-definition.md` first. Review staged or recent changes using `git diff`.

### Review Criteria

1. **Correctness** — Does the code do what was intended? Logic errors?
2. **Completeness** — Are all acceptance criteria addressed? Anything missing?
3. **Convention Compliance** — Does it follow the Project Definition's naming, style, and patterns?
4. **Test Coverage** — Tests for happy path, edge cases, and error cases? Missing scenarios?
5. **Security** — Exposed secrets, unsafe inputs, missing validation, injection risks?
6. **Performance** — Obvious issues (N+1 queries, unnecessary loops)?
7. **Cleanliness** — Debug code, TODO comments, commented-out code, temporary hacks?
8. **Scope** — Does the change stay within boundaries? Scope creep?

### Output Format

For each issue found:
- **Severity:** Critical / Warning / Suggestion
- **Location:** File and line
- **Description:** What the issue is
- **Recommendation:** How to fix it

If the changes look good, say so — don't invent issues.
