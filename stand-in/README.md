# stand-in

[![Crates.io](https://img.shields.io/crates/v/stand-in.svg)](https://crates.io/crates/stand-in)
[![Docs.rs](https://docs.rs/stand-in/badge.svg)](https://docs.rs/stand-in)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**A stand-in for your MCP server boilerplate.**

You write with `stand-in` declarative macros that look like your MCP server — tools, resources, prompts — but when the compiler rolls, the macros step aside and production-ready code takes their place. You never touch the generated code. You only ever interact with the stand-in.

## Status

🚧 **Maturing** — Core macros (`#[mcp_tool]`, `#[mcp_server]`, `#[mcp_prompt]`, `#[mcp_resource]`) and both transports (Stdio, Streamable HTTP) are implemented. The public surface is stable; APIs may still change before 1.0.

## Installation

```toml
[dependencies]
stand-in = "0.1.0"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

### Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `stdio` | ✅ | Stdio transport for local/CLI usage |
| `http`  |     | Streamable HTTP transport (MCP spec 2025-03-26) |

To enable HTTP transport:

```toml
stand-in = { version = "0.1.0", features = ["http"] }
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

### Adding a Resource

```rust
use stand_in::prelude::*;

/// Concrete resource — fixed URI, no parameters.
#[mcp_resource(
    uri = "info://version",
    name = "Server Version",
    mime_type = "application/json"
)]
async fn server_version() -> Result<String> {
    Ok(serde_json::json!({"version": "1.0.0"}).to_string())
}

/// Template resource — URI template with {param}, extracted at read time.
#[mcp_resource(
    uri = "docs://{topic}/readme",
    name = "Documentation",
    description = "Documentation for a given topic",
    mime_type = "text/markdown"
)]
async fn docs_readme(topic: String) -> Result<String> {
    Ok(format!("# {topic}\n\nDocumentation for {topic}."))
}
```

Resources with `{param}` in the URI become template resources. Concrete resources (no `{param}`) appear in `resources/list`; templates appear in `resources/templates/list`. Return `Result<Vec<u8>>` for binary data — the macro auto-detects the return type and produces base64-encoded `BlobResourceContents`.

## Features

- **`#[mcp_tool]`** — Declare a tool with typed parameters. Schema is inferred from the function signature.
- **`#[mcp_prompt]`** — Define reusable prompt templates with typed arguments. Arguments are inferred from the function signature; `Option<T>` parameters are optional.
- **`#[mcp_resource]`** — Expose data as MCP resources (concrete URIs or URI templates with `{param}`). Return `Result<String>` for text content or `Result<Vec<u8>>` for base64-encoded blobs.
- **`#[mcp_server]`** — Wire everything together. Generates initialization, capability negotiation, and dispatch.
- **Transports** — Stdio (default) and Streamable HTTP (feature-gated). Extensible via the `Transport` trait.
- **Async-first** — Built on `tokio`. Every handler is `async fn`.

## Philosophy

Inspired by frameworks like Spring Boot, `stand-in` follows a simple principle: **convention eliminates configuration**. If the shape of your code already tells us what you mean, you shouldn't have to say it twice.

The MCP protocol is well-defined but verbose to implement. Every server needs the same handshake, the same capability negotiation, the same JSON-RPC dispatch. `stand-in` absorbs all of that behind derive macros and attribute macros, so you focus on what your server *does* — not on how it speaks the protocol.

## Related Crates

| Crate | Role |
|---|---|
| [`stand-in`](https://crates.io/crates/stand-in) | The main library — runtime, transports, protocol types, re-exported macros. |
| [`stand-in-macros`](https://crates.io/crates/stand-in-macros) | Procedural macros. Generates JSON-RPC dispatch, capability advertisement, and handler wiring at compile time. (Pulled in automatically by `stand-in`.) |
| [`stand-in-client`](https://crates.io/crates/stand-in-client) | The client-side SDK — connect to, discover, and call any MCP server from Rust. |

## Why "stand-in"?

Because good infrastructure disappears.

A stand-in does essential work — they're on set for hours so the real performance can happen in minutes. But you never see them in the final cut. That's exactly what these macros do: they show up in your source code, do the hard work at compile time, and vanish from the binary.

Your code reads like a declaration of intent. The compiler turns it into a server. The stand-in was never in the final cut.

## License

MIT
