# Iteration: 003 — Stdio Server

> Implement protocol types, tool abstractions, macros, and stdio transport for a minimal working MCP server.

## Metadata

- **Type:** feature
- **Status:** in-progress
- **Created:** 2026-03-12
- **Completed:** —
- **Author:** Francisco Tomé Barros Jr

## Description

Implement everything needed for a user to write a minimal MCP server over stdio using `#[mcp_tool]` and `#[mcp_server]` macros. Covers 6 implementation layers: protocol types, tool types, server types, transport, and two proc macros.

## Acceptance Criteria

- [ ] `cargo build --all-features` compiles cleanly
- [ ] `cargo clippy --all-features -- -D warnings` passes
- [ ] `cargo test --all-features` passes
- [ ] Example server runs and responds to `initialize`, `tools/list`, `tools/call` over stdin/stdout
- [ ] JSON-RPC error responses for unknown methods, missing params, invalid JSON
- [ ] Tool errors returned as `CallToolResult` with `isError: true` (not JSON-RPC errors)
- [ ] `Option<T>` parameters are optional in the generated schema
- [ ] Graceful shutdown on stdin EOF

## Implementation Tasks

### Milestone 1: Protocol Types
- [ ] Create `JsonRpcRequest` (`protocol/request.rs`)
- [ ] Create `JsonRpcResponse` with `success()` / `error()` (`protocol/response.rs`)
- [ ] Create `JsonRpcError` with standard codes (`protocol/error.rs`)
- [ ] Create `JsonRpcNotification` (`protocol/notification.rs`)
- [ ] Create `protocol/mod.rs` facade, wire into `lib.rs` and prelude
- [ ] Unit tests for all protocol types

### Milestone 2: Tool Types + McpTool Trait
- [ ] Create `Content`, `InputSchema`, `ToolDefinition`, `CallToolParams`, `CallToolResult`, `ListToolsResult`
- [ ] Create `McpTool` trait with `to_definition()` default impl
- [ ] Create `ToolRegistry` (register, list, call)
- [ ] Create `tool/mod.rs` facade, wire into `lib.rs` and prelude
- [ ] Unit tests for all tool types

### Milestone 3: Server Types + RequestHandler
- [ ] Create `ServerCapabilities`, `ToolsCapability`, `ServerInfo`, `ClientInfo`, `InitializeParams`, `InitializeResult`
- [ ] Create `RequestHandler` (dispatch initialize, tools/list, tools/call, unknown method)
- [ ] Create `server/mod.rs` facade, wire into `lib.rs` and prelude
- [ ] Unit tests for all server types + handler dispatch

### Milestone 4: Transport Trait + StdioTransport
- [ ] Create `Transport` trait (`transport/transport_trait.rs`)
- [ ] Create `StdioTransport` (stdin reader, JSON parse, dispatch, stdout writer, EOF shutdown)
- [ ] Create `transport/mod.rs` facade (feature-gated), wire into `lib.rs` and prelude
- [ ] Unit tests for transport

### Milestone 5: `#[mcp_tool]` Macro
- [ ] Create schema inference module (`stand-in-macros/src/schema.rs`)
- [ ] Implement `#[mcp_tool]` expansion with `inventory::submit!`
- [ ] Add `inventory` dependency to workspace
- [ ] Unit tests for schema inference + macro expansion

### Milestone 6: `#[mcp_server]` Macro
- [ ] Implement `#[mcp_server]` expansion with `inventory::iter` discovery
- [ ] Wire into `stand-in-macros/src/lib.rs`
- [ ] Unit tests for macro expansion

### Milestone 7: Example + Integration + ARCHITECTURE.md
- [ ] Create `examples/hello_server.rs`
- [ ] Create `tests/stdio_server.rs` integration tests
- [ ] Create `ARCHITECTURE.md`

### Milestone 8: Verify (Phase 5)
- [ ] Run all quality gates (clippy, fmt, test, build, doc)
- [ ] Verify all acceptance criteria
- [ ] Self-review (diff, conventions, scope)
- [ ] Update README, CHANGELOG, memory.md

## Quality Checks

- [ ] All quality gates pass (lint, format, type check, tests, build)
- [ ] Tests cover acceptance criteria (~50 unit + 4 integration)
- [ ] Documentation updated (ARCHITECTURE.md, README, CHANGELOG)
- [ ] No debug code or TODOs left behind

## Decisions Made

| Decision | Rationale | Date |
|----------|-----------|------|
| `inventory` for tool discovery | Zero boilerplate, auto-registration via `inventory::submit!` | 2026-03-12 |
| Protocol version 2025-03-26 | Latest MCP spec | 2026-03-12 |
| Server name/version from Cargo.toml | Auto-derived via `env!()`, no macro attrs needed | 2026-03-12 |
| One type per file | Project convention from project-definition.md | 2026-03-12 |

## Blockers & Notes

- `inventory` crate uses linker tricks — works on Linux, macOS, Windows but verify in CI

## References

- **Specification:** `C:\Users\franciscotbjr\.windsurf\plans\stdio-server-spec-e6e21b.md`
- **PR/MR:** —
- **Commits:** —
- **Related Issues:** —
