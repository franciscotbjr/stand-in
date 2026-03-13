//! Reference MCP server for testing and demonstrating stand-in.
//!
//! Run: `cargo run -p stand-in-reference`
//! Then send JSON-RPC over stdin:
//!
//! ```json
//! {"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"0.1.0"}}}
//! {"jsonrpc":"2.0","method":"notifications/initialized"}
//! {"jsonrpc":"2.0","id":2,"method":"tools/list"}
//! {"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"greet","arguments":{"name":"World"}}}
//! {"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"add","arguments":{"a":2,"b":3}}}
//! {"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"echo","arguments":{"message":"hello"}}}
//! ```

use stand_in::prelude::*;

#[mcp_tool(name = "greet", description = "Greet someone by name")]
async fn greet(name: String) -> Result<String> {
    Ok(format!("Hello, {name}!"))
}

#[mcp_tool(name = "add", description = "Add two integers")]
async fn add(a: i64, b: i64) -> Result<String> {
    Ok(format!("{}", a + b))
}

#[mcp_tool(name = "echo", description = "Echo back a message")]
async fn echo(message: String) -> Result<String> {
    Ok(message)
}

#[mcp_server]
struct ReferenceServer;

#[tokio::main]
async fn main() {
    ReferenceServer::serve(StdioTransport::default())
        .await
        .unwrap();
}
