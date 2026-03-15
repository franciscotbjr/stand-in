# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.3] - 2026-03-14

### Added

- **`#[mcp_prompt]` macro** — declare reusable prompt templates with typed arguments
  - Infers argument list from function signature (`Option<T>` → optional)
  - Generates `McpPrompt` trait implementation and registers via `inventory`
  - Return type `Prompt` with `Prompt::user(text)` and `Prompt::assistant(text)` constructors
- **`prompts/list`** dispatch in `RequestHandler` — returns all registered prompts
- **`prompts/get`** dispatch in `RequestHandler` — executes a prompt by name with arguments
- **`PromptsCapability`** advertised in `initialize` response (`ServerCapabilities`)
- **`PromptRegistry`** — holds registered prompts, dispatches `get_prompt`
- **`PromptError`** variant added to `Error` enum
- `Prompt` and `PromptMessage` re-exported from `stand_in::prelude`

## [0.0.2] - 2026-03-14

### Added

- **Tracing instrumentation** across the HTTP transport execution path
  - `http_transport.rs`: `debug`/`info`/`warn` on POST/GET/DELETE handlers
  - `session_store.rs`: `info` for session create/remove, `debug` for validate
  - `handler.rs`: `info` for method dispatch, `error`/`warn` for failures
  - `sse.rs`: `trace` for SSE events, `debug` for lagged messages
  - Client disconnect detection via `StreamDropGuard` (logs when SSE stream closes)
- **ASCII startup banner** — block-letter "STAND-IN" with dynamic version and bind address, printed on HTTP server start
- `tracing-subscriber` as dev-dependency with `EnvFilter` in `examples/http_server.rs`

## [0.0.1] - 2026-03-13

### Added

- **Streamable HTTP transport** (feature: `http`) — MCP 2025-03-26 spec
  - `HttpTransport` struct with `POST/GET/DELETE /mcp` handlers
  - Session management via `Mcp-Session-Id` header (`Session`, `SessionStore`)
  - SSE notification stream on `GET /mcp`
  - CORS support via `tower-http`
  - Graceful shutdown on Ctrl+C
  - `#[mcp_server(host = "...", port = N)]` macro attributes for HTTP config
  - `serve_http()` convenience method (feature-gated)
  - `examples/http_server.rs` — minimal HTTP server example
  - 10 HTTP integration tests (full lifecycle, error cases)
- Cargo workspace with two crates: `stand-in` (library) and `stand-in-macros` (proc macros)
- Stub macros: `#[mcp_server]`, `#[mcp_tool]`, `#[mcp_resource]`, `#[mcp_prompt]`
- Custom error types with `thiserror` (`Error` enum, `Result` alias)
- Prelude module (`use stand_in::prelude::*`)
- Feature flags: `stdio` (default), `http` (optional Streamable HTTP transport)
- GitHub Actions CI workflow (build, test, lint on Linux/macOS/Windows)
- GitHub Actions publish workflow (crates.io on version tags)
- MIT LICENSE file
- Design Source methodology in `impl/`
