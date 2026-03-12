# Project Definition — stand-in

> Pre-filled for a Rust library with proc macros for MCP server development.

---

## Project Identity

- **Project Name:** stand-in
- **Description:** A stand-in for your MCP server boilerplate — declarative macros that generate production-ready MCP server code at compile time
- **Project Type:** Library (Rust crate with proc macros)
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
| tokio | 1.50.0 | Async runtime |
| axum | 0.8.8 | Streamable HTTP transport (feature-gated) |

### Key Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| serde | 1.0.228 | Serialization |
| serde_json | 1.0.149 | JSON support |
| thiserror | 2.0.18 | Error derive macros |
| async-trait | 0.1.89 | Async trait support |
| tracing | 0.1.44 | Structured logging |
| syn | 2.0.117 | Proc macro parsing |
| quote | 1.0.45 | Proc macro code generation |
| proc-macro2 | 1.0.106 | Proc macro utilities |
| tower-http | 0.6.8 | HTTP middleware (CORS) |
| tokio-stream | 0.1.18 | SSE streaming |
| uuid | 1.22.0 | Session ID generation |

### Build System & Package Manager

- **Package Manager:** cargo
- **Build Tool:** cargo
- **Task Runner:** cargo

## Repository Structure

```
stand-in/
├── Cargo.toml              # workspace root
├── README.md
├── LICENSE
├── impl/                   # Design Source methodology
├── stand-in/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # re-exports, prelude
│       ├── error.rs        # Error enum
│       ├── transport/      # Stdio, Streamable HTTP
│       ├── protocol/       # JSON-RPC, MCP types
│       └── runtime/        # Server execution
└── stand-in-macros/
    ├── Cargo.toml
    └── src/
        └── lib.rs          # #[mcp_server], #[mcp_tool], etc.
```

### Key Directories

| Directory | Purpose |
|-----------|---------|
| stand-in/src/ | Main library source |
| stand-in-macros/src/ | Procedural macros |
| tests/ | Integration tests |
| examples/ | Usage examples |

## Code Conventions

### Naming

| Item | Convention | Example |
|------|-----------|---------|
| Files | snake_case | tool_registry.rs |
| Functions/Methods | snake_case | get_user_by_id |
| Types/Structs/Enums | PascalCase | ToolRegistry |
| Constants | SCREAMING_SNAKE_CASE | MAX_RETRIES |
| Modules | snake_case | http_client |
| Error variants | {Type}Error suffix | IoError, JsonError |

### Code Style

- **Formatter:** rustfmt (default settings)
- **Max Line Length:** 100
- **Indentation:** 4 spaces
- **Import Order:** std → external crates → crate internal

### Patterns & Conventions

- **Error Handling:** Custom Error enum with thiserror, `{Type}Error` suffix, manual From impls
- **API Design:** Constructor + `with_*` method chain for optional fields
- **Visibility:** Start private, expose only what's needed
- **One type per file:** Following design-source methodology
- **Facade pattern:** Each module's `mod.rs` contains only `mod` and `pub use`

## Testing

### Strategy

- **Unit Tests:** Co-located with source in `#[cfg(test)] mod tests`
- **Integration Tests:** In `tests/` directory, one file per feature area
- **Test Framework:** cargo test
- **Mocking:** mockito for HTTP mocking
- **Coverage Target:** No formal target; focus on behavior coverage

### Test Naming Convention

`test_{what}_{scenario}` — e.g., `test_tool_macro_generates_input_schema`

## Quality Gates

```bash
# Linter
cargo clippy --all-features -- -D warnings

# Formatter check
cargo fmt --check

# Tests
cargo test --all-features

# Build
cargo build --all-features

# Doc build
cargo doc --all-features --no-deps
```

## Documentation

### Required Documentation Files

| File | Purpose |
|------|---------|
| README.md | Project overview, usage, feature flags |
| CHANGELOG.md | Version history |
| LICENSE | MIT license |

### Documentation Style

- **Code Comments:** Rustdoc (`///` for public, `//` for internal)
- **Doc Examples:** `no_run` attribute on doc examples

## Deployment

- **Target Environment:** crates.io
- **CI/CD:** GitHub Actions
- **Branch Strategy:** main + feature branches

## Constraints & Non-Negotiables

- No unsafe code without justification and documentation
- All public items must have rustdoc documentation
- All types must be Send + Sync
- No `#[from]` on error variants that expose external types — use manual From impls
- Feature flags for optional functionality (stdio vs http transport)

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `stdio` | yes | Stdio transport for local/CLI usage |
| `http` | no | Streamable HTTP transport (axum-based) |
