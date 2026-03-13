//! Integration tests for the HTTP MCP server.
//!
//! Spawns an HTTP server in a background task and tests the full
//! lifecycle using `reqwest` as the HTTP client.

#![cfg(feature = "http")]

use std::net::SocketAddr;
use std::time::Duration;

use reqwest::Client;
use serde_json::{Value, json};

use stand_in::prelude::*;

// ---------------------------------------------------------------------------
// Tool definitions (duplicated from reference server for test isolation)
// ---------------------------------------------------------------------------

#[mcp_tool(name = "greet", description = "Greet someone by name")]
async fn greet(name: String) -> Result<String> {
    Ok(format!("Hello, {name}!"))
}

#[mcp_tool(name = "add", description = "Add two integers")]
async fn add(a: i64, b: i64) -> Result<String> {
    Ok(format!("{}", a + b))
}

#[mcp_server]
struct TestHttpServer;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Find a free port and return the SocketAddr.
fn free_addr() -> SocketAddr {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap()
}

/// Spawn the HTTP server on a random port and return the base URL.
async fn spawn_server() -> String {
    let addr = free_addr();
    let url = format!("http://{addr}");

    tokio::spawn(async move {
        TestHttpServer::serve(HttpTransport::new(addr))
            .await
            .unwrap();
    });

    // Give the server a moment to bind
    tokio::time::sleep(Duration::from_millis(100)).await;

    url
}

fn client() -> Client {
    Client::new()
}

fn initialize_request() -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "1.0.0" }
        }
    })
}

/// POST an initialize request and return the session ID.
async fn initialize(client: &Client, base_url: &str) -> String {
    let resp = client
        .post(format!("{base_url}/mcp"))
        .json(&initialize_request())
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let session_id = resp
        .headers()
        .get("mcp-session-id")
        .expect("Missing Mcp-Session-Id header")
        .to_str()
        .unwrap()
        .to_string();

    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["result"]["protocolVersion"], "2025-03-26");
    assert!(body["result"]["serverInfo"]["name"].is_string());

    session_id
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_initialize_creates_session() {
    let base_url = spawn_server().await;
    let c = client();
    let session_id = initialize(&c, &base_url).await;
    assert!(!session_id.is_empty());
}

#[tokio::test]
async fn test_tools_list_with_session() {
    let base_url = spawn_server().await;
    let c = client();
    let session_id = initialize(&c, &base_url).await;

    let resp = c
        .post(format!("{base_url}/mcp"))
        .header("mcp-session-id", &session_id)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let tools = body["result"]["tools"].as_array().unwrap();
    assert!(tools.len() >= 2);
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"greet"));
    assert!(names.contains(&"add"));
}

#[tokio::test]
async fn test_tools_call_with_session() {
    let base_url = spawn_server().await;
    let c = client();
    let session_id = initialize(&c, &base_url).await;

    let resp = c
        .post(format!("{base_url}/mcp"))
        .header("mcp-session-id", &session_id)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": { "name": "greet", "arguments": { "name": "World" } }
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["result"]["content"][0]["text"], "Hello, World!");
}

#[tokio::test]
async fn test_post_without_session_returns_400() {
    let base_url = spawn_server().await;
    let c = client();

    // Non-initialize request without session header
    let resp = c
        .post(format!("{base_url}/mcp"))
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], -32600);
}

#[tokio::test]
async fn test_post_with_invalid_session_returns_404() {
    let base_url = spawn_server().await;
    let c = client();

    let resp = c
        .post(format!("{base_url}/mcp"))
        .header("mcp-session-id", "nonexistent-session-id")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 404);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], -32600);
}

#[tokio::test]
async fn test_delete_session_terminates() {
    let base_url = spawn_server().await;
    let c = client();
    let session_id = initialize(&c, &base_url).await;

    // DELETE the session
    let resp = c
        .delete(format!("{base_url}/mcp"))
        .header("mcp-session-id", &session_id)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    // Subsequent request with that session should fail
    let resp = c
        .post(format!("{base_url}/mcp"))
        .header("mcp-session-id", &session_id)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_delete_without_session_returns_400() {
    let base_url = spawn_server().await;
    let c = client();

    let resp = c.delete(format!("{base_url}/mcp")).send().await.unwrap();

    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn test_delete_invalid_session_returns_404() {
    let base_url = spawn_server().await;
    let c = client();

    let resp = c
        .delete(format!("{base_url}/mcp"))
        .header("mcp-session-id", "bogus")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_get_without_session_returns_400() {
    let base_url = spawn_server().await;
    let c = client();

    let resp = c.get(format!("{base_url}/mcp")).send().await.unwrap();

    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn test_get_with_invalid_session_returns_404() {
    let base_url = spawn_server().await;
    let c = client();

    let resp = c
        .get(format!("{base_url}/mcp"))
        .header("mcp-session-id", "bogus")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 404);
}
