//! Integration tests for the stdio MCP server.
//!
//! Spawns the `stand-in-reference` binary as a child process and communicates
//! via stdin/stdout using line-delimited JSON-RPC.

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::Once;

static BUILD: Once = Once::new();

fn ensure_binary_built() {
    BUILD.call_once(|| {
        let status = Command::new(env!("CARGO"))
            .args(["build", "-p", "stand-in-reference"])
            .status()
            .expect("Failed to run cargo build");
        assert!(status.success(), "Failed to build stand-in-reference");
    });
}

fn reference_binary_path() -> std::path::PathBuf {
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // go up from stand-in/ to workspace root
    path.push("target");
    path.push("debug");
    if cfg!(windows) {
        path.push("stand-in-reference.exe");
    } else {
        path.push("stand-in-reference");
    }
    path
}

fn spawn_server() -> std::process::Child {
    ensure_binary_built();
    let binary = reference_binary_path();
    Command::new(binary)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to spawn stand-in-reference")
}

fn send_and_receive(child: &mut std::process::Child, request: &str) -> String {
    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    stdin
        .write_all(format!("{request}\n").as_bytes())
        .expect("Failed to write to stdin");
    stdin.flush().expect("Failed to flush stdin");

    let stdout = child.stdout.as_mut().expect("Failed to open stdout");
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    reader.read_line(&mut line).expect("Failed to read line");
    line
}

#[test]
fn test_full_lifecycle() {
    let mut child = spawn_server();

    // 1. Initialize
    let resp = send_and_receive(
        &mut child,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}"#,
    );
    let json: serde_json::Value = serde_json::from_str(resp.trim()).unwrap();
    assert_eq!(json["result"]["protocolVersion"], "2025-03-26");
    assert!(json["result"]["serverInfo"]["name"].is_string());

    // 2. Initialized notification
    let resp = send_and_receive(
        &mut child,
        r#"{"jsonrpc":"2.0","id":2,"method":"notifications/initialized"}"#,
    );
    let json: serde_json::Value = serde_json::from_str(resp.trim()).unwrap();
    assert!(json["error"].is_null());

    // 3. Tools list
    let resp = send_and_receive(
        &mut child,
        r#"{"jsonrpc":"2.0","id":3,"method":"tools/list"}"#,
    );
    let json: serde_json::Value = serde_json::from_str(resp.trim()).unwrap();
    let tools = json["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 3);
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"greet"));
    assert!(names.contains(&"add"));
    assert!(names.contains(&"echo"));

    // 4. Tools call
    let resp = send_and_receive(
        &mut child,
        r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"greet","arguments":{"name":"World"}}}"#,
    );
    let json: serde_json::Value = serde_json::from_str(resp.trim()).unwrap();
    assert_eq!(json["result"]["content"][0]["text"], "Hello, World!");

    // 5. Close stdin → server should shut down gracefully
    drop(child.stdin.take());
    let status = child.wait().expect("Failed to wait for child");
    assert!(status.success());
}

#[test]
fn test_unknown_method_error() {
    let mut child = spawn_server();

    let resp = send_and_receive(&mut child, r#"{"jsonrpc":"2.0","id":1,"method":"foo/bar"}"#);
    let json: serde_json::Value = serde_json::from_str(resp.trim()).unwrap();
    assert_eq!(json["error"]["code"], -32601);

    drop(child.stdin.take());
    child.wait().unwrap();
}

#[test]
fn test_malformed_json_error() {
    let mut child = spawn_server();

    let resp = send_and_receive(&mut child, r#"not valid json"#);
    let json: serde_json::Value = serde_json::from_str(resp.trim()).unwrap();
    assert_eq!(json["error"]["code"], -32700);

    drop(child.stdin.take());
    child.wait().unwrap();
}

#[test]
fn test_unknown_tool_error() {
    let mut child = spawn_server();

    let resp = send_and_receive(
        &mut child,
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"nonexistent","arguments":{}}}"#,
    );
    let json: serde_json::Value = serde_json::from_str(resp.trim()).unwrap();
    // Unknown tool returns as CallToolResult with isError, not a JSON-RPC error
    assert_eq!(json["result"]["isError"], true);

    drop(child.stdin.take());
    child.wait().unwrap();
}
