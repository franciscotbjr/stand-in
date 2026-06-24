//! MCP client example: dynamic API over stdio transport.
//!
//! Run:
//!   cargo run -p stand-in-client --example stdio_client
//!
//! Without arguments, the example builds and connects to `stand-in-reference`
//! (the reference MCP server from this workspace) automatically.
//!
//! To use a different MCP server:
//!   cargo run -p stand-in-client --example stdio_client -- <program> [args...]
//!
//! This example demonstrates:
//! - Building and connecting a `Client` via `StdioTransport::command`
//! - Inspecting cached tools, resources, and prompts from the handshake
//! - Calling a tool with typed JSON arguments
//! - Reading a concrete and a template resource by URI
//! - The two error planes: tool errors (data) vs protocol errors (Err)
//! - Subscribe / unsubscribe flow
//! - Graceful disconnect

use std::path::PathBuf;
use std::time::Duration;

use stand_in_client::prelude::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Locates the `stand-in-reference` binary in the workspace target directory.
/// Builds it automatically if the binary is missing.
fn reference_binary() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // up to workspace root
    path.push("target");
    path.push("debug");
    if cfg!(windows) {
        path.push("stand-in-reference.exe");
    } else {
        path.push("stand-in-reference");
    }

    if !path.exists() {
        eprintln!("  Building stand-in-reference...");

        let status = std::process::Command::new(env!("CARGO"))
            .args(["build", "-p", "stand-in-reference"])
            .status()
            .expect("Failed to run cargo build");

        if !status.success() {
            eprintln!("Failed to build stand-in-reference.");
            std::process::exit(1);
        }
    }

    path
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    let (program, cmd_args) = if args.len() > 1 {
        let c: Vec<&str> = args[2..].iter().map(|s| s.as_str()).collect();
        (args[1].clone(), c)
    } else {
        let binary = reference_binary();
        (binary.to_string_lossy().to_string(), Vec::new())
    };

    println!("=== stand-in-client :: stdio example ===\n");
    println!("Connecting to: {program} {}\n", cmd_args.join(" "));

    // ── Build + connect ──────────────────────────────────────────────────

    let client = Client::builder()
        .transport(StdioTransport::command(&program, &cmd_args))
        .client_info("stdio-example", env!("CARGO_PKG_VERSION"))
        .timeout(Duration::from_secs(15))
        .connect()
        .await
        .expect("Failed to connect to MCP server");

    // ── Server identity ──────────────────────────────────────────────────

    let info = client.server_info();
    println!("Server: {} v{}", info.name, info.version);
    let caps = client.server_capabilities();
    println!(
        "Capabilities: tools={} prompts={} resources={}",
        caps.tools.is_some(),
        caps.prompts.is_some(),
        caps.resources.is_some()
    );
    println!();

    // ── Cached tools ─────────────────────────────────────────────────────

    println!("── Tools ({}) ──", client.tools().len());
    for t in client.tools() {
        println!("  {}{}", t.name, desc_suffix(&t.description),);
    }
    println!();

    // ── Cached resources ─────────────────────────────────────────────────

    println!("── Resources ({}) ──", client.resources().len());
    for r in client.resources() {
        println!("  {}  ({})", r.uri, r.name);
    }
    println!();

    // ── Cached resource templates ────────────────────────────────────────

    let templates = client.resource_templates();
    if !templates.is_empty() {
        println!("── Resource Templates ({}) ──", templates.len());
        for t in templates {
            println!("  {}  ({})", t.uri_template, t.name);
        }
        println!();
    }

    // ── Cached prompts ───────────────────────────────────────────────────

    if !client.prompts().is_empty() {
        println!("── Prompts ({}) ──", client.prompts().len());
        for p in client.prompts() {
            println!("  {}{}", p.name, desc_suffix(&p.description),);
        }
        println!();
    }

    // ── Call a tool ──────────────────────────────────────────────────────

    println!("── Call: greet ──");
    let result = client
        .call_tool("greet", serde_json::json!({ "name": "stand-in" }))
        .await
        .expect("call_tool(greet) should succeed");
    let text = content_text(&result.content);
    println!("  greet(\"stand-in\") => \"{text}\"");
    println!(
        "  is_error: {}",
        result
            .is_error
            .map_or("none".to_string(), |v| v.to_string())
    );
    println!();

    // ── Call: add ────────────────────────────────────────────────────────

    println!("── Call: add ──");
    let add_result = client
        .call_tool("add", serde_json::json!({ "a": 10, "b": 32 }))
        .await
        .expect("call_tool(add) should succeed");
    let add_text = content_text(&add_result.content);
    println!("  add(10, 32) => \"{add_text}\"");
    println!();

    // ── Two error planes ─────────────────────────────────────────────────

    println!("── Two error planes ──");

    // Plane 1: tool execution error → Ok with isError = true
    let result = client
        .call_tool("nope", serde_json::json!({}))
        .await
        .expect("call_tool for unknown tool returns Ok (data error)");
    println!(
        "  call_tool(\"nope\") => Ok, is_error = {}  (tool error is DATA)",
        result.is_error.unwrap_or(false)
    );

    // Plane 2: protocol error → Err
    let err = client.read_resource("bad://nonexistent").await.unwrap_err();
    println!("  read_resource(\"bad://nonexistent\") => Err({err})  (protocol error is Err)");
    println!();

    // ── Read a resource ──────────────────────────────────────────────────

    let resources = client.resources();
    if !resources.is_empty() {
        let uri = &resources[0].uri;
        println!("── Read resource: {uri} ──");
        let rr = client
            .read_resource(uri)
            .await
            .expect("read_resource should succeed");
        for c in &rr.contents {
            match c {
                ResourceContents::Text { text, .. } => println!("  {text}"),
                ResourceContents::Blob { blob, .. } => {
                    println!("  [blob, {} bytes base64]", blob.len())
                }
            }
        }
        println!();
    }

    // ── Read a template resource ─────────────────────────────────────────

    let templates = client.resource_templates();
    if !templates.is_empty() {
        let tpl = &templates[0].uri_template;
        let concrete = substitute_first_param(tpl, "stand-in");

        println!("── Read template: {tpl} => {concrete} ──");
        match client.read_resource(&concrete).await {
            Ok(rr) => {
                for c in &rr.contents {
                    match c {
                        ResourceContents::Text { text, .. } => {
                            println!("  {text}");
                        }
                        ResourceContents::Blob { blob, .. } => {
                            println!("  [blob, {} bytes base64]", blob.len())
                        }
                    }
                }
            }
            Err(e) => println!("  (template read returned: {e})"),
        }
        println!();
    }

    // ── Subscribe / unsubscribe ──────────────────────────────────────────

    let resources = client.resources();
    if !resources.is_empty() {
        let uri = &resources[0].uri;
        println!("── Subscribe: {uri} ──");
        client
            .subscribe(uri)
            .await
            .expect("subscribe should succeed");
        println!("  Subscribed.");

        println!("── Unsubscribe: {uri} ──");
        client
            .unsubscribe(uri)
            .await
            .expect("unsubscribe should succeed");
        println!("  Unsubscribed.");
        println!();
    }

    // ── Disconnect ───────────────────────────────────────────────────────

    println!("── Disconnecting ──");
    client
        .disconnect()
        .await
        .expect("disconnect should succeed");
    println!("Done.");
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

fn desc_suffix(desc: &str) -> String {
    if desc.is_empty() {
        String::new()
    } else {
        format!(" — {desc}")
    }
}

fn content_text(content: &[Content]) -> String {
    content
        .iter()
        .map(|c| match c {
            Content::Text { text } => text.clone(),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Replaces the first `{param}` in a URI template with `value`.
fn substitute_first_param(template: &str, value: &str) -> String {
    let start = template.find('{');
    let end = template.find('}');
    match (start, end) {
        (Some(s), Some(e)) if s < e => {
            format!("{}{value}{}", &template[..s], &template[e + 1..])
        }
        _ => template.to_string(),
    }
}
