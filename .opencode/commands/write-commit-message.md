---
description: Generate a well-structured commit message
agent: build
---

Please write a commit message for the following changes.

## Rules
1. Use a short, imperative subject line (50 characters max) — e.g., "Add user authentication endpoint"
2. Leave a blank line after the subject
3. Write a body explaining WHAT changed and WHY (not HOW — the code shows how)
4. If this closes an issue or relates to a ticket, include the reference
5. Keep the total message concise — aim for clarity, not completeness

Format:
```
<subject line>

<body — what changed and why>

<optional: references>
```

First, run `git diff` and `git diff --cached` to see the changes, then generate the message.
