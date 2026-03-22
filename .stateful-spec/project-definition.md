# Project Definition — stand-in

## Project Identity

- **Project Name:** stand-in
- **Description:** A Rust library that provides declarative macros to eliminate MCP server boilerplate — tools, resources, prompts — using convention-over-configuration.
- **Project Type:** library (Cargo workspace)
- **Repository URL:** https://github.com/franciscotbjr/stand-in
- **License:** MIT

## Technology Stack

### Language(s)

| Language | Version | Role |
|----------|---------|------|
| Rust | Edition 2024 | Primary |

### Framework(s)

| Framework | Version | Purpose |
|-----------|---------|---------|
| tokio | 1.50.0 | Async runtime (rt-multi-thread, macros, signal, io-util, io-std) |
| axum | 0.8.8 | HTTP transport (feature-gated) |

### Key Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| serde | 1.0.228 | Serialization / deserialization |
| serde_json | 1.0.149 | JSON support |
| thiserror | 2.0.18 | Error derive macros |
| async-trait | 0.1.89 | Async trait support |
| tracing | 0.1.44 | Structured logging |
| tracing-subscriber | 0.3.22 | Tracing output (env-filter) |
| tower-http | 0.6.8 | CORS middleware (http feature) |
| uuid | 1.22.0 | Session ID generation (http feature) |
| syn | 2.0.117 | Proc macro AST parsing |
| quote | 1.0.45 | Proc macro code generation |
| proc-macro2 | 1.0.106 | Proc macro primitives |
| inventory | 0.3 | Tool/prompt discovery via linker |
| tokio-stream | 0.1.18 | Async streams (http feature) |

### Build System & Package Manager

- **Package Manager:** cargo
- **Build Tool:** cargo
- **Task Runner:** cargo

## Repository Structure

```
stand-in/
├── Cargo.toml              # Workspace root
├── Cargo.lock
├── stand-in/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs           # Crate root with prelude export
│   │   ├── error.rs         # Centralized Error enum + Result<T> alias
│   │   ├── protocol/        # JSON-RPC 2.0 wire format
│   │   ├── tool/            # MCP tool abstraction layer
│   │   ├── prompt/          # MCP prompt template system
│   │   ├── server/          # MCP server types + request dispatch
│   │   └── transport/       # Communication layer (stdio, HTTP)
│   └── tests/               # Integration tests
├── stand-in-macros/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs           # Proc macro entry points
│       ├── mcp_tool.rs      # #[mcp_tool] expansion
│       ├── mcp_server.rs    # #[mcp_server] expansion
│       ├── mcp_prompt.rs    # #[mcp_prompt] expansion
│       └── schema.rs        # Rust type → JSON Schema inference
├── stand-in-reference/
│   ├── Cargo.toml
│   └── src/main.rs          # Reference server (3 tools, not published)
├── .stateful-spec/          # Stateful Spec methodology tracking
├── .claude/                 # Claude Code configuration
├── .github/workflows/       # CI/CD (build + publish)
├── ARCHITECTURE.md
├── CHANGELOG.md
├── README.md
└── LICENSE
```

### Key Directories

| Directory | Purpose |
|-----------|---------|
| stand-in/src/ | Core library source code |
| stand-in-macros/src/ | Procedural macro implementations |
| stand-in-reference/src/ | Reference implementation for testing |
| stand-in/tests/ | Integration tests (stdio + HTTP transports) |

## Code Conventions

### Naming

| Item | Convention | Example |
|------|-----------|---------|
| Files | snake_case | tool_definition.rs |
| Functions/Methods | snake_case | get_user_by_id |
| Types/Structs/Enums | PascalCase | ToolDefinition |
| Constants | SCREAMING_SNAKE_CASE | MAX_RETRIES |
| Modules | snake_case | http_transport |
| Lifetimes | lowercase, short | 'a, 'de |
| Type aliases | PascalCase | Result\<T\> |

### Code Style

- **Formatter:** rustfmt (default settings, no rustfmt.toml)
- **Indentation:** 4 spaces
- **Import Order:** std → external crates → crate internal → super/self

### Patterns & Conventions

- **Module organization:** One type per file, facade `mod.rs` re-exports
- **Error handling:** Centralized `Error` enum with `thiserror`, `Result<T>` type alias, manual `From` impls
- **Tool errors vs protocol errors:** Tool execution errors → `CallToolResult { isError: true }`, JSON-RPC errors → protocol-level only
- **Serde:** `#[serde(rename = "camelCase")]` for JSON protocol compliance, `#[serde(skip_serializing_if = "Option::is_none")]`
- **Builder pattern:** `InputSchema::object().with_properties(...)`, `Prompt::user(text)`
- **API Design:** Constructor with required fields + `with_*` method chain for optional fields
- **Visibility:** Start private, expose only what's needed. Use `pub(crate)` and `pub(super)`
- **Async:** All handlers are `async fn`, `async-trait` for trait methods
- **Logging:** `tracing` crate with structured fields (info, debug, warn, error levels)
- **Tool discovery:** `inventory` crate pattern — `#[mcp_tool]` generates `ToolFactory`, `#[mcp_server]` collects via `inventory::iter`
- **Documentation:** Rustdoc on public items with `//!` module-level docs

## Testing

### Strategy

- **Unit Tests:** Co-located with source in `#[cfg(test)] mod tests`
- **Integration Tests:** In `stand-in/tests/` directory, one file per transport
  - `stdio_server.rs` — Spawns reference binary, tests JSON-RPC over stdin/stdout
  - `http_server.rs` — Feature-gated (`#[cfg(feature = "http")]`), spawns HTTP server on random port
- **Test Framework:** cargo test
- **HTTP Testing:** reqwest client
- **Coverage Target:** No formal target; focus on behavior coverage

### Test Naming Convention

`test_{what_is_being_tested}_{scenario}` — e.g., `test_full_lifecycle`, `test_unknown_tool_error`

## Quality Gates

```bash
# Formatter check
cargo fmt --check

# Linter (all warnings are errors)
cargo clippy --all-features -- -D warnings

# Tests (all features, all platforms)
cargo test --all-features

# Build
cargo build --all-features

# Doc build
cargo doc --all-features --no-deps
```

### CI Pipeline

- **Build workflow:** format → clippy → test (Linux/macOS/Windows) → build → docs
- **Publish workflow:** cargo-audit → nextest (Linux + Windows) → crates.io publish on `v*` tags

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `stdio` | Yes | Stdio transport for local/CLI usage |
| `http` | No | Streamable HTTP transport (axum, sessions, CORS, SSE) |

## Documentation

### Required Documentation Files

| File | Purpose |
|------|---------|
| README.md | Project overview, usage, feature flags |
| CHANGELOG.md | Version history (Keep a Changelog format) |
| ARCHITECTURE.md | Design decisions, module structure |
| LICENSE | MIT license file |

### Documentation Style

- **Code Comments:** Rustdoc (`///` for public, `//` for internal)

## Deployment

- **Target Environment:** crates.io
- **CI/CD:** GitHub Actions
- **Branch Strategy:** main + feature branches
- **Versioning:** Workspace-level version (currently 0.0.3)

## Constraints & Non-Negotiables

- No unsafe code without justification and documentation
- All public items must have rustdoc documentation
- All clippy warnings treated as errors (`-D warnings`)
- Feature flags for optional functionality (HTTP transport is opt-in)
- MCP protocol version target: 2025-03-26
- Server identity auto-derived from Cargo.toml via `env!("CARGO_PKG_NAME")` / `env!("CARGO_PKG_VERSION")`
- Multi-platform support required (Linux, macOS, Windows)
