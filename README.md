# stand-in

[![Build](https://github.com/franciscotbjr/stand-in/actions/workflows/build.yml/badge.svg)](https://github.com/franciscotbjr/stand-in/actions/workflows/build.yml)
[![Crates.io](https://img.shields.io/crates/v/stand-in.svg)](https://crates.io/crates/stand-in)
[![Docs.rs](https://docs.rs/stand-in/badge.svg)](https://docs.rs/stand-in)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**A stand-in for your MCP server boilerplate.**

You write with `stand-in`  declarative macros that look like your MCP server — tools, resources, prompts — but when the compiler rolls, the macros step aside and production-ready code takes their place. You never touch the generated code. You only ever interact with the stand-in.

## Status

🚧 **Early Development** — Core macros (`#[mcp_tool]`, `#[mcp_server]`) and both transports (Stdio, Streamable HTTP) are implemented. Resources and prompts are not yet available.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
stand-in = "0.0.2"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

### Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `stdio` | ✅ | Stdio transport for local/CLI usage |
| `http` | ✅ | Streamable HTTP transport (MCP spec 2025-03-26) |

To enable HTTP transport:

```toml
stand-in = { version = "0.0.2", features = ["http"] }
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
- **`#[mcp_resource]`** — Expose a resource with URI templates. Supports static and dynamic content.
- **`#[mcp_prompt]`** — Define reusable prompt templates with typed arguments.
- **`#[mcp_server]`** — Wire everything together. Generates initialization, capability negotiation, and dispatch.
- **Transports** — Stdio (default) and Streamable HTTP (feature-gated). Extensible via the `Transport` trait.
- **Async-first** — Built on `tokio`. Every handler is `async fn`.

## Example: A More Complete Server

```rust
use stand_in::prelude::*;

/// A server that exposes project management tools and resources.
#[mcp_server(
    name = "project-hub",
    version = "0.1.0"
)]
struct ProjectHub;

#[mcp_tool(description = "List all open tasks for a project")]
async fn list_tasks(project_id: String) -> Result<Vec<Task>> {
    db::tasks::find_open(&project_id).await
}

#[mcp_tool(description = "Create a new task")]
async fn create_task(project_id: String, title: String, assignee: Option<String>) -> Result<Task> {
    db::tasks::create(&project_id, &title, assignee.as_deref()).await
}

#[mcp_resource(
    uri = "project://{project_id}/readme",
    description = "The project README"
)]
async fn project_readme(project_id: String) -> Result<String> {
    storage::read_file(&project_id, "README.md").await
}

#[mcp_prompt(
    name = "summarize_project",
    description = "Generate a project status summary"
)]
async fn summarize_project(project_id: String, format: Option<String>) -> Result<Prompt> {
    let tasks = list_tasks(project_id.clone()).await?;
    Ok(Prompt::user(format!(
        "Summarize this project (format: {}). Open tasks: {:?}",
        format.unwrap_or("brief".into()),
        tasks
    )))
}

#[tokio::main]
async fn main() {
    ProjectHub::serve(StdioTransport::default()).await;
}
```

## Why "stand-in"?

Because good infrastructure disappears.

A stand-in does essential work — they're on set for hours so the real performance can happen in minutes. But you never see them in the final cut. That's exactly what these macros do: they show up in your source code, do the hard work at compile time, and vanish from the binary.

Your code reads like a declaration of intent. The compiler turns it into a server. The stand-in was never in the final cut.

## License

MIT