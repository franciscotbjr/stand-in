Write tests for existing or newly implemented code.

## Instructions

Read `design-source/project-definition.md` first. Ask the developer what code needs tests if not already provided.

### Process

1. Use the testing framework from the Project Definition (built-in Rust tests + tokio::test for async)
2. Follow the test naming convention: `test_{what}_{scenario}`
3. Write tests for:
   - **Happy path** — Normal, expected usage
   - **Edge cases** — Boundary values, empty inputs, large inputs
   - **Error cases** — Invalid inputs, failure scenarios, error handling
4. Each test should be independent — no test should depend on another test's state
5. Use descriptive test names that explain what is being verified
6. Include setup/teardown if needed

### Placement

- Unit tests: Co-located with source in `#[cfg(test)] mod tests`
- Integration tests: `stand-in/tests/` directory
