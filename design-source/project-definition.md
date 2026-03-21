# Project Definition — stand-in

## Project Identity

- **Project Name:** stand-in
- **Description:** A declarative macro framework for building MCP servers in Rust with zero boilerplate
- **Project Type:** library
- **Repository URL:** https://github.com/franciscotbjr/stand-in
- **License:** MIT
- **MCP Protocol Version:** 2025-03-26

## Technology Stack

### Language(s)

| Language | Version | Role |
|----------|---------|------|
| Rust | Edition 2024 | Primary |

### Framework(s)

| Framework | Version | Purpose |
|-----------|---------|---------|
| tokio | 1.x | Async runtime |
| axum | 0.8.x | HTTP transport (feature-gated) |

### Key Dependencies

| Dependency | Purpose |
|------------|---------|
| serde / serde_json | Serialization / deserialization |
| thiserror | Error derive macros |
| async-trait | Async trait support |
| inventory | Automatic tool/prompt registration (service discovery) |
| tracing | Logging / instrumentation |
| syn / quote / proc-macro2 | Procedural macro generation |
| tower-http | HTTP middleware (feature-gated) |
| uuid | Session IDs (feature-gated) |

### Build System & Package Manager

- **Package Manager:** cargo
- **Build Tool:** cargo
- **Task Runner:** cargo

## Repository Structure

```
stand-in/
├── Cargo.toml                  # Workspace root
├── stand-in/                   # Main library crate
│   ├── src/
│   │   ├── lib.rs              # Crate root, prelude, re-exports
│   │   ├── error.rs            # Error enum + Result alias
│   │   ├── prompt/             # MCP prompt abstraction
│   │   ├── protocol/           # JSON-RPC 2.0 wire format
│   │   ├── server/             # MCP server types + dispatch
│   │   ├── tool/               # MCP tool abstraction
│   │   └── transport/          # Communication layer (stdio, http)
│   ├── tests/                  # Integration tests
│   └── examples/               # Usage examples
├── stand-in-macros/            # Procedural macros crate
│   └── src/
│       ├── lib.rs              # Proc macro entry points
│       ├── mcp_tool.rs         # #[mcp_tool] expansion
│       ├── mcp_server.rs       # #[mcp_server] expansion
│       ├── mcp_prompt.rs       # #[mcp_prompt] expansion
│       └── schema.rs           # Rust type → JSON Schema inference
├── stand-in-reference/         # Reference/test server (not published)
│   └── src/main.rs             # Demo with greet, add, echo tools
├── design-source/              # Design Source methodology tracking
├── .claude/                    # Claude Code configuration
├── .github/workflows/          # CI/CD pipelines
├── README.md
├── ARCHITECTURE.md
├── CHANGELOG.md
└── LICENSE
```

### Key Directories

| Directory | Purpose |
|-----------|---------|
| stand-in/src/ | Main library — transport, server, tool, prompt, protocol modules |
| stand-in-macros/src/ | Procedural macros (#[mcp_tool], #[mcp_server], #[mcp_prompt]) |
| stand-in-reference/src/ | Reference MCP server for testing and demos (not published) |
| stand-in/tests/ | Integration tests (one file per transport) |
| stand-in/examples/ | Usage examples (hello_server, http_server) |

## Code Conventions

### Naming

| Item | Convention | Example |
|------|-----------|---------|
| Files | snake_case | tool_registry.rs |
| Functions/Methods | snake_case | get_prompt_result |
| Types/Structs/Enums | PascalCase | ToolDefinition |
| Constants | SCREAMING_SNAKE_CASE | MAX_RETRIES |
| Modules | snake_case | http_transport |
| Lifetimes | lowercase, short | 'a, 'de |

### Code Style

- **Formatter:** rustfmt (default settings)
- **Max Line Length:** 100 (rustfmt default)
- **Indentation:** 4 spaces
- **Import Order:** std → external crates → crate internal → super/self

### Patterns & Conventions

- **Error Handling:** Custom Error enum with thiserror, Result type alias, manual From impls for external types
- **API Design:** Constructor with required fields + with_* method chain for optional fields
- **Visibility:** Start private, expose only what's needed. Use pub(crate) and pub(super) for internal sharing
- **Documentation:** Rustdoc on all public items

## Testing

### Strategy

| Aspect | Detail |
|--------|--------|
| Unit Tests | Co-located with source in `#[cfg(test)] mod tests` |
| Integration Tests | `stand-in/tests/` — one file per transport |
| Framework | Built-in Rust testing + `#[tokio::test]` for async |
| Naming | `test_{what}_{scenario}` |

### Current Tests

- `stand-in/tests/stdio_server.rs` — 4 tests (full lifecycle, unknown methods, malformed JSON, unknown tools)
- `stand-in/tests/http_server.rs` — 11 tests (initialization, session management, tool calls, prompts, error handling)

## Quality Gates

```bash
# Formatter check
cargo fmt --check

# Linter
cargo clippy --all-features -- -D warnings

# Tests
cargo test --all-features

# Build
cargo build --all-features

# Doc build
cargo doc --all-features --no-deps
```

## Feature Flags

| Feature | Default | Purpose |
|---------|---------|---------|
| stdio | Yes | Stdio transport |
| http | No | Streamable HTTP transport (axum, tower-http, uuid) |

## CI/CD

- **Platform:** GitHub Actions
- **Build pipeline** (`build.yml`): Format check, clippy, tests (multi-platform: Ubuntu, macOS, Windows), doc build
- **Publish pipeline** (`publish.yml`): cargo-audit, cargo nextest (Linux + Windows), publish to crates.io via cargo-release
- **Branch strategy:** main + feature branches

## Deployment

- **Target Environment:** crates.io
- **Versioning:** Semantic versioning, tags trigger publish

## Constraints & Non-Negotiables

- No unsafe code without justification and documentation
- All public items must have rustdoc documentation
- Feature flags for optional functionality
- No `#[from]` on error variants that expose external types — use manual From impls
- Minimize diff when modifying existing code
- All quality gates must pass before merging
