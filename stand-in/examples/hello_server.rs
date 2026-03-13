//! A minimal MCP server that exposes a single "greet" tool.
//!
//! Run: `cargo run --example hello_server`
//! Then send JSON-RPC over stdin:
//!
//! ```json
//! {"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"0.1.0"}}}
//! {"jsonrpc":"2.0","method":"notifications/initialized"}
//! {"jsonrpc":"2.0","id":2,"method":"tools/list"}
//! {"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"greet","arguments":{"name":"World"}}}
//! ```

use stand_in::prelude::*;

#[mcp_tool(name = "greet", description = "Greet someone by name")]
async fn greet(name: String) -> Result<String> {
    Ok(format!("Hello, {name}!"))
}

#[mcp_server]
struct HelloServer;

#[tokio::main]
async fn main() {
    HelloServer::serve(StdioTransport).await.unwrap();
}
