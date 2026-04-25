---
description: Diagnose and fix a bug with structured root cause analysis
agent: build
---

I need help debugging an issue. Please help me find the root cause and fix it.

## Instructions
1. Analyze the error and the relevant code
2. Identify the most likely root cause — explain your reasoning
3. If you're not certain, list the top 2-3 possible causes ranked by likelihood
4. For each possible cause, suggest a diagnostic step to confirm or eliminate it
5. Once the root cause is identified, propose a minimal fix
6. The fix should:
   - Address the root cause, not just the symptom
   - Be as small as possible — prefer single-line changes when sufficient
   - Not introduce new issues
7. Suggest a regression test that would catch this bug in the future

Do NOT make broad changes or refactor during debugging. Focus on understanding and fixing the specific issue.

Ask the developer for:
- The problem description (error message, unexpected behavior, steps to reproduce)
- The relevant code (file references)
- What they've already tried (if anything)
