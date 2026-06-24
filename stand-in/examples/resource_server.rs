//! An MCP server demonstrating `#[mcp_resource]` — concrete and template resources.
//!
//! Run: `cargo run --example resource_server`
//! Then send JSON-RPC over stdin:
//!
//! ```json
//! {"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"0.1.0"}}}
//! {"jsonrpc":"2.0","method":"notifications/initialized"}
//! {"jsonrpc":"2.0","id":2,"method":"resources/list"}
//! {"jsonrpc":"2.0","id":3,"method":"resources/templates/list"}
//! {"jsonrpc":"2.0","id":4,"method":"resources/read","params":{"uri":"info://version"}}
//! {"jsonrpc":"2.0","id":5,"method":"resources/read","params":{"uri":"docs://rust/readme"}}
//! {"jsonrpc":"2.0","id":6,"method":"resources/read","params":{"uri":"config://settings"}}
//! {"jsonrpc":"2.0","id":7,"method":"resources/subscribe","params":{"uri":"info://version"}}
//! {"jsonrpc":"2.0","id":8,"method":"resources/unsubscribe","params":{"uri":"info://version"}}
//! ```

use stand_in::prelude::*;

// ----- Concrete resource: returns server version info -----

#[mcp_resource(
    uri = "info://version",
    name = "Server Version",
    description = "Stand-in resource server version info",
    mime_type = "application/json"
)]
async fn server_version() -> Result<String> {
    Ok(serde_json::json!({
        "name": env!("CARGO_PKG_NAME"),
        "version": env!("CARGO_PKG_VERSION")
    })
    .to_string())
}

// ----- Concrete resource: returns server settings -----

#[mcp_resource(
    uri = "config://settings",
    name = "Server Settings",
    mime_type = "application/json"
)]
async fn server_settings() -> Result<String> {
    Ok(serde_json::json!({
        "max_connections": 100,
        "timeout": 30
    })
    .to_string())
}

// ----- Template resource: documentation by topic -----

#[mcp_resource(
    uri = "docs://{topic}/readme",
    name = "Documentation",
    description = "Documentation for a given topic",
    mime_type = "text/markdown"
)]
async fn docs_readme(topic: String) -> Result<String> {
    Ok(format!("# {topic}\n\nDocumentation for **{topic}**."))
}

#[mcp_server]
struct ResourceServer;

#[tokio::main]
async fn main() {
    ResourceServer::serve(StdioTransport).await.unwrap();
}
