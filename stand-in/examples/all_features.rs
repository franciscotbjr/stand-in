//! A single MCP server demonstrating ALL stand-in features:
//! tools, prompts, and resources over Streamable HTTP.
//!
//! Run: `RUST_LOG=stand_in=info cargo run --example all_features --features http`
//!
//! For detailed request tracing:
//! `RUST_LOG=stand_in=debug cargo run --example all_features --features http`
//!
//! Then test with curl (replace `<SID>` with the Mcp-Session-Id from initialize):
//!
//! ```bash
//! # Initialize (creates session — save the Mcp-Session-Id header value)
//! curl -s -D- -X POST http://127.0.0.1:3000/mcp \
//!   -H "Content-Type: application/json" \
//!   -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}'
//!
//! # --- Tools ---
//! # List tools
//! curl -s -X POST http://127.0.0.1:3000/mcp \
//!   -H "Content-Type: application/json" -H "Mcp-Session-Id: <SID>" \
//!   -d '{"jsonrpc":"2.0","id":2,"method":"tools/list"}'
//!
//! # Call greet
//! curl -s -X POST http://127.0.0.1:3000/mcp \
//!   -H "Content-Type: application/json" -H "Mcp-Session-Id: <SID>" \
//!   -d '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"greet","arguments":{"name":"World"}}}'
//!
//! # Call add
//! curl -s -X POST http://127.0.0.1:3000/mcp \
//!   -H "Content-Type: application/json" -H "Mcp-Session-Id: <SID>" \
//!   -d '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"add","arguments":{"a":2,"b":3}}}'
//!
//! # Call is_prime (boolean return)
//! curl -s -X POST http://127.0.0.1:3000/mcp \
//!   -H "Content-Type: application/json" -H "Mcp-Session-Id: <SID>" \
//!   -d '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"is_prime","arguments":{"n":7}}}'
//!
//! # Call divide — triggers isError: true (zero divisor)
//! curl -s -X POST http://127.0.0.1:3000/mcp \
//!   -H "Content-Type: application/json" -H "Mcp-Session-Id: <SID>" \
//!   -d '{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"divide","arguments":{"a":10,"b":0}}}'
//!
//! # --- Prompts ---
//! # List prompts
//! curl -s -X POST http://127.0.0.1:3000/mcp \
//!   -H "Content-Type: application/json" -H "Mcp-Session-Id: <SID>" \
//!   -d '{"jsonrpc":"2.0","id":7,"method":"prompts/list"}'
//!
//! # Get summarize prompt (with optional audience argument)
//! curl -s -X POST http://127.0.0.1:3000/mcp \
//!   -H "Content-Type: application/json" -H "Mcp-Session-Id: <SID>" \
//!   -d '{"jsonrpc":"2.0","id":8,"method":"prompts/get","params":{"name":"summarize","arguments":{"text":"Rust is a systems programming language...","audience":"beginner"}}}'
//!
//! # Get explain_code prompt (Prompt::assistant)
//! curl -s -X POST http://127.0.0.1:3000/mcp \
//!   -H "Content-Type: application/json" -H "Mcp-Session-Id: <SID>" \
//!   -d '{"jsonrpc":"2.0","id":9,"method":"prompts/get","params":{"name":"explain_code","arguments":{"code":"fn main() { println!(\"Hello\"); }"}}}'
//!
//! # --- Resources ---
//! # List concrete resources
//! curl -s -X POST http://127.0.0.1:3000/mcp \
//!   -H "Content-Type: application/json" -H "Mcp-Session-Id: <SID>" \
//!   -d '{"jsonrpc":"2.0","id":10,"method":"resources/list"}'
//!
//! # List template resources
//! curl -s -X POST http://127.0.0.1:3000/mcp \
//!   -H "Content-Type: application/json" -H "Mcp-Session-Id: <SID>" \
//!   -d '{"jsonrpc":"2.0","id":11,"method":"resources/templates/list"}'
//!
//! # Read a concrete text resource
//! curl -s -X POST http://127.0.0.1:3000/mcp \
//!   -H "Content-Type: application/json" -H "Mcp-Session-Id: <SID>" \
//!   -d '{"jsonrpc":"2.0","id":12,"method":"resources/read","params":{"uri":"info://version"}}'
//!
//! # Read a concrete blob resource (returns base64-encoded PNG)
//! curl -s -X POST http://127.0.0.1:3000/mcp \
//!   -H "Content-Type: application/json" -H "Mcp-Session-Id: <SID>" \
//!   -d '{"jsonrpc":"2.0","id":13,"method":"resources/read","params":{"uri":"data://logo"}}'
//!
//! # Read a template resource
//! curl -s -X POST http://127.0.0.1:3000/mcp \
//!   -H "Content-Type: application/json" -H "Mcp-Session-Id: <SID>" \
//!   -d '{"jsonrpc":"2.0","id":14,"method":"resources/read","params":{"uri":"docs://rust/guide"}}'
//!
//! # Subscribe to resource updates (requires active SSE stream)
//! curl -s -X POST http://127.0.0.1:3000/mcp \
//!   -H "Content-Type: application/json" -H "Mcp-Session-Id: <SID>" \
//!   -d '{"jsonrpc":"2.0","id":15,"method":"resources/subscribe","params":{"uri":"info://version"}}'
//!
//! # --- Cleanup ---
//! # Terminate session
//! curl -s -X DELETE http://127.0.0.1:3000/mcp \
//!   -H "Mcp-Session-Id: <SID>"
//! ```

use stand_in::prelude::*;
use tracing_subscriber::EnvFilter;

// ---------------------------------------------------------------------------
// Tools
// ---------------------------------------------------------------------------

#[mcp_tool(name = "greet", description = "Greet someone by name")]
async fn greet(name: String) -> Result<String> {
    Ok(format!("Hello, {name}!"))
}

#[mcp_tool(name = "add", description = "Add two integers")]
async fn add(a: i64, b: i64) -> Result<String> {
    Ok(format!("{}", a + b))
}

#[mcp_tool(name = "is_prime", description = "Check if a number is prime")]
async fn is_prime(n: i64) -> Result<String> {
    if n < 2 {
        return Ok("false".to_string());
    }
    for i in 2..=((n as f64).sqrt() as i64) {
        if n % i == 0 {
            return Ok("false".to_string());
        }
    }
    Ok("true".to_string())
}

#[mcp_tool(
    name = "divide",
    description = "Divide a by b — returns error on division by zero"
)]
async fn divide(a: i64, b: i64) -> Result<String> {
    if b == 0 {
        return Err(Error::ToolError("Division by zero".to_string()));
    }
    Ok(format!("{}", a / b))
}

// ---------------------------------------------------------------------------
// Prompts
// ---------------------------------------------------------------------------

#[mcp_prompt(
    name = "summarize",
    description = "Summarize text for a given audience"
)]
async fn summarize(text: String, audience: Option<String>) -> Result<Prompt> {
    let level = audience.as_deref().unwrap_or("general");
    Ok(Prompt::user(format!(
        "Summarize the following text for a {level} audience:\n\n{text}"
    )))
}

#[mcp_prompt(
    name = "explain_code",
    description = "Explain a piece of code with an example response"
)]
async fn explain_code(code: String) -> Result<Prompt> {
    Ok(Prompt::assistant(format!(
        "Here is an explanation of the code you provided:\n\n\
         ```\n{code}\n```\n\n\
         This code does the following: ..."
    )))
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Concrete text resource: server version info.
#[mcp_resource(
    uri = "info://version",
    name = "Server Version",
    description = "Build version of the all-features example server",
    mime_type = "application/json"
)]
async fn server_version() -> Result<String> {
    Ok(serde_json::json!({
        "name": "stand-in all-features",
        "version": env!("CARGO_PKG_VERSION"),
        "protocol": "2025-03-26"
    })
    .to_string())
}

/// Concrete blob resource: a minimal PNG file (Vec<u8> → base64 BlobResourceContents).
#[mcp_resource(
    uri = "data://logo",
    name = "Server Logo",
    description = "A placeholder logo image",
    mime_type = "image/png"
)]
async fn server_logo() -> Result<Vec<u8>> {
    // Minimal valid 1x1 pixel PNG (transparent)
    Ok(vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77,
        0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, // IDAT chunk
        0x54, 0x08, 0xD7, 0x63, 0xF8, 0xFF, 0xFF, 0x07, 0x00, 0x01, 0x01, 0x00, 0xE7, 0x87, 0x8E,
        0x76, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, // IEND chunk
        0xAE, 0x42, 0x60, 0x82,
    ])
}

/// Template resource: documentation by topic.
#[mcp_resource(
    uri = "docs://{topic}/guide",
    name = "Documentation",
    description = "A quick-start guide for a given topic",
    mime_type = "text/markdown"
)]
async fn docs_guide(topic: String) -> Result<String> {
    Ok(format!(
        "# {topic} Guide\n\n\
         Welcome to the {topic} guide.\n\n\
         ## Getting Started\n\n\
         To get started with {topic}, follow these steps...\n\n\
         ## API Reference\n\n\
         See the `{topic}` module for full API documentation.\n"
    ))
}

// ---------------------------------------------------------------------------
// Server
// ---------------------------------------------------------------------------

#[mcp_server]
struct AllFeaturesServer;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("stand_in=info".parse().unwrap()),
        )
        .init();

    // Default: 127.0.0.1:3000. Override with #[mcp_server(host = "...", port = N)]
    AllFeaturesServer::serve_http().await.unwrap();
}
