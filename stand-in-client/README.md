# stand-in-client

[![Crates.io](https://img.shields.io/crates/v/stand-in-client.svg)](https://crates.io/crates/stand-in-client)
[![Docs.rs](https://docs.rs/stand-in-client/badge.svg)](https://docs.rs/stand-in-client)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**MCP client SDK — connect, discover, and call MCP servers from Rust.**

Part of the [stand-in](https://crates.io/crates/stand-in) ecosystem (which provides the server-side macros), `stand-in-client` gives you an async, UI-agnostic client library to talk to any MCP server over stdio or Streamable HTTP. Use it from agents, CLIs, proxies, desktop apps, or anywhere you need to consume MCP tools, resources, and prompts.

## Installation

```toml
[dependencies]
stand-in-client = "0.2.0"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

### Feature Flags

| Feature  | Default | Description |
|----------|---------|-------------|
| `macros` | Yes     | `#[mcp_client]` proc macro for typed stubs |
| `http`   | No      | Streamable HTTP transport (`reqwest`-based) |
| `oauth`  | No      | OAuth 2.0 (Authorization Code + PKCE S256) for HTTP servers — implies `http` |

Enable HTTP:
```toml
stand-in-client = { version = "0.2.0", features = ["http"] }
```

## Quick Start — Dynamic API

```rust,no_run
use stand_in_client::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> stand_in_client::error::Result<()> {
    // Connect to an MCP server via stdio
    let client = Client::builder()
        .transport(StdioTransport::command("my-server", &[] as &[&str]))
        .client_info("my-app", "1.0")
        .timeout(Duration::from_secs(30))
        .connect()
        .await?;

    // Inspect what the server offers (cached after handshake)
    println!("Server: {} v{}", client.server_info().name, client.server_info().version);
    for tool in client.tools() {
        println!("  Tool: {}", tool.name);
    }

    // Call a tool
    let result = client.call_tool("greet", serde_json::json!({ "name": "World" })).await?;
    println!("{}", result.content.first().map(|c| c.as_text()).unwrap_or_default());

    // Read a resource
    let data = client.read_resource("info://version").await?;

    // Get a prompt
    let prompt = client.get_prompt("summarize", serde_json::json!({ "doc": "..." })).await?;

    // Subscribe to resource updates (HTTP transport)
    client.subscribe("file:///example").await?;

    // Listen for server notifications
    let mut notes = client.notifications();
    while let Ok(notification) = notes.recv().await {
        println!("Notification: {:?}", notification);
    }

    client.disconnect().await?;
    Ok(())
}
```

### Two Error Planes

The SDK never conflates tool errors and protocol errors:

- **Tool execution errors** (`isError: true`) are returned as **data** — `call_tool()` returns `Ok(CallToolResult)`.
- **Protocol errors** (bad JSON-RPC, handshake failure, transport closed) are returned as **`Err(Error::...)`**.

```rust,no_run
# use stand_in_client::prelude::*;
# async fn example(client: &Client) -> stand_in_client::error::Result<()> {
// Tool errors are data, not Err:
let result = client.call_tool("divide_by_zero", serde_json::json!({})).await?;
if result.is_error.unwrap_or(false) {
    // handle tool error as data — not a panic
}

// Protocol errors are Err:
match client.read_resource("bad://nonexistent").await {
    Err(Error::ProtocolError(_)) => { /* bad URI, not a resource error */ }
    _ => {}
}
# Ok(())
# }
```

## Typed API — `#[mcp_client]`

For servers whose API is known at compile time, the `#[mcp_client]` macro generates typed stubs from a trait:

```rust,ignore
use stand_in_client::prelude::*;

#[mcp_client]
pub trait Weather {
    /// Calls the tool "get_weather" (maps from method name).
    async fn get_weather(&self, city: String) -> stand_in_client::Result<String>;

    /// Uses #[tool] to map to the "add" tool and deserializes the result as i64.
    #[tool(name = "add")]
    async fn sum(&self, a: i64, b: i64) -> stand_in_client::Result<i64>;
}

// Usage:
let client = Client::builder()
    .transport(StdioTransport::command("weather-server", &[] as &[&str]))
    .client_info("typed-app", "1.0")
    .connect().await?;

let api = WeatherClient::new(client);
let greeting = api.get_weather("São Paulo".into()).await?;
let total = api.sum(40, 2).await?;

// The typed layer collapses isError into Err(Error::ToolError(...))
```

**Return type detection:** `Result<String>` extracts text directly. Any other type (`i64`, `YourStruct`, etc.) is parsed from the tool's text output with `serde_json::from_str`.

## Transports

### Stdio (default)

Launches a subprocess and communicates via newline-delimited JSON on stdin/stdout. The process is killed when the client disconnects.

```rust,no_run
# use stand_in_client::prelude::*;
let transport = StdioTransport::command("npx", &["-y", "@modelcontextprotocol/server-filesystem", "."]);
```

### Streamable HTTP (`http` feature)

Connects to MCP 2025-03-26 Streamable HTTP servers via `POST /mcp`, with automatic session management (`Mcp-Session-Id`) and an SSE stream for server-to-client notifications.

```rust,ignore
# use stand_in_client::prelude::*;
use stand_in_client::transport::HttpTransport;

let transport = HttpTransport::new("http://127.0.0.1:3000/mcp");
```

## Examples

Clone the repository and run:

```bash
# Dynamic API over stdio (auto-builds a reference server)
cargo run -p stand-in-client --example stdio_client

# Dynamic API over Streamable HTTP (+ notifications)
# First start a server: cargo run -p stand-in --example all_features --features http
cargo run -p stand-in-client --example http_client --features http

# Typed API with #[mcp_client] (self-contained, in-process)
cargo run -p stand-in-client --example typed_client --features http
```

## Architecture

- **`Client`** — high-level async API: `connect()`, `call_tool()`, `read_resource()`, `get_prompt()`, `subscribe()`, `disconnect()`.
- **`ClientTransport` trait** — the transport seam; `StdioTransport` and `HttpTransport` implement it.
- **Read-loop** — background task that correlates `id`→`oneshot` for responses and broadcasts notifications without an `id`.
- **`#[mcp_client]` macro** — procedural macro (crate `stand-in-client-macros`) that generates typed stubs from a trait definition. Feature-gated behind `macros` (on by default).

The SDK reuses the public serde types from [`stand-in`](https://crates.io/crates/stand-in) (`ToolDefinition`, `CallToolResult`, `Resource`, `Content`, `GetPromptResult`, etc.) — you never need to type `stand_in::`.

## License

MIT
