# stand-in

[![Build](https://github.com/franciscotbjr/stand-in/actions/workflows/build.yml/badge.svg)](https://github.com/franciscotbjr/stand-in/actions/workflows/build.yml)
[![Crates.io](https://img.shields.io/crates/v/stand-in.svg)](https://crates.io/crates/stand-in)
[![Docs.rs](https://docs.rs/stand-in/badge.svg)](https://docs.rs/stand-in)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**A stand-in for your MCP server boilerplate.**

You write with `stand-in`  declarative macros that look like your MCP server — tools, resources, prompts — but when the compiler rolls, the macros step aside and production-ready code takes their place. You never touch the generated code. You only ever interact with the stand-in.

## Status

🚧 **Early Development** — Core macros (`#[mcp_tool]`, `#[mcp_server]`, `#[mcp_prompt]`) and both transports (Stdio, Streamable HTTP) are implemented. Resources are not yet available.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
stand-in = "0.0.3"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

### Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `stdio` | ✅ | Stdio transport for local/CLI usage |
| `http` | ✅ | Streamable HTTP transport (MCP spec 2025-03-26) |

To enable HTTP transport:

```toml
stand-in = { version = "0.0.3", features = ["http"] }
```

## Quick Start

```rust
use stand_in::prelude::*;

#[mcp_tool(
    name = "get_weather",
    description = "Returns current weather for a given city"
)]
async fn get_weather(city: String) -> Result<String> {
    let forecast = fetch_weather(&city).await?;
    Ok(format!("{}: {}°C, {}", city, forecast.temp, forecast.condition))
}

#[mcp_server]
struct MyServer;

#[tokio::main]
async fn main() {
    MyServer::serve(StdioTransport::default()).await;
}
```

That's it. No handler registration. No JSON-RPC routing. No protocol plumbing. The stand-in handles the setup; the compiler delivers the performance.

### Adding a Prompt

```rust
use stand_in::prelude::*;

#[mcp_prompt(
    name = "summarize",
    description = "Summarize a document for a given audience"
)]
async fn summarize(document: String, audience: Option<String>) -> Result<Prompt> {
    let level = audience.as_deref().unwrap_or("general");
    Ok(Prompt::user(format!(
        "Summarize the following for a {level} audience:\n\n{document}"
    )))
}
```

`Option<T>` parameters become optional arguments in the MCP prompt definition. Required parameters stay required. The return type is always `Result<Prompt>`.

## Philosophy

Inspired by frameworks like Spring Boot, `stand-in` follows a simple principle: **convention eliminates configuration**. If the shape of your code already tells us what you mean, you shouldn't have to say it twice.

The MCP protocol is well-defined but verbose to implement. Every server needs the same handshake, the same capability negotiation, the same JSON-RPC dispatch. `stand-in` absorbs all of that behind derive macros and attribute macros, so you focus on what your server *does* — not on how it speaks the protocol.

## Workspace Structure

The project is organized as a Cargo workspace with two crates:

| Crate | Role |
|---|---|
| `stand-in` | The main library. Re-exports macros, provides runtime, transport, and protocol types. |
| `stand-in-macros` | Procedural macros. Generates the JSON-RPC dispatch, capability advertisement, and handler wiring at compile time. |

```
stand-in/
├── Cargo.toml              # workspace root
├── stand-in/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs           # re-exports, runtime, transports
│       └── ...
└── stand-in-macros/
    ├── Cargo.toml
    └── src/
        └── lib.rs           # proc macros: #[mcp_server], #[mcp_tool], etc.
```

## Features

- **`#[mcp_tool]`** — Declare a tool with typed parameters. Schema is inferred from the function signature.
- **`#[mcp_prompt]`** — Define reusable prompt templates with typed arguments. Arguments are inferred from the function signature; `Option<T>` parameters are optional.
- **`#[mcp_server]`** — Wire everything together. Generates initialization, capability negotiation, and dispatch.
- **Transports** — Stdio (default) and Streamable HTTP (feature-gated). Extensible via the `Transport` trait.
- **Async-first** — Built on `tokio`. Every handler is `async fn`.
- **`#[mcp_resource]`** — _(not yet implemented)_

## Example: A More Complete Server

```rust
use stand_in::prelude::*;

/// A server that exposes project management tools and prompts.
/// Server name and version are read from Cargo.toml at compile time.
#[mcp_server]
struct ProjectHub;

#[mcp_tool(name = "list_tasks", description = "List all open tasks for a project")]
async fn list_tasks(project_id: String) -> Result<String> {
    // ... fetch from database
    Ok(format!("Tasks for {project_id}: ..."))
}

#[mcp_tool(name = "create_task", description = "Create a new task")]
async fn create_task(project_id: String, title: String, assignee: Option<String>) -> Result<String> {
    // ... write to database
    Ok(format!("Created task '{title}' in {project_id}"))
}

#[mcp_prompt(
    name = "summarize_project",
    description = "Generate a project status summary"
)]
async fn summarize_project(project_id: String, format: Option<String>) -> Result<Prompt> {
    let level = format.as_deref().unwrap_or("brief");
    Ok(Prompt::user(format!(
        "Summarize project {project_id} in a {level} format."
    )))
}

#[tokio::main]
async fn main() {
    ProjectHub::serve(StdioTransport::default()).await.unwrap();
}
```

## Why "stand-in"?

Because good infrastructure disappears.

A stand-in does essential work — they're on set for hours so the real performance can happen in minutes. But you never see them in the final cut. That's exactly what these macros do: they show up in your source code, do the hard work at compile time, and vanish from the binary.

Your code reads like a declaration of intent. The compiler turns it into a server. The stand-in was never in the final cut.

## License

MIT