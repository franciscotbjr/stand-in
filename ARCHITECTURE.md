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

## stand-in-client (SDK cliente)

The `stand-in-client` crate is the client-side counterpart of `stand-in`, published independently at `0.1.0`.

### Architecture

```
stand-in-client/
├── Cargo.toml                  # version=0.1.0; deps: stand-in(default-features=false), tokio, ...
├── src/
│   ├── lib.rs                  # crate root: #[deny(missing_docs)], re-exports, pub prelude
│   ├── error.rs                # Error enum + Result alias (thiserror, manual From impls)
│   ├── client.rs               # Client + ClientBuilder (high-level async API)
│   ├── connection.rs           # Internal actor: read-loop, id↔oneshot correlation (alive),
│   │                           #   broadcast of notifications, handshake orchestration
│   ├── correlation.rs          # PendingRequests (Arc<Mutex<HashMap>>), atomic id counter
│   ├── notification.rs         # Notification type delivered to consumers
│   ├── prelude.rs              # Re-exports: Client, ClientBuilder, StdioTransport, etc.
│   └── transport/
│       ├── mod.rs              # ClientTransport trait (async-trait, Send + Sync)
│       ├── stdio.rs            # Subprocess launch, stdin/stdout JSON framing, kill_on_drop
│       ├── http.rs             # reqwest POST + Mcp-Session-Id + GET SSE + DELETE
│       └── sse.rs              # SSE event parser (emits all events, not just last)
├── tests/
│   ├── api_surface.rs          # Compile-time: prelude is self-sufficient
│   ├── stdio_client.rs         # Integration: Client ↔ stand-in-reference via stdio
│   ├── http_client.rs          # Integration: Client ↔ stand-in via Streamable HTTP
│   └── macro_client.rs         # Integration: #[mcp_client] typed stub ↔ server in-process
└── examples/
    ├── stdio_client.rs          # Dynamic API, auto-builds reference server
    ├── http_client.rs           # Dynamic API + notifications (requires http feature)
    └── typed_client.rs          # #[mcp_client] typed stub (self-contained, in-process)
```

```
stand-in-client-macros/
├── Cargo.toml                  # proc-macro = true; deps: syn, quote, proc-macro2
└── src/
    ├── lib.rs                  # Entry point: #[proc_macro_attribute] for mcp_client
    └── mcp_client.rs           # Trait → typed client struct expansion
```

### Key Design Decisions

#### Client engine: alive read-loop with id↔oneshot correlation

**Decision:** The `Client` spawns a background read-loop (`connection.rs`) that drains the transport's incoming stream and routes each message:
- Messages **with an `id`** → resolved via `PendingRequests` (a `HashMap<Id, oneshot::Sender>`), waking the awaiting `call_tool`/`read_resource`/`get_prompt` future.
- Messages **without an `id`** (notifications) → broadcast to all subscribers via `tokio::sync::broadcast`.

**Rationale:** This is the "alive" read-loop that failed 3 times in earlier attempts. The receiver must be **held** (not discarded) so pending futures actually wake. Tests use a `FakeTransport` to prove the loop routes both paths deterministically.

#### Two error planes (never conflate)

- **Execution errors** (`CallToolResult { isError: true }`) are **data** returned by `call_tool()` as `Ok(...)`.
- **Protocol/transport errors** (bad JSON-RPC, handshake failure, I/O) are returned as `Err(Error::...)`.

The typed `#[mcp_client]` layer **collapses** both planes into `Err(Error::ToolError(...))` for ergonomic Rust `Result` usage.

#### Transport seam: `ClientTransport` trait

**Decision:** `async_trait` with interior mutability (`tokio::Mutex` for receive buffers). Both transports implement `send(&self)` + `receive(&self)` (shared, not `&mut self`), so the read-loop can own the receive side while `Client` methods share the send side.

**Transports:**
- **Stdio** — launches a subprocess via `tokio::process::Command`, communicates via newline-delimited JSON on stdin (write) and stdout (read), stderr drained to `tracing`, `kill_on_drop(true)`.
- **Streamable HTTP** (`http` feature) — `POST /mcp` for request/response with `Accept: application/json, text/event-stream`; captures `Mcp-Session-Id` from the response and replays it on subsequent requests; opens a persistent `GET /mcp` SSE stream for server→client notifications; both POST responses and SSE events feed a single queue drained by `receive()`, keeping the read-loop transport-agnostic.

#### Crate structure: two crates, macro optional

Mirrors the `stand-in` / `stand-in-macros` split:
- `stand-in-client` — the dynamic core (always needed).
- `stand-in-client-macros` — the `#[mcp_client]` proc macro (behind the `macros` feature, default on).

#### Reuse of `stand-in` types

`stand-in-client` depends on `stand-in` with `default-features = false`, reusing public serde types: `JsonRpcRequest`, `ToolDefinition`, `CallToolResult`, `Content`, `Resource`, `ResourceTemplate`, `GetPromptResult`, `ServerCapabilities`, `ServerInfo`, `ClientInfo`, `InitializeParams`, etc. The client adds only the client side of the transport layer — no duplication of MCP types.

### Publishing order

1. `stand-in` (0.0.4 — already on crates.io)
2. `stand-in-client-macros` (0.1.0)
3. `stand-in-client` (0.1.0)

`stand-in-client-macros` must be published before `stand-in-client` because `cargo publish` discards `path` and resolves `stand-in-client-macros` by `version` from crates.io.

## stand-in-mcp-studio-ds (Design System)

The `stand-in-mcp-studio-ds` crate (`publish = false`, path dep) is the design system for the native desktop MCP client. It depends on `egui` (not `eframe`) and `jandi-colors`, and provides:

- **`ds::theme::Tokens`** — 34-field ramp (surface/text/border, primary, code_bg, shadow) with dark/light palettes derived from `jandi-colors`, semantic OKLCH colors (ok/warn/err), WCAG contrast ratio checks, `Density` 5-vars, and `apply_theme`/`ui.ds_tokens()` for zero raw-color rendering
- **`ds::Icon`** — 19 hand-drawn glyphs as `egui::Shape` (zero dependencies, crisp at any DPI)
- **`DsWidget` trait** — contract for stateless atoms (`impl egui::Widget` + default methods for focus ring, press nudge, disabled state)
- **`DsStatefulWidget` trait** — `fn ui(self, ui: &mut Ui, state: &mut State) -> Response` for components that persist mutation across frames (segmented control, checkbox, field, list selection, accordion)
- **28+ components** — atoms (Button, IconButton, CopyButton, ToggleLink, Badge, CapabilityPill, Field, Checkbox, SegmentedControl, ConnectionDot, Spinner, Timing, SectionLabel) and molecules (Panel, PresetCard, ListItem, ListSearch, DetailHeader, Meta, CodeView, ContentBlocks, Message, LogRow, HistoryRow, EmptyState, HintBar, PrivacyBadge, BrandMark)
- **`motion.rs`** — animation timings (hover 120ms, bg 140ms, press 40ms, pulse 1s, spinner 0.6s, fade 220ms) via `ctx.animate_*`

**Anti-facade boundary (D24):** The app crate ships `clippy.toml` with `disallowed-methods` and `disallowed-types` (`-D warnings`) banning raw egui widget constructors and raw colors outside the `ds` crate. Screens compose only `ds::*` — enforcement is at compile time, not review.

## stand-in-mcp-studio (MCP Explorer)

The `stand-in-mcp-studio` crate (`publish = false`, binary) is a native desktop MCP inspector built with egui/eframe (**wgpu** backend). It consumes the `stand-in-client` SDK and focuses exclusively on the UI layer — it never reimplements the MCP protocol engine.

### Architecture

```
stand-in-mcp-studio-ds/     (publish = false — Design System crate, D24; path dep)
├── Cargo.toml              # deps: egui, epaint, jandi-colors; embedded fonts
└── src/
    ├── lib.rs              # DS facade + ui.ds_tokens()
    ├── theme/              # Tokens, palette, semantic, density, typography, apply
    ├── widget.rs           # DsWidget + DsStatefulWidget traits
    ├── icon.rs             # 19 hand-drawn glyphs
    ├── motion.rs           # Animation timings
    └── <component>.rs      # One file per component (atom/molecule)

stand-in-mcp-studio/        (publish = false — App binary)
├── Cargo.toml              # deps: eframe/egui wgpu, stand-in-mcp-studio-ds, stand-in-client (http), tokio, directories, serde
├── clippy.toml             # disallowed-methods/types: ban raw egui + raw colors outside ds crate (D24)
├── src/
│   ├── main.rs             # eframe entry (WGPU): maximized window, engine thread, --capture mode
│   ├── app.rs              # JandiApp: eframe::App; owns state + channels; shell layout
│   ├── capture.rs          # --capture: deterministic fixture → ViewportCommand::Screenshot → PNG
│   ├── bridge/             # Async bridge — NO egui here (headless-testable)
│   │   ├── command.rs      # UiCommand (UI → engine)
│   │   ├── event.rs        # EngineEvent (engine → UI)
│   │   └── engine.rs       # Runs stand-in-client::Client; maps Cmd↔Event; bridges notifications()
│   ├── state/              # Connection, session, history, logs, settings + view types
│   ├── i18n/               # Lang { PtBr, En, Es } + dictionaries
│   └── ui/                 # App-level screen compositions (sidebar, topbar, tab_bar, tabs, settings, onboarding)
│                           #   Compose ONLY ds::* — clippy disallowed enforces this at compile time
├── screenshots/
│   ├── prototype/          # Reference screenshots from the React prototype (dark+light, all screens)
│   └── mK-*.png            # Per-milestone captures (visual gate D22 artifact)
└── tests/
    ├── bridge_stdio.rs     # Headless engine test against stand-in-reference via stdio
    └── bridge_http.rs      # Headless engine test against all_features via HTTP (subscribe→notify)
```

### Key Design Decisions

#### Async bridge: `UiCommand`/`EngineEvent` over the SDK

**Decision:** The GUI (eframe main thread) and the protocol engine (tokio background thread) communicate via two channels:

- **`UiCommand`** — GUI → engine (connect, disconnect, list tools, call tool, read resource, subscribe, etc.)
- **`EngineEvent`** — engine → GUI (connection state changes, tool/resource/prompt lists, results, notifications)

The engine thread runs a `while let Ok(cmd) = rx.recv()` loop over `stand-in-client::Client`. Every `EngineEvent` triggers `request_repaint()` so the UI stays live. The `RepaintHook` trait (with `NoopRepaint` for headless tests and `EguiRepaint` for real app) makes the bridge testable without opening a window.

**Rationale:** This is the pattern that failed 3 times in earlier iterations. The engine must never be on the GUI thread (blocking), and the repaint must be cabled to every state-changing event (otherwise the UI "freezes"). Making the repaint injectable enables the headless `bridge_stdio.rs` and `bridge_http.rs` integration tests.

#### State: pure reducer, GUI-agnostic

The `state/` module defines `AppState` (connection, session info, tool/resource/prompt lists, subscription set, notification ring buffer, history ring buffer, settings, UI-local selection/filter state) and a pure `apply(&mut AppState, EngineEvent)` reducer. This keeps state mutation decoupled from rendering and testable in isolation.

#### Visibility: `--capture` visual gate (D22)

The `--capture` mode loads a deterministic fixture, renders the screen via `ViewportCommand::Screenshot` (wgpu-only), and writes a PNG to `screenshots/`. This provides an automated visual regression gate — each milestone's screenshot is compared against the corresponding React prototype screenshot. The `--capture` mode lives in the codebase; actual capture is executed by the reviewer (PM) who has a display, not by the headless CI/engineer.

#### Two crates: DS separated with clippy enforcement (D24)

The Design System is a separate crate (`stand-in-mcp-studio-ds`) with `clippy.toml` `disallowed-methods` and `disallowed-types` in the app crate banning raw egui widget constructors and raw `Color32::from_*`/`jandi_colors::*` outside of `ds`. This makes the anti-facade boundary a **compile error**, not a review checklist item. The DS crate has no eframe dependency — it is pure egui.

#### Backend: wgpu (D2)

eframe runs on the **`wgpu`** backend. The primary reason is that `glow` in egui 0.34 dropped pixel capture — without `wgpu`, the `--capture` visual gate cannot produce screenshots. The cost: Linux needs Vulkan system dependencies (installed in `binaries.yml`).

## Error Handling

Tool execution errors are returned as `CallToolResult` with `isError: true` — they are **not** JSON-RPC errors. JSON-RPC errors are reserved for protocol-level problems (parse error, invalid request, method not found, invalid params, internal error).
