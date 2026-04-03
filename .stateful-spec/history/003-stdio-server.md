# Iteration: 003 — Stdio Server

> Implement protocol types, tool abstractions, macros, and stdio transport for a minimal working MCP server.

## Metadata

- **Type:** feature
- **Status:** done
- **Created:** 2026-03-12
- **Completed:** 2026-03-12
- **Author:** Francisco Tarcizo Bomfim Jr

## Description

Implement everything needed for a user to write a minimal MCP server over stdio using `#[mcp_tool]` and `#[mcp_server]` macros. Covers 6 implementation layers: protocol types, tool types, server types, transport, and two proc macros.

## Acceptance Criteria

- [x] `cargo build --all-features` compiles cleanly
- [x] `cargo clippy --all-features -- -D warnings` passes
- [x] `cargo test --all-features` passes
- [x] Example server runs and responds to `initialize`, `tools/list`, `tools/call` over stdin/stdout
- [x] JSON-RPC error responses for unknown methods, missing params, invalid JSON
- [x] Tool errors returned as `CallToolResult` with `isError: true` (not JSON-RPC errors)
- [x] `Option<T>` parameters are optional in the generated schema
- [x] Graceful shutdown on stdin EOF

## Implementation Tasks

### Milestone 1: Protocol Types
- [x] Create `JsonRpcRequest` (`protocol/request.rs`)
- [x] Create `JsonRpcResponse` with `success()` / `error()` (`protocol/response.rs`)
- [x] Create `JsonRpcError` with standard codes (`protocol/error.rs`)
- [x] Create `JsonRpcNotification` (`protocol/notification.rs`)
- [x] Create `protocol/mod.rs` facade, wire into `lib.rs` and prelude
- [x] Unit tests for all protocol types

### Milestone 2: Tool Types + McpTool Trait
- [x] Create `Content`, `InputSchema`, `ToolDefinition`, `CallToolParams`, `CallToolResult`, `ListToolsResult`
- [x] Create `McpTool` trait with `to_definition()` default impl
- [x] Create `ToolRegistry` (register, list, call)
- [x] Create `tool/mod.rs` facade, wire into `lib.rs` and prelude
- [x] Unit tests for all tool types

### Milestone 3: Server Types + RequestHandler
- [x] Create `ServerCapabilities`, `ToolsCapability`, `ServerInfo`, `ClientInfo`, `InitializeParams`, `InitializeResult`
- [x] Create `RequestHandler` (dispatch initialize, tools/list, tools/call, unknown method)
- [x] Create `server/mod.rs` facade, wire into `lib.rs` and prelude
- [x] Unit tests for all server types + handler dispatch

### Milestone 4: Transport Trait + StdioTransport
- [x] Create `Transport` trait (`transport/transport_trait.rs`)
- [x] Create `StdioTransport` (stdin reader, JSON parse, dispatch, stdout writer, EOF shutdown)
- [x] Create `transport/mod.rs` facade (feature-gated), wire into `lib.rs` and prelude
- [x] Unit tests for transport

### Milestone 5: `#[mcp_tool]` Macro
- [x] Create schema inference module (`stand-in-macros/src/schema.rs`)
- [x] Implement `#[mcp_tool]` expansion with `inventory::submit!`
- [x] Add `inventory` dependency to workspace
- [x] Unit tests for schema inference + macro expansion

### Milestone 6: `#[mcp_server]` Macro
- [x] Implement `#[mcp_server]` expansion with `inventory::iter` discovery
- [x] Wire into `stand-in-macros/src/lib.rs`
- [x] Unit tests for macro expansion

### Milestone 7: Example + Integration + ARCHITECTURE.md
- [x] Create `examples/hello_server.rs`
- [x] Create `tests/stdio_server.rs` integration tests
- [x] Create `ARCHITECTURE.md`

### Milestone 8: Verify (Phase 5)
- [x] Run all quality gates (clippy, fmt, test, build, doc)
- [x] Verify all acceptance criteria
- [x] Self-review (diff, conventions, scope)
- [x] Update README, CHANGELOG, memory.md

## Quality Checks

- [x] All quality gates pass (lint, format, type check, tests, build)
- [x] Tests cover acceptance criteria (51 unit + 4 integration = 55 total)
- [x] Documentation updated (ARCHITECTURE.md)
- [x] No debug code or TODOs left behind

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
- **Commits:** ede553e, cef5e30, 2df4855, 82b868a, cd99fe7, 9a37fcc, b497b91, d731b7b
- **Related Issues:** —
