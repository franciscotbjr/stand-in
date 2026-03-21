Debug an issue by finding the root cause and proposing a minimal fix.

## Instructions

Read `design-source/project-definition.md` first. Ask the developer to describe the problem if not already provided.

### Process

1. Analyze the error and the relevant code
2. Identify the most likely root cause — explain your reasoning
3. If uncertain, list the top 2-3 possible causes ranked by likelihood
4. For each possible cause, suggest a diagnostic step to confirm or eliminate it
5. Once root cause is identified, propose a minimal fix that:
   - Addresses the root cause, not just the symptom
   - Is as small as possible
   - Does not introduce new issues
6. Suggest a regression test that would catch this bug in the future

Do NOT make broad changes or refactor during debugging. Focus on understanding and fixing the specific issue.
