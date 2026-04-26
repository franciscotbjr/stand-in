# Architecture

> High-level design decisions and module structure for the stand-in project.

## Module Structure

```
stand-in/src/
├── lib.rs              # Crate root, prelude, re-exports
├── error.rs            # Error enum + Result alias
├── protocol/           # JSON-RPC 2.0 wire format
│   ├── request.rs      # JsonRpcRequest
│   ├── response.rs     # JsonRpcResponse
│   ├── error.rs        # JsonRpcError (standard codes)
│   └── notification.rs # JsonRpcNotification
├── tool/               # MCP tool abstraction
│   ├── content.rs      # Content (tagged enum: text)
│   ├── input_schema.rs # InputSchema (JSON Schema)
│   ├── tool_definition.rs  # ToolDefinition
│   ├── call_tool_params.rs # CallToolParams
│   ├── call_tool_result.rs # CallToolResult
│   ├── list_tools_result.rs # ListToolsResult
│   ├── tool_trait.rs   # McpTool trait
│   ├── tool_registry.rs # ToolRegistry
│   └── tool_factory.rs # ToolFactory (inventory integration)
├── resource/           # MCP resource abstraction
│   ├── resource_trait.rs   # McpResource trait
│   ├── resource_registry.rs # ResourceRegistry
│   ├── resource_factory.rs # ResourceFactory (inventory integration)
│   ├── resource_definition.rs # Resource
│   ├── resource_template.rs  # ResourceTemplate
│   ├── resource_contents.rs  # ResourceContents (text/blob)
│   ├── resource_annotations.rs # ResourceAnnotations
│   ├── read_resource_params.rs  # ReadResourceParams
│   ├── read_resource_result.rs  # ReadResourceResult
│   ├── list_resources_result.rs # ListResourcesResult
│   ├── list_resource_templates_result.rs # ListResourceTemplatesResult
│   ├── subscribe_params.rs  # SubscribeParams
│   └── unsubscribe_params.rs # UnsubscribeParams
├── server/             # MCP server types + dispatch
│   ├── server_capabilities.rs # ServerCapabilities
│   ├── tools_capability.rs    # ToolsCapability
│   ├── server_info.rs  # ServerInfo
│   ├── client_info.rs  # ClientInfo
│   ├── initialize_params.rs   # InitializeParams
│   ├── initialize_result.rs   # InitializeResult
│   └── handler.rs      # RequestHandler (method dispatch)
└── transport/          # Communication layer
    ├── transport_trait.rs # Transport trait
    ├── stdio.rs        # StdioTransport (feature: stdio)
    ├── http_transport.rs # HttpTransport + axum handlers (feature: http)
    ├── session.rs      # Session struct (feature: http)
    ├── session_store.rs # SessionStore (feature: http)
    └── sse.rs          # SSE event helpers (feature: http)

stand-in-macros/src/
├── lib.rs              # Proc macro entry points
├── mcp_tool.rs         # #[mcp_tool] expansion
├── mcp_server.rs       # #[mcp_server] expansion
├── mcp_prompt.rs       # #[mcp_prompt] expansion
├── mcp_resource.rs     # #[mcp_resource] expansion
└── schema.rs           # Rust type → JSON Schema inference
```

## Key Design Decisions

### Tool Discovery: `inventory` Crate

**Decision:** Use the [`inventory`](https://crates.io/crates/inventory) crate for automatic tool registration.

**Alternatives considered:**
- **Explicit list** (`#[mcp_server(tools = [greet, weather])]`) — requires the user to manually enumerate every tool, which is error-prone and scales poorly.
- **`inventory` crate** — each `#[mcp_tool]` auto-registers via `inventory::submit!`, and `#[mcp_server]` collects all registered tools via `inventory::iter`. Zero boilerplate for the user.

**Rationale:** The `inventory` pattern mirrors how test frameworks and plugin systems work in Rust. It eliminates the need for users to maintain a tool list, which would be a common source of bugs (forgetting to add a new tool). The trade-off is a linker-level dependency, but `inventory` is well-tested on Linux, macOS, and Windows.

**How it works:**
1. `#[mcp_tool]` generates a `ToolFactory` struct and submits it via `inventory::submit!`
2. `#[mcp_server]` collects all factories via `inventory::iter::<ToolFactory>`
3. Each factory is called to create tool instances that are registered in the `ToolRegistry`

### Protocol Version

**Decision:** Target MCP protocol version `2025-03-26` (latest spec at time of implementation).

### Server Identity

**Decision:** Auto-derive server name and version from `Cargo.toml` via `env!("CARGO_PKG_NAME")` and `env!("CARGO_PKG_VERSION")`. No macro attributes needed.

### One Type Per File

**Decision:** Each public type lives in its own file with a facade `mod.rs`. This keeps files small, focused, and easy to navigate. Co-located `#[cfg(test)]` modules provide unit tests.

### Transport Abstraction

**Decision:** The `Transport` trait abstracts communication. Two implementations exist:
- **`StdioTransport`** (feature: `stdio`, default) — line-delimited JSON-RPC over stdin/stdout
- **`HttpTransport`** (feature: `http`) — MCP 2025-03-26 Streamable HTTP: POST/GET/DELETE on `/mcp`, session management via `Mcp-Session-Id` header, SSE for notifications, CORS via `tower-http`

The `#[mcp_server]` macro generates both `serve(transport)` (generic) and `serve_http()` (convenience, feature-gated). Optional `host`/`port` attributes control the HTTP bind address: `#[mcp_server(host = "0.0.0.0", port = 8080)]`.

### HTTP Session Management

**Decision:** In-memory `SessionStore` using `Arc<RwLock<HashMap<String, Session>>>`. Sessions are created on successful `initialize`, validated on every subsequent request, and removed on `DELETE /mcp`. UUIDs generated via the `uuid` crate.

**Rationale:** Simple, sufficient for single-process servers. Distributed session stores can be added later without changing the handler logic.

### Resources

**Decision:** Resources follow the same pattern as tools and prompts: `McpResource` trait, `ResourceRegistry`, `ResourceFactory` with `inventory::submit!`. Both concrete resources (fixed URI) and template resources (URI with `{param}` placeholders) are supported.

**How it works:**
1. `#[mcp_resource(uri, name, description, mime_type)]` declares a resource
2. If the URI contains `{param}` placeholders, it's a template resource; function parameters become template variables
3. The macro detects the return type (`Result<String>` → `TextResourceContents`)
4. `resources/list` returns concrete resources; `resources/templates/list` returns template resources
5. `resources/read` matches URIs: exact match on concrete, template pattern match on templates
6. `resources/subscribe` and `resources/unsubscribe` manage SSE notification subscriptions via `ResourceRegistry`

**Rationale:** Mirrors the pattern established by tools and prompts for consistency. Template matching uses simple `{param}` substring detection split by `/` segments — sufficient for 95% of use cases without an RFC 6570 dependency.

## Error Handling

Tool execution errors are returned as `CallToolResult` with `isError: true` — they are **not** JSON-RPC errors. JSON-RPC errors are reserved for protocol-level problems (parse error, invalid request, method not found, invalid params, internal error).
