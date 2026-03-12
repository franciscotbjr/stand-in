# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Cargo workspace with two crates: `stand-in` (library) and `stand-in-macros` (proc macros)
- Stub macros: `#[mcp_server]`, `#[mcp_tool]`, `#[mcp_resource]`, `#[mcp_prompt]`
- Custom error types with `thiserror` (`Error` enum, `Result` alias)
- Prelude module (`use stand_in::prelude::*`)
- Feature flags: `stdio` (default), `http` (optional Streamable HTTP transport)
- GitHub Actions CI workflow (build, test, lint on Linux/macOS/Windows)
- GitHub Actions publish workflow (crates.io on version tags)
- MIT LICENSE file
- Design Source methodology in `impl/`
