# Changelog

All notable changes to the `stand-in-macros` crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

This crate versions in lockstep with [`stand-in`](https://crates.io/crates/stand-in).

## [0.1.0] — 2026-06-29

Minor version bump in lockstep with `stand-in` 0.1.0, signalling API maturity ahead of
1.0. The generated code and macro surface are **unchanged** since 0.0.4.

### Changed

- **Toolchain:** Rust edition 2024 and a minimum supported Rust version of
  `rust-version = "1.95.0"`.

## [0.0.4] — 2026-04-25

### Added

- **`#[mcp_resource]`** — generates the `McpResource` implementation; detects concrete vs
  template (`{param}`) resources from the URI, infers parameters from the function
  signature, and selects `TextResourceContents` vs base64 `BlobResourceContents` from the
  return type (`Result<String>` vs `Result<Vec<u8>>`).

## [0.0.3] — 2026-03-14

### Added

- **`#[mcp_prompt]`** — generates the `McpPrompt` implementation; infers the argument list
  from the function signature (`Option<T>` → optional argument).

## [0.0.1] — 2026-03-13

### Added

- **`#[mcp_tool]`** — declares a tool and infers its JSON Schema from the function
  signature.
- **`#[mcp_server]`** — generates initialization, capability negotiation, and JSON-RPC
  dispatch; accepts `host`/`port` attributes for the HTTP transport.
- Stub attribute macros registered through the `inventory` discovery pattern.
