//! A minimal MCP server over HTTP that exposes a single "greet" tool.
//!
//! Run: `cargo run --example http_server --features http`
//!
//! Then test with curl:
//!
//! ```bash
//! # Initialize (creates session)
//! curl -X POST http://127.0.0.1:3000/mcp \
//!   -H "Content-Type: application/json" \
//!   -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"0.1.0"}}}'
//!
//! # List tools (use Mcp-Session-Id from the initialize response header)
//! curl -X POST http://127.0.0.1:3000/mcp \
//!   -H "Content-Type: application/json" \
//!   -H "Mcp-Session-Id: <SESSION_ID>" \
//!   -d '{"jsonrpc":"2.0","id":2,"method":"tools/list"}'
//!
//! # Call a tool
//! curl -X POST http://127.0.0.1:3000/mcp \
//!   -H "Content-Type: application/json" \
//!   -H "Mcp-Session-Id: <SESSION_ID>" \
//!   -d '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"greet","arguments":{"name":"World"}}}'
//!
//! # Terminate session
//! curl -X DELETE http://127.0.0.1:3000/mcp \
//!   -H "Mcp-Session-Id: <SESSION_ID>"
//! ```

use stand_in::prelude::*;

#[mcp_tool(name = "greet", description = "Greet someone by name")]
async fn greet(name: String) -> Result<String> {
    Ok(format!("Hello, {name}!"))
}

#[mcp_server]
struct HttpExample;

#[tokio::main]
async fn main() {
    // Option A: use default address (127.0.0.1:3000)
    HttpExample::serve_http().await.unwrap();

    // Option B: override at runtime (uncomment to use)
    // HttpExample::serve(HttpTransport::new(([0, 0, 0, 0], 8080))).await.unwrap();
}
