//! MCP client example: dynamic API over Streamable HTTP (+ notifications).
//!
//! Run:
//!   cargo run -p stand-in-client --example http_client --features http
//!
//! Before running this example, start the `all_features` server:
//!   cargo run -p stand-in --example all_features --features http
//!
//! (The `all_features` server listens on `http://127.0.0.1:3000`.)
//!
//! Override the server URL via the `MCP_URL` environment variable:
//!   MCP_URL=http://other:8080/mcp cargo run -p stand-in-client --example http_client --features http
//!
//! This example demonstrates:
//! - Connecting via `HttpTransport` to a Streamable HTTP MCP server
//! - Inspecting server capabilities and cached lists
//! - Calling tools, reading resources, and getting prompts
//! - Subscribing to resource updates
//! - Consuming the notification broadcast stream (with honest caveat)
//! - Graceful disconnect (DELETE /mcp)

#![cfg(feature = "http")]

use std::time::Duration;

use stand_in_client::prelude::*;

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    let url = match std::env::var("MCP_URL") {
        Ok(u) => u,
        Err(_) => "http://127.0.0.1:3000/mcp".to_string(),
    };

    println!("=== stand-in-client :: Streamable HTTP example ===\n");
    println!("Connecting to: {url}\n");

    // ── Build + connect ──────────────────────────────────────────────────

    let client = Client::builder()
        .transport(HttpTransport::new(&url))
        .client_info("http-example", env!("CARGO_PKG_VERSION"))
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
        println!("  {}{}", t.name, desc_suffix(&t.description));
    }
    println!();

    // ── Cached resources ─────────────────────────────────────────────────

    let resources = client.resources();
    if !resources.is_empty() {
        println!("── Resources ({}) ──", resources.len());
        for r in resources {
            println!("  {}  ({})", r.uri, r.name);
        }
        println!();
    }

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
            println!("  {}{}", p.name, desc_suffix(&p.description));
        }
        println!();
    }

    // ── Call tool: greet ─────────────────────────────────────────────────

    if has_tool(&client, "greet") {
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
    }

    // ── Call tool: add ───────────────────────────────────────────────────

    if has_tool(&client, "add") {
        println!("── Call: add ──");
        let result = client
            .call_tool("add", serde_json::json!({ "a": 10, "b": 32 }))
            .await
            .expect("call_tool(add) should succeed");
        let text = content_text(&result.content);
        println!("  add(10, 32) => \"{text}\"");
        println!();
    }

    // ── Two error planes ─────────────────────────────────────────────────

    println!("── Two error planes ──");

    // Plane 1: isError (tool execution failure is data, not Err)
    // The `divide` tool on `all_features` returns isError on division by zero.
    if has_tool(&client, "divide") {
        let result = client
            .call_tool("divide", serde_json::json!({ "a": 10, "b": 0 }))
            .await
            .expect("call_tool(divide) returns Ok even on error");
        println!(
            "  divide(10, 0) => Ok, is_error = {}  (tool error is DATA)",
            result.is_error.unwrap_or(false)
        );
    }

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
        let concrete = substitute_first_param(tpl, "rust");

        println!("── Read template: {tpl} => {concrete} ──");
        match client.read_resource(&concrete).await {
            Ok(rr) => {
                for c in &rr.contents {
                    match c {
                        ResourceContents::Text { text, .. } => println!("  {text}"),
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

    // ── Get a prompt ─────────────────────────────────────────────────────

    if !client.prompts().is_empty() {
        let prompt_name = &client.prompts()[0].name;
        println!("── Get prompt: {prompt_name} ──");
        match client
            .get_prompt(
                prompt_name,
                serde_json::json!({ "text": "MCP is a protocol for AI-model servers.", "audience": "beginner" }),
            )
            .await
        {
            Ok(result) => {
                for msg in &result.messages {
                    let role = match msg.role {
                        PromptRole::User => "user",
                        PromptRole::Assistant => "assistant",
                    };
                    match &msg.content {
                        PromptContent::Text { text } => {
                            println!("  [{role}] {text}");
                        }
                    }
                }
            }
            Err(e) => println!("  get_prompt returned: {e}"),
        }
        println!();
    }

    // ── Subscribe ────────────────────────────────────────────────────────

    let resources = client.resources();
    if !resources.is_empty() {
        let uri = &resources[0].uri;
        println!("── Subscribe: {uri} ──");
        client
            .subscribe(uri)
            .await
            .expect("subscribe should succeed");
        println!("  Subscribed to resource updates.");
    }

    // ── Notification stream (honest caveat) ──────────────────────────────

    println!("── Notification stream ──");
    println!("  Listening for server-pushed notifications...");
    println!("  (Note: the `all_features` server does not emit live `resources/updated`");
    println!("   notifications — that trigger is roadmap server-side #4.)");
    println!("  The `Disconnected` notification will arrive when the session ends.");
    println!();

    let mut notif_rx = client.notifications();

    // Consume one notification to demonstrate the pattern.
    match tokio::time::timeout(Duration::from_secs(3), notif_rx.recv()).await {
        Ok(Ok(n)) => println!("  Received: {n:?}"),
        Ok(Err(_)) => println!("  (notification channel lagged)"),
        Err(_) => println!("  (no notification within 3 s — expected)"),
    }
    println!();

    // ── Disconnect ───────────────────────────────────────────────────────

    println!("── Disconnecting (DELETE /mcp) ──");
    client
        .disconnect()
        .await
        .expect("disconnect should succeed");

    // The Disconnected notification arrives after transport close.
    if let Ok(Ok(n)) = tokio::time::timeout(Duration::from_secs(3), notif_rx.recv()).await
        && matches!(n, Notification::Disconnected)
    {
        println!("  Received: Disconnected");
    } // channel may already be closed — ignore other outcomes

    println!("Done.");
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

fn has_tool(client: &Client, name: &str) -> bool {
    client.tools().iter().any(|t| t.name == name)
}

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
