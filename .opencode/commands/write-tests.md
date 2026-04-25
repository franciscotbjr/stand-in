---
description: Generate tests for existing or new code
agent: build
---

Please write tests for the following code. Follow the project's testing conventions from the Project Definition.

## Instructions
1. Use the testing framework and patterns specified in the Project Definition
2. Follow the project's test naming convention
3. Write tests for:
   - Happy path — Normal, expected usage
   - Edge cases — Boundary values, empty inputs, large inputs, null/undefined
   - Error cases — Invalid inputs, failure scenarios, error handling
4. Each test should be independent — no test should depend on another test's state
5. Use descriptive test names that explain what is being verified
6. Include setup/teardown if needed (fixtures, mocks, test data)
7. Keep tests focused — one assertion concept per test

Place tests in the location specified by the Project Definition.

Ask the developer for:
- The code to be tested (file references)
- Optionally: a specification with test scenarios
