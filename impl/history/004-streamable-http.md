# Iteration: 004 — Streamable HTTP Transport

> Implement MCP 2025-03-26 Streamable HTTP transport behind the `http` feature flag.

## Metadata

- **Type:** feature
- **Status:** in-progress
- **Created:** 2026-03-13
- **Completed:** —
- **Author:** Francisco Tomé Barros Jr

## Description

Add Streamable HTTP transport to stand-in: POST/GET/DELETE on `/mcp`, session management via `Mcp-Session-Id` header, SSE for server-initiated notifications, CORS support, and `#[mcp_server(host, port)]` macro attributes that generate a `serve_http()` convenience method.

## Acceptance Criteria

- [ ] `cargo build --all-features` compiles cleanly
- [ ] `cargo clippy --all-features -- -D warnings` passes
- [ ] `cargo test --all-features` passes
- [ ] POST `/mcp` with `initialize` creates a session and returns `Mcp-Session-Id` header
- [ ] POST `/mcp` with valid session dispatches `tools/list`, `tools/call`
- [ ] POST `/mcp` without session header (non-initialize) returns 400
- [ ] POST `/mcp` with invalid session returns 404
- [ ] GET `/mcp` with valid session returns SSE notification stream
- [ ] DELETE `/mcp` terminates session; subsequent requests fail
- [ ] `HttpTransport::default()` binds to `127.0.0.1:3000`
- [ ] `HttpTransport::new(addr)` allows custom bind address
- [ ] `#[mcp_server(host = "...", port = N)]` generates `serve_http()` using those defaults
- [ ] `#[mcp_server]` without host/port generates `serve_http()` using `HttpTransport::default()`
- [ ] Graceful shutdown on Ctrl+C
- [ ] CORS headers present on responses

## Implementation Tasks

### Milestone 1: Session Types
- [ ] Create `Session` (`transport/session.rs`)
- [ ] Create `SessionStore` (`transport/session_store.rs`)
- [ ] Wire into `transport/mod.rs` (feature-gated `http`)
- [ ] Unit tests for session create, validate, remove
- [ ] Quality gates pass, commit

### Milestone 2: SSE Helpers
- [ ] Create `sse.rs` (event formatting, notification stream builder)
- [ ] Wire into `transport/mod.rs` (feature-gated `http`)
- [ ] Unit tests for event formatting
- [ ] Quality gates pass, commit

### Milestone 3: HttpTransport + Route Handlers
- [ ] Create `HttpTransport` struct (`transport/http_transport.rs`)
- [ ] Implement POST `/mcp` handler (session creation, validation, dispatch)
- [ ] Implement GET `/mcp` handler (SSE stream)
- [ ] Implement DELETE `/mcp` handler (session termination)
- [ ] Implement `Transport` trait for `HttpTransport`
- [ ] Wire into `transport/mod.rs`, `lib.rs` prelude
- [ ] Unit tests for HttpTransport config
- [ ] Quality gates pass, commit

### Milestone 4: `#[mcp_server]` Macro Update
- [ ] Parse optional `host` and `port` attributes
- [ ] Generate `serve_http()` method (feature-gated `http`)
- [ ] Unit tests for macro expansion with/without attrs
- [ ] Quality gates pass, commit

### Milestone 5: Example + Reference Server
- [ ] Create `examples/http_server.rs`
- [ ] Update `stand-in-reference` for HTTP mode
- [ ] Quality gates pass, commit

### Milestone 6: Integration Tests
- [ ] Create `tests/http_server.rs`
- [ ] Test full lifecycle (initialize → tools/list → tools/call → DELETE)
- [ ] Test error cases (no session, invalid session, malformed JSON)
- [ ] Quality gates pass, commit

### Milestone 7: Verify (Phase 5)
- [ ] Run all quality gates
- [ ] Verify all acceptance criteria
- [ ] Self-review (diff, conventions, scope)
- [ ] Update ARCHITECTURE.md, CHANGELOG.md, memory.md

## Decisions Made

| Decision | Rationale | Date |
|----------|-----------|------|
| `Arc<RequestHandler>` for concurrency | RequestHandler is Send + Sync, simple sharing | 2026-03-13 |
| In-memory session store | Simple, sufficient for single-process servers | 2026-03-13 |
| Broadcast channel for SSE | Multiple GET listeners per session, tokio native | 2026-03-13 |
| Macro-level host/port with runtime override | Convention + flexibility | 2026-03-13 |

## Blockers & Notes

- `tokio-stream` needs `sync` feature for `BroadcastStream` — add during M2
- `reqwest` needed as dev-dependency for integration tests — add during M6

## References

- **Specification:** `C:\Users\franciscotbjr\.windsurf\plans\streamable-http-transport-51994b.md`
- **PR/MR:** —
- **Commits:** —
- **Related Issues:** —
