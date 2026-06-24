//! Integration tests for the client-side stdio transport.
//!
//! Spawns `stand-in-reference` as a child process and exercises the transport
//! layer (send/receive/close), proving that:
//! - The framing matches the server (line-delimited JSON)
//! - `send` and `receive` work concurrently over a shared `Arc`
//! - `close` cleanly terminates the child process

use std::sync::Arc;
use std::sync::Once;
use std::time::Duration;

use stand_in_client::prelude::*;
use tokio::time::timeout;

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
    path.pop(); // go up from stand-in-client/ to workspace root
    path.push("target");
    path.push("debug");
    if cfg!(windows) {
        path.push("stand-in-reference.exe");
    } else {
        path.push("stand-in-reference");
    }
    path
}

#[tokio::test]
async fn test_connect_send_receive_close() {
    ensure_binary_built();
    let binary = reference_binary_path();

    let mut transport = StdioTransport::command(&binary, &[] as &[&str]);
    transport.connect().await.expect("connect should succeed");

    // Send an initialize request
    transport
        .send(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}"#,
        )
        .await
        .expect("send should succeed");

    // Receive the response (with timeout guard to prevent CI hangs)
    let line = timeout(Duration::from_secs(5), transport.receive())
        .await
        .expect("receive timed out")
        .expect("receive should succeed")
        .expect("should get a response line");

    let json: serde_json::Value =
        serde_json::from_str(&line).expect("response should be valid JSON");
    assert_eq!(json["id"], 1);
    assert_eq!(json["result"]["protocolVersion"], "2025-03-26");
    assert!(json["result"]["serverInfo"]["name"].is_string());

    // Close the transport
    transport.close().await.expect("close should succeed");
}

#[tokio::test]
async fn test_concurrent_send_receive() {
    ensure_binary_built();
    let binary = reference_binary_path();

    let mut transport = StdioTransport::command(&binary, &[] as &[&str]);
    transport.connect().await.expect("connect should succeed");

    let shared = Arc::new(transport);

    // Send an initialize request from one task
    let send_handle = {
        let t = shared.clone();
        tokio::spawn(async move {
            t.send(
                r#"{"jsonrpc":"2.0","id":2,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}"#,
            )
            .await
            .expect("send should succeed")
        })
    };

    // Receive from another task concurrently (with timeout guard)
    let recv_handle = {
        let t = shared.clone();
        tokio::spawn(async move {
            timeout(Duration::from_secs(5), t.receive())
                .await
                .expect("receive timed out")
                .expect("receive should succeed")
                .expect("should get a response line")
        })
    };

    // Both should complete
    send_handle.await.expect("send task should not panic");
    let line = recv_handle.await.expect("recv task should not panic");

    let json: serde_json::Value =
        serde_json::from_str(&line).expect("response should be valid JSON");
    assert_eq!(json["id"], 2);
    assert_eq!(json["result"]["protocolVersion"], "2025-03-26");

    shared.close().await.expect("close should succeed");
}

#[tokio::test]
async fn test_send_receive_before_connect_errors() {
    let transport = StdioTransport::command("nonexistent_binary_xyz", &[] as &[&str]);

    // send before connect
    let result = transport.send(r#"{"jsonrpc":"2.0","id":1}"#).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not connected"));

    // receive before connect (with timeout guard)
    let result = timeout(Duration::from_secs(3), transport.receive())
        .await
        .expect("receive timed out");
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());

    // close before connect (idempotent, no-op)
    let result = transport.close().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_close_is_idempotent() {
    ensure_binary_built();
    let binary = reference_binary_path();

    let mut transport = StdioTransport::command(&binary, &[] as &[&str]);
    transport.connect().await.expect("connect should succeed");

    // Close once
    transport.close().await.expect("first close should succeed");
    // Close again — should not panic
    transport
        .close()
        .await
        .expect("second close should succeed");

    // send after close
    let result = transport.send(r#"{"jsonrpc":"2.0","id":1}"#).await;
    assert!(result.is_err());

    // receive after close (with timeout guard)
    let result = timeout(Duration::from_secs(3), transport.receive())
        .await
        .expect("receive timed out");
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_connect_spawn_error() {
    let mut transport = StdioTransport::command("nonexistent_binary_xyz_abc_123", &[] as &[&str]);
    let result = transport.connect().await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("failed to spawn"));
    assert!(msg.contains("nonexistent_binary_xyz_abc_123"));
}

// ── M4: Client-level full lifecycle integration tests ───────────────────────

#[tokio::test]
async fn test_client_full_lifecycle() {
    ensure_binary_built();
    let binary = reference_binary_path();

    let client = timeout(
        Duration::from_secs(30),
        Client::builder()
            .transport(StdioTransport::command(&binary, &[] as &[&str]))
            .client_info("test-client", "0.1.0")
            .timeout(Duration::from_secs(10))
            .connect(),
    )
    .await
    .expect("connect timed out")
    .expect("connect should succeed");

    // Server identity
    assert_eq!(client.server_info().name, "stand-in-reference");
    assert!(
        client
            .server_info()
            .version
            .chars()
            .next()
            .unwrap()
            .is_ascii_digit()
    );
    let caps = client.server_capabilities();
    assert!(caps.tools.is_some());
    assert!(caps.resources.is_some());
    // reference has no prompts macro → prompts capability is advertised
    // by the macro but the reference has no prompts registered
    assert!(caps.prompts.is_some());

    // Cached lists
    assert_eq!(client.tools().len(), 3);
    let tool_names: Vec<&str> = client.tools().iter().map(|t| t.name.as_str()).collect();
    assert!(tool_names.contains(&"greet"));
    assert!(tool_names.contains(&"add"));
    assert!(tool_names.contains(&"echo"));

    assert_eq!(client.resources().len(), 2);
    let resource_uris: Vec<&str> = client.resources().iter().map(|r| r.uri.as_str()).collect();
    assert!(resource_uris.contains(&"info://version"));
    assert!(resource_uris.contains(&"config://settings"));

    assert_eq!(client.resource_templates().len(), 1);
    assert_eq!(
        client.resource_templates()[0].uri_template,
        "docs://{topic}/readme"
    );

    assert_eq!(client.prompts().len(), 1);
    assert_eq!(client.prompts()[0].name, "greeting");

    // call_tool: greet
    let result = timeout(
        Duration::from_secs(10),
        client.call_tool("greet", serde_json::json!({ "name": "World" })),
    )
    .await
    .expect("call_tool(greet) timed out")
    .expect("call_tool(greet) should succeed");
    assert_eq!(result.is_error, None);
    let content = &result.content;
    assert_eq!(content.len(), 1);
    #[allow(unreachable_patterns)]
    let text = match &content[0] {
        Content::Text { text } => text.as_str(),
        _ => panic!("expected text content for greet"),
    };
    assert_eq!(text, "Hello, World!");

    // call_tool: add
    let result = client
        .call_tool("add", serde_json::json!({ "a": 2, "b": 3 }))
        .await
        .expect("call_tool(add) should succeed");
    #[allow(unreachable_patterns)]
    let text = match &result.content[0] {
        Content::Text { text } => text.as_str(),
        _ => panic!("expected text content for add"),
    };
    assert_eq!(text, "5");

    // Two error planes: unknown tool → Ok with isError: true
    let result = client
        .call_tool("nope", serde_json::json!({}))
        .await
        .expect("call_tool(nope) should return Ok (isError is data)");
    assert_eq!(result.is_error, Some(true));

    // read_resource: concrete
    let rr = client
        .read_resource("info://version")
        .await
        .expect("read_resource(info://version) should succeed");
    let contents = &rr.contents;
    assert_eq!(contents.len(), 1);
    if let ResourceContents::Text {
        ref text, ref uri, ..
    } = contents[0]
    {
        assert_eq!(uri, "info://version");
        let v: serde_json::Value =
            serde_json::from_str(text).expect("version resource should be valid JSON");
        assert_eq!(v["name"], "stand-in-reference");
    } else {
        panic!("expected text resource contents");
    }

    // read_resource: template
    let rr = client
        .read_resource("docs://rust/readme")
        .await
        .expect("read_resource(docs://rust/readme) should succeed");
    if let ResourceContents::Text { ref text, .. } = rr.contents[0] {
        assert!(text.contains("# rust"));
    } else {
        panic!("expected text resource contents");
    }

    // Protocol error plane: unknown URI → Err
    let result = client.read_resource("bad://nope").await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("-32601"));

    // subscribe (no live push on stdio, but the request should succeed)
    client
        .subscribe("info://version")
        .await
        .expect("subscribe should succeed");

    // Disconnect
    timeout(Duration::from_secs(10), client.disconnect())
        .await
        .expect("disconnect timed out")
        .expect("disconnect should succeed");
}

#[tokio::test]
async fn test_client_prompts_for_reference() {
    ensure_binary_built();
    let binary = reference_binary_path();

    let client = Client::builder()
        .transport(StdioTransport::command(&binary, &[] as &[&str]))
        .client_info("test", "0")
        .timeout(Duration::from_secs(10))
        .connect()
        .await
        .expect("connect should succeed");

    assert!(!client.prompts().is_empty());
    assert_eq!(client.prompts()[0].name, "greeting");

    // get_prompt for nonexistent prompt → protocol error
    let result = client
        .get_prompt("nonexistent", serde_json::json!({}))
        .await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("-32601"));

    client
        .disconnect()
        .await
        .expect("disconnect should succeed");
}

#[tokio::test]
async fn test_client_unsubscribe_ok() {
    ensure_binary_built();
    let binary = reference_binary_path();

    let client = Client::builder()
        .transport(StdioTransport::command(&binary, &[] as &[&str]))
        .client_info("test", "0")
        .timeout(Duration::from_secs(10))
        .connect()
        .await
        .expect("connect should succeed");

    client
        .unsubscribe("info://version")
        .await
        .expect("unsubscribe should succeed");

    client
        .disconnect()
        .await
        .expect("disconnect should succeed");
}

#[tokio::test]
async fn test_client_refresh() {
    ensure_binary_built();
    let binary = reference_binary_path();

    let mut client = Client::builder()
        .transport(StdioTransport::command(&binary, &[] as &[&str]))
        .client_info("test-refresh", "0.1.0")
        .timeout(Duration::from_secs(10))
        .connect()
        .await
        .expect("connect should succeed");

    // Initial cache state
    assert!(!client.tools().is_empty());
    assert!(!client.resources().is_empty());
    assert!(!client.resource_templates().is_empty());

    // Refresh and verify caches remain populated
    timeout(Duration::from_secs(10), client.refresh())
        .await
        .expect("refresh timed out")
        .expect("refresh should succeed");

    assert_eq!(client.tools().len(), 3);
    assert_eq!(client.resources().len(), 2);
    assert_eq!(client.resource_templates().len(), 1);
    assert_eq!(client.prompts().len(), 1);
    assert_eq!(client.prompts()[0].name, "greeting");

    client
        .disconnect()
        .await
        .expect("disconnect should succeed");
}
