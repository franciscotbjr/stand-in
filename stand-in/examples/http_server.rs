//! A minimal MCP server over HTTP that exposes a "greet" tool and a "greeting_prompt" prompt.
//!
//! Run: `RUST_LOG=stand_in=info cargo run --example http_server --features http`
//!
//! For detailed request tracing:
//! `RUST_LOG=stand_in=debug cargo run --example http_server --features http`
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
//! # List prompts
//! curl -X POST http://127.0.0.1:3000/mcp \
//!   -H "Content-Type: application/json" \
//!   -H "Mcp-Session-Id: <SESSION_ID>" \
//!   -d '{"jsonrpc":"2.0","id":4,"method":"prompts/list"}'
//!
//! # Get a prompt (name is required; style is optional)
//! curl -X POST http://127.0.0.1:3000/mcp \
//!   -H "Content-Type: application/json" \
//!   -H "Mcp-Session-Id: <SESSION_ID>" \
//!   -d '{"jsonrpc":"2.0","id":5,"method":"prompts/get","params":{"name":"greeting_prompt","arguments":{"name":"Alice","style":"formal"}}}'
//!
//! # Terminate session
//! curl -X DELETE http://127.0.0.1:3000/mcp \
//!   -H "Mcp-Session-Id: <SESSION_ID>"
//! ```

use stand_in::prelude::*;
use tracing_subscriber::EnvFilter;

#[mcp_tool(name = "greet", description = "Greet someone by name")]
async fn greet(name: String) -> Result<String> {
    Ok(format!("Hello, {name}!"))
}

#[mcp_prompt(
    name = "greeting_prompt",
    description = "Generate a greeting message for a person"
)]
async fn greeting_prompt(name: String, style: Option<String>) -> Result<Prompt> {
    let instruction = match style.as_deref() {
        Some("formal") => format!("Write a formal greeting addressed to {name}."),
        Some("casual") => format!("Write a casual, friendly greeting for {name}."),
        _ => format!("Write a greeting for {name}."),
    };
    Ok(Prompt::user(instruction))
}

#[mcp_server]
struct HttpExample;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("stand_in=info".parse().unwrap()),
        )
        .init();

    // Option A: use default address (127.0.0.1:3000)
    HttpExample::serve_http().await.unwrap();

    // Option B: override at runtime (uncomment to use)
    // HttpExample::serve(HttpTransport::new(([0, 0, 0, 0], 8080))).await.unwrap();
}
