# Changelog

All notable changes to `stand-in-client` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] — 2026-06-29

### Added

- **Authorization credentials** (`http` feature) — `Credential` with `NoAuth` / `Basic` / `Bearer` variants and a redacted `Debug`; `HttpTransport::with_credential(...)` injects the `Authorization` header on every HTTP verb (`POST` / `GET`-SSE / `DELETE /mcp`).
- **OAuth 2.0** (`oauth` feature, implies `http`) — Authorization Code flow with PKCE (S256), a loopback redirect listener, and token exchange + refresh. UI-agnostic `authorize(open_url)` entry point; `OAuthTokens::to_credential()` bridges issued tokens into the `Bearer` credential above; `Error::OAuthError` for the OAuth plane. Deps `sha2` / `rand` are pulled only under this feature.

### Changed

- **Edition 2024 / MSRV 1.95** — toolchain unified across the workspace (raises the minimum supported Rust version).

## [0.1.0] — 2026-06-02

### Added

- **Dynamic core** — `Client` with builder pattern: `connect()`, `call_tool()`, `read_resource()`, `get_prompt()`, `subscribe()`, `unsubscribe()`, `notifications()`, `disconnect()`.
- **Automatic handshake** — `initialize` (2025-03-26) → `notifications/initialized` → auto-fetch `tools/list`, `resources/list`, `resources/templates/list`, `prompts/list`.
- **Read-loop** — background task with alive `id`→`oneshot` correlation for responses and `broadcast` channel for server-to-client notifications.
- **Two error planes** — tool execution errors are `Ok(CallToolResult { isError: true })` (data); protocol errors are `Err(Error::...)`.
- **Stdio transport** — `StdioTransport::command()` launches a subprocess, communicates via newline-delimited JSON on stdin/stdout, kills on drop.
- **Streamable HTTP transport** (`http` feature) — `HttpTransport` with `POST`/`GET`/`DELETE /mcp`, `Mcp-Session-Id` session management, SSE stream for notifications.
- **`#[mcp_client]` macro** (`macros` feature, default) — generates typed stubs from a `trait` definition, infers return-type deserialization, collapses `isError` into `Err(Error::ToolError(...))`.
- **Reuses `stand-in` types** (`default-features = false`) — `ToolDefinition`, `CallToolResult`, `Content`, `Resource`, `ResourceTemplate`, `GetPromptResult`, `ServerInfo`, `ServerCapabilities`.
- **Examples** — `stdio_client` (dynamic, auto-builds reference server), `http_client` (dynamic + notifications), `typed_client` (macro, self-contained in-process).
- **Crate-level docs** — `#![deny(missing_docs)]`, rustdoc on every public item, doctest examples.
- **`prelude` module** — single `use stand_in_client::prelude::*` imports everything needed.
