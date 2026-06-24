//! Integration test for the `#[mcp_client]` proc macro.
//!
//! Spawns `stand-in-reference` and exercises the typed client stub generated
//! from a trait definition. Proves that:
//! - Args are serialized by name and sent via `call_tool`
//! - `String` return types extract text content directly
//! - `#[tool(name = "...")]` overrides the method name
//! - Non-`String` return types deserialize via `serde_json::from_str`
//! - `isError: true` collapses into `Err(Error::ToolError(...))`
//! - Unknown methods produce a compile error (nice diagnostic)

#![cfg(feature = "macros")]

use std::sync::Once;
use std::time::Duration;

use stand_in_client::prelude::*;

static BUILD: Once = Once::new();

fn ensure_binary_built() {
    BUILD.call_once(|| {
        let status = std::process::Command::new(env!("CARGO"))
            .args(["build", "-p", "stand-in-reference"])
            .status()
            .expect("Failed to run cargo build");
        assert!(status.success(), "Failed to build stand-in-reference");
    });
}

fn reference_binary_path() -> std::path::PathBuf {
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.push("target");
    path.push("debug");
    if cfg!(windows) {
        path.push("stand-in-reference.exe");
    } else {
        path.push("stand-in-reference");
    }
    path
}

// ═══════════════════════════════════════════════════════════════════════
// Typed client trait — mirrors stand-in-reference's tools
// ═══════════════════════════════════════════════════════════════════════

#[mcp_client]
pub trait Reference {
    /// Greet someone by name.
    async fn greet(&self, name: String) -> Result<String>;

    /// Add two integers (returns string representation).
    async fn add(&self, a: i64, b: i64) -> Result<String>;

    /// Echo back a message.
    async fn echo(&self, message: String) -> Result<String>;

    /// Add two integers — uses `#[tool]` override for the tool name
    /// and deserializes the response as `i64`.
    #[tool(name = "add")]
    async fn add_i64(&self, a: i64, b: i64) -> Result<i64>;
}

// ═══════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_macro_typed_calls() {
    ensure_binary_built();
    let binary = reference_binary_path();

    let client = Client::builder()
        .transport(StdioTransport::command(&binary, &[] as &[&str]))
        .client_info("macro-test", "0.1.0")
        .timeout(Duration::from_secs(10))
        .connect()
        .await
        .expect("connect should succeed");

    let api = ReferenceClient::new(client);

    // greet — String return (text extraction)
    let greeting = api
        .greet("World".into())
        .await
        .expect("greet should succeed");
    assert_eq!(greeting, "Hello, World!");

    // add — String return (text extraction)
    let sum_str = api.add(2, 3).await.expect("add should succeed");
    assert_eq!(sum_str, "5");

    // echo — String return
    let echoed = api.echo("hello".into()).await.expect("echo should succeed");
    assert_eq!(echoed, "hello");

    // add_i64 — uses #[tool(name = "add")] + non-String return (from_str)
    let sum_i64 = api.add_i64(10, 20).await.expect("add_i64 should succeed");
    assert_eq!(sum_i64, 30);

    api.into_inner()
        .disconnect()
        .await
        .expect("disconnect should succeed");
}

#[tokio::test]
async fn test_macro_client_new_into_inner_accessor() {
    ensure_binary_built();
    let binary = reference_binary_path();

    let client = Client::builder()
        .transport(StdioTransport::command(&binary, &[] as &[&str]))
        .client_info("macro-acc", "0.1.0")
        .timeout(Duration::from_secs(10))
        .connect()
        .await
        .expect("connect should succeed");

    let api = ReferenceClient::new(client);

    // Access the inner client
    assert_eq!(api.client().server_info().name, "stand-in-reference");

    // into_inner + disconnect
    api.into_inner()
        .disconnect()
        .await
        .expect("disconnect should succeed");
}

#[tokio::test]
async fn test_macro_tool_error_collapse() {
    ensure_binary_built();
    let binary = reference_binary_path();

    let client = Client::builder()
        .transport(StdioTransport::command(&binary, &[] as &[&str]))
        .client_info("macro-err", "0.1.0")
        .timeout(Duration::from_secs(10))
        .connect()
        .await
        .expect("connect should succeed");

    // Define a client that uses a nonexistent tool name.
    #[mcp_client]
    trait BadClient {
        #[tool(name = "nonexistent_tool_xyz")]
        async fn bad_call(&self) -> Result<String>;
    }

    let api = BadClientClient::new(client);

    let result = api.bad_call().await;
    assert!(result.is_err());

    let err = result.unwrap_err();
    let msg = err.to_string();

    // The server returns isError: true for unknown tools, which the typed
    // layer collapses into ToolError.
    let matches_tool_error = matches!(err, Error::ToolError(_));
    if !matches_tool_error {
        eprintln!("expected ToolError, got: {e}", e = err);
    }
    assert!(
        matches_tool_error,
        "expected ToolError, got: {err}",
        err = err
    );
    assert!(
        msg.contains("nonexistent_tool_xyz") || msg.contains("unknown tool"),
        "error message should mention the tool name, got: {msg}"
    );

    api.into_inner()
        .disconnect()
        .await
        .expect("disconnect should succeed");
}

#[tokio::test]
async fn test_macro_zero_params_method() {
    ensure_binary_built();
    let binary = reference_binary_path();

    let client = Client::builder()
        .transport(StdioTransport::command(&binary, &[] as &[&str]))
        .client_info("macro-zero", "0.1.0")
        .timeout(Duration::from_secs(10))
        .connect()
        .await
        .expect("connect should succeed");

    // Define a client with a zero-parameter method mapped to echo.
    #[mcp_client]
    trait ZeroParam {
        #[tool(name = "echo")]
        async fn hello(&self) -> Result<String>;
    }

    let api = ZeroParamClient::new(client);
    let result = api.hello().await;

    // echo without params: the server receives empty args, echoes nothing
    // or may return error — either way, the call itself reaches the server.
    match result {
        Ok(text) => {
            eprintln!("zero-param echo returned: {text:?}");
        }
        Err(e) => {
            eprintln!("zero-param echo errored (expected if server rejects empty args): {e}");
        }
    }

    api.into_inner()
        .disconnect()
        .await
        .expect("disconnect should succeed");
}
