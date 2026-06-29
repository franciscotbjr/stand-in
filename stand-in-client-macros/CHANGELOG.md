# Changelog

All notable changes to the `stand-in-client-macros` crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

This crate is released as a pair with [`stand-in-client`](https://crates.io/crates/stand-in-client)
and shares its version line.

## [0.2.0] — 2026-06-29

Version bump released alongside `stand-in-client` 0.2.0 (which adds authorization
credentials and OAuth 2.0). The generated code and macro surface are **unchanged** since
0.1.0.

### Changed

- **Toolchain:** Rust edition 2024 and a minimum supported Rust version of
  `rust-version = "1.95.0"`.

## [0.1.0] — 2026-06-02

### Added

- **`#[mcp_client]`** — generates typed MCP client stubs from a `trait` definition: maps
  methods to tool calls, infers return-type deserialization, supports `#[tool(name = "...")]`
  overrides, and collapses `isError` results into `Err(Error::ToolError(...))`.
