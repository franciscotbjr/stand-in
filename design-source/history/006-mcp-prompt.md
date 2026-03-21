# Iteration 006 — `#[mcp_prompt]` Support

- **Type:** feature
- **Status:** done
- **Completed:** 2026-03-14
- **Started:** 2026-03-14
- **Author:** Francisco Tomé Barros Jr.

## Goal

Implement the `#[mcp_prompt]` macro and the full prompt subsystem (types, registry, dispatch, capability advertisement), following the identical patterns used for `#[mcp_tool]`.

## Milestones

- [x] M1 — Core protocol types (`stand-in/src/prompt/`)
- [x] M2 — `McpPrompt` trait, `PromptFactory`, `PromptRegistry`
- [x] M3 — `#[mcp_prompt]` macro (`stand-in-macros/src/mcp_prompt.rs`)
- [x] M4 — Server integration (`RequestHandler`, `ServerCapabilities`, `mcp_server` macro)
- [x] M5 — Error variant + library exports
- [x] M6 — Tests, CHANGELOG, README

## Decisions

- `McpPrompt::execute()` returns `Result<Prompt>` directly (no wrapping, unlike `CallToolResult::text()`)
- `Prompt::user(text)` and `Prompt::assistant(text)` constructors
- `PromptArgument` name inferred from function param ident; `required` inferred from `Option<T>`
- `GetPromptResult` built in the registry by combining prompt definition description + execute result messages
- `PromptError(String)` variant added to `Error` enum (same pattern as `ToolError`)
