//! API surface test — proves that `use stand_in_client::prelude::*` imports
//! everything a consumer needs, without ever typing `stand_in::`.
//!
//! This test only needs to **compile** — the types and method chains prove the
//! prelude is self-sufficient. The function bodies exercise every major
//! public type path.

#![allow(unused_imports, dead_code, unused_variables)]

use stand_in_client::prelude::*;

/// Prove we can name every type a consumer touches, including the
/// re-exported `stand-in` types, entirely through the prelude.
fn _compile_time_type_check() {
    // ── Client crate types ─────────────────────────────────────────────
    let _: Client;
    let _: ClientBuilder;
    let _: Error;
    let _: Result<()>;
    let _: Notification;
    let _: StdioTransport;

    #[cfg(feature = "http")]
    {
        let _: HttpTransport;
    }

    // ── Re-exported stand-in types (consumer must not type stand_in::) ──
    let _: Content;
    let _: CallToolResult;
    let _: InputSchema;
    let _: ToolDefinition;

    let _: Resource;
    let _: ResourceTemplate;
    let _: ResourceContents;
    let _: ReadResourceResult;

    let _: PromptDefinition;
    let _: GetPromptResult;
    let _: PromptMessage;
    let _: PromptContent;
    let _: PromptRole;

    let _: ServerInfo;
    let _: ServerCapabilities;
}

/// Prove method chains compile on borrowed and owned handles.
fn _call_chains(client: &Client) {
    let _tools: &[ToolDefinition] = client.tools();
    let _resources: &[Resource] = client.resources();
    let _templates: &[ResourceTemplate] = client.resource_templates();
    let _prompts: &[PromptDefinition] = client.prompts();
    let _info: &ServerInfo = client.server_info();
    let _caps: &ServerCapabilities = client.server_capabilities();
}
