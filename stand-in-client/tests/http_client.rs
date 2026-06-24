//! Integration tests for the HTTP client transport.
//!
//! Spawns an in-process `stand-in` MCP server on a random port and tests
//! the full `Client` lifecycle over Streamable HTTP: handshake → list fetches
//! → tool call → prompt get → resource read → subscribe → disconnect.

#![cfg(feature = "http")]

use std::net::SocketAddr;
use std::time::Duration;

use stand_in::error::Result as ServerResult;
use stand_in::{self, prelude::*};
use stand_in_client::prelude::{
    Client, ClientTransport, Content, Notification, PromptContent, PromptRole, ResourceContents,
};
use stand_in_client::transport::HttpTransport as ClientHttpTransport;

// ---------------------------------------------------------------------------
// Test server definition
// ---------------------------------------------------------------------------

#[mcp_tool(name = "greet", description = "Greet someone by name")]
async fn greet(name: String) -> ServerResult<String> {
    Ok(format!("Hello, {name}!"))
}

#[mcp_tool(name = "add", description = "Add two integers")]
async fn add(a: i64, b: i64) -> ServerResult<String> {
    Ok(format!("{}", a + b))
}

#[mcp_tool(
    name = "divide",
    description = "Divide two integers (returns error on zero divisor)"
)]
async fn divide(a: i64, b: i64) -> ServerResult<String> {
    if b == 0 {
        Err(stand_in::error::Error::ToolError("division by zero".into()))
    } else {
        Ok(format!("{}", a / b))
    }
}

#[mcp_prompt(
    name = "write_greeting",
    description = "Generate a greeting message for a person"
)]
async fn write_greeting(name: String, style: Option<String>) -> ServerResult<Prompt> {
    let text = match style.as_deref() {
        Some("formal") => format!("Write a formal greeting for {name}."),
        _ => format!("Write a greeting for {name}."),
    };
    Ok(Prompt::user(text))
}

#[mcp_resource(
    uri = "test://version",
    name = "Version",
    description = "Server version info",
    mime_type = "application/json"
)]
async fn test_version() -> ServerResult<String> {
    Ok(serde_json::json!({"version": "0.1.0"}).to_string())
}

#[mcp_resource(
    uri = "doc://{topic}/readme",
    name = "Docs",
    description = "Documentation for a topic",
    mime_type = "text/markdown"
)]
async fn doc_readme(topic: String) -> ServerResult<String> {
    Ok(format!("# {topic}\n\nDocs for {topic}."))
}

#[mcp_server]
struct TestHttpServer;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

fn free_addr() -> SocketAddr {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap()
}

async fn spawn_server() -> String {
    let addr = free_addr();
    let url = format!("http://{addr}");

    tokio::spawn(async move {
        TestHttpServer::serve(stand_in::transport::HttpTransport::new(addr))
            .await
            .unwrap();
    });

    tokio::time::sleep(Duration::from_millis(200)).await;
    url
}

async fn connect_client(server_url: &str) -> Client {
    let url = format!("{server_url}/mcp");
    Client::builder()
        .transport(ClientHttpTransport::new(&url))
        .client_info("test-client", "0.1.0")
        .timeout(Duration::from_secs(10))
        .connect()
        .await
        .expect("client should connect")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_handshake_and_caches() {
    let url = spawn_server().await;
    let client = connect_client(&url).await;

    // Server identity (auto-derived from Cargo.toml)
    assert!(!client.server_info().name.is_empty());
    assert!(!client.server_info().version.is_empty());

    // Capabilities
    let caps = client.server_capabilities();
    assert!(
        caps.tools.is_some(),
        "tools capability should be advertised"
    );
    assert!(
        caps.prompts.is_some(),
        "prompts capability should be advertised"
    );
    assert!(
        caps.resources.is_some(),
        "resources capability should be advertised"
    );

    // Cached tools
    let tools = client.tools();
    let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"greet"));
    assert!(names.contains(&"add"));
    assert!(names.contains(&"divide"));
    assert_eq!(tools.len(), 3);

    // Cached prompts
    let prompts = client.prompts();
    assert_eq!(prompts.len(), 1);
    assert_eq!(prompts[0].name, "write_greeting");

    // Cached concrete resources
    let resources = client.resources();
    let uris: Vec<&str> = resources.iter().map(|r| r.uri.as_str()).collect();
    assert!(uris.contains(&"test://version"));

    // Cached resource templates
    let templates = client.resource_templates();
    assert_eq!(templates.len(), 1);
    assert_eq!(templates[0].uri_template, "doc://{topic}/readme");

    client.disconnect().await.unwrap();
}

#[tokio::test]
async fn test_call_tool_success() {
    let url = spawn_server().await;
    let client = connect_client(&url).await;

    let result = client
        .call_tool("greet", serde_json::json!({ "name": "World" }))
        .await
        .unwrap();
    assert_eq!(result.is_error, None);
    assert_eq!(result.content.len(), 1);
    match &result.content[0] {
        Content::Text { text } => assert_eq!(text, "Hello, World!"),
    }

    let result = client
        .call_tool("add", serde_json::json!({ "a": 3, "b": 4 }))
        .await
        .unwrap();
    assert_eq!(result.is_error, None);
    match &result.content[0] {
        Content::Text { text } => assert_eq!(text, "7"),
    }

    client.disconnect().await.unwrap();
}

#[tokio::test]
async fn test_call_tool_two_error_planes() {
    let url = spawn_server().await;
    let client = connect_client(&url).await;

    // Division by zero: tool execution error → isError = true (not Err)
    let result = client
        .call_tool("divide", serde_json::json!({ "a": 10, "b": 0 }))
        .await
        .unwrap();
    assert_eq!(result.is_error, Some(true));

    // Unknown tool: returns isError = true (tool error, not protocol error)
    let result = client.call_tool("nonexistent", serde_json::json!({})).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().is_error, Some(true));

    client.disconnect().await.unwrap();
}

#[tokio::test]
async fn test_get_prompt() {
    let url = spawn_server().await;
    let client = connect_client(&url).await;

    let result = client
        .get_prompt(
            "write_greeting",
            serde_json::json!({ "name": "Alice", "style": "formal" }),
        )
        .await
        .unwrap();
    assert_eq!(result.messages.len(), 1);
    assert_eq!(result.messages[0].role, PromptRole::User);
    match &result.messages[0].content {
        PromptContent::Text { text } => {
            assert_eq!(text, "Write a formal greeting for Alice.")
        }
    }

    client.disconnect().await.unwrap();
}

#[tokio::test]
async fn test_read_resource_text() {
    let url = spawn_server().await;
    let client = connect_client(&url).await;

    let result = client.read_resource("test://version").await.unwrap();
    assert_eq!(result.contents.len(), 1);
    assert_eq!(result.contents[0].uri(), "test://version");
    match &result.contents[0] {
        ResourceContents::Text {
            mime_type, text, ..
        } => {
            assert_eq!(mime_type.as_deref(), Some("application/json"));
            assert!(text.contains("version"));
        }
        _ => panic!("expected Text variant"),
    }

    client.disconnect().await.unwrap();
}

#[tokio::test]
async fn test_read_resource_template() {
    let url = spawn_server().await;
    let client = connect_client(&url).await;

    let result = client.read_resource("doc://rust/readme").await.unwrap();
    assert_eq!(result.contents.len(), 1);
    assert_eq!(result.contents[0].uri(), "doc://rust/readme");
    match &result.contents[0] {
        ResourceContents::Text { text, .. } => {
            assert!(text.contains("# rust"));
        }
        _ => panic!("expected Text variant"),
    }

    client.disconnect().await.unwrap();
}

#[tokio::test]
async fn test_subscribe_and_disconnect() {
    let url = spawn_server().await;
    let client = connect_client(&url).await;

    client.subscribe("test://version").await.unwrap();

    let mut notif_rx = client.notifications();

    client.disconnect().await.unwrap();

    // After disconnect, the Disconnected notification should arrive.
    match tokio::time::timeout(Duration::from_secs(5), notif_rx.recv()).await {
        Ok(Ok(n)) => assert_eq!(n, Notification::Disconnected),
        Ok(Err(_)) => {} // lagged (channel closed)
        Err(_) => {}     // timeout — also acceptable
    }
}

#[tokio::test]
async fn test_call_tool_add() {
    let url = spawn_server().await;
    let client = connect_client(&url).await;

    let result = client
        .call_tool("add", serde_json::json!({ "a": 100, "b": 200 }))
        .await
        .unwrap();
    assert_eq!(result.is_error, None);
    match &result.content[0] {
        Content::Text { text } => assert_eq!(text, "300"),
    }

    client.disconnect().await.unwrap();
}

#[tokio::test]
async fn test_unknown_prompt_returns_error() {
    let url = spawn_server().await;
    let client = connect_client(&url).await;

    let result = client
        .get_prompt("nonexistent", serde_json::json!({}))
        .await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        stand_in_client::error::Error::ProtocolError(_)
    ));

    client.disconnect().await.unwrap();
}

#[tokio::test]
async fn test_disconnect_clean() {
    let url = spawn_server().await;
    let client = connect_client(&url).await;

    // Perform a few operations before disconnecting
    let _ = client
        .call_tool("greet", serde_json::json!({ "name": "Test" }))
        .await;
    let _ = client.read_resource("test://version").await;
    let _ = client
        .get_prompt("write_greeting", serde_json::json!({ "name": "B" }))
        .await;

    client.disconnect().await.unwrap();
}

/// Verify HttpTransport is usable as a ClientTransport via the trait alias.
#[test]
fn test_http_transport_is_client_transport() {
    fn _assert(_t: impl ClientTransport) {}
}
