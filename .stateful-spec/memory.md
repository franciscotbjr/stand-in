# stand-in — Project Memory

## Project

**stand-in** — A Rust library providing declarative macros (`#[mcp_tool]`, `#[mcp_server]`, `#[mcp_prompt]`) to eliminate MCP server boilerplate using convention-over-configuration.

**Status:** Active development (v0.0.3, early stage)

## Active Work

_No active iterations._

## Recent Completions

_None tracked yet — Stateful Spec initialized on 2026-03-22._

## Key Decisions

| Date | Decision | Rationale |
|------|----------|-----------|
| — | `inventory` crate for tool discovery | Zero-boilerplate registration via linker; tested on all 3 platforms |
| — | Tool errors ≠ JSON-RPC errors | MCP spec: tool execution failures are `CallToolResult.isError`, not protocol errors |
| — | One type per file + facade mod.rs | Clear organization, co-located unit tests |
| — | Rust Edition 2024 | Latest stable edition |

## Constraints & Reminders

- All quality gates must pass: `cargo fmt --check`, `cargo clippy --all-features -- -D warnings`, `cargo test --all-features`
- Multi-platform CI: Linux, macOS, Windows
- Feature-gated HTTP transport (axum) — don't assume it's always enabled
- MCP protocol version: 2025-03-26

## History Index

| # | Name | Status | Date |
|---|------|--------|------|
| — | _No iterations yet_ | — | — |
