# Project Memory

> This file is the AI's entry point for understanding the project's current state. Keep it updated as work progresses.

## Project Summary

- **Project:** stand-in
- **Description:** A stand-in for your MCP server boilerplate — declarative macros that generate production-ready MCP server code at compile time
- **Last Updated:** 2026-03-12
- **Current Status:** Active development

## Active Work

> What is currently in progress? Reference the iteration file.

- None — ready for next feature

## Recent Completions

> Last 3-5 completed iterations for quick context.

| # | Name | Type | Completed |
|---|------|------|-----------|
| 003 | Stdio Server | feature | 2026-03-12 |
| 002 | CI/CD Setup | chore | 2026-03-12 |
| 001 | Project Setup | chore | 2026-03-11 |

## Key Decisions

> Important decisions that affect how the AI should work on this project. For detailed ADRs, see `history/` files.

- **Workspace structure** — Two crates: `stand-in` (main lib) and `stand-in-macros` (proc macros)
- **Transport strategy** — Both Stdio and Streamable HTTP via feature flags
- **Schema inference** — `#[mcp_tool]` macro infers JSON Schema from Rust function signatures
- **Error handling** — Custom Error enum with `{Type}Error` suffix convention
- **CI/CD** — GitHub Actions for build (push/PR) and publish (version tags)
- **Tool discovery** — `inventory` crate for zero-boilerplate auto-registration of `#[mcp_tool]` functions
- **MCP protocol** — Version 2025-03-26, server identity auto-derived from Cargo.toml via `env!()`

## Constraints & Reminders

> Things the AI must always remember when working on this project.

- No unsafe code without justification and documentation
- All public items must have rustdoc documentation
- All types must be Send + Sync
- No `#[from]` on error variants that expose external types — use manual From impls
- Feature flags for optional functionality (stdio vs http transport)
- One type per file, facade pattern in `mod.rs`

## History Index

> Complete list of iterations. Newest first.

| # | Name | Type | Status | File |
|---|------|------|--------|------|
| 003 | Stdio Server | feature | done | `history/003-stdio-server.md` |
| 002 | CI/CD Setup | chore | done | `history/002-ci-cd-setup.md` |
| 001 | Project Setup | chore | done | `history/001-project-setup.md` |

## How to Use This File

1. **AI assistants:** Read this file first when joining the project. It provides context about what's happening and what to remember.
2. **Developers:** Update this file when starting or completing work. Keep the Active Work and History Index current.
3. **New team members:** This file + the Project Definition give you everything needed to onboard an AI assistant.
