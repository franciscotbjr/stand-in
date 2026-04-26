# Iteration 009 — Resources Support

- **Type:** feature
- **Status:** done
- **Created:** 2026-04-25
- **Completed:** 2026-04-25
- **Author:** Francisco Tarcizo Bomfim Jr.

## Goal

Implement the `#[mcp_resource]` macro and the full resources subsystem (types, trait, registry, dispatch, capability advertisement, subscribe/unsubscribe with SSE notification sending), following the identical patterns used for `#[mcp_tool]` and `#[mcp_prompt]`.

## Constraints (Non-Negotiable)

- **All tests must pass** at every milestone — unit tests (co-located `#[cfg(test)] mod tests`) and integration tests (`stand-in/tests/`)
- **All quality gates must pass** at M9 final verification:

```bash
cargo fmt --check
cargo clippy --all-features -- -D warnings
cargo test --all-features
cargo build --all-features
cargo doc --all-features --no-deps
```

## Architecture

Following the exact patterns of `tools/` and `prompts/`, the resources subsystem lives in `stand-in/src/resource/` with a mirror macro `mcp_resource.rs` in `stand-in-macros/src/`.

### Key Types

| Concept | File | Pattern from |
|---------|------|-------------|
| `McpResource` trait | `stand-in/src/resource/resource_trait.rs` | `McpTool` / `McpPrompt` |
| `ResourceRegistry` | `resource_registry.rs` | `PromptRegistry` |
| `ResourceFactory` | `resource_factory.rs` | `PromptFactory` |
| `Resource` struct | `resource_definition.rs` | `ToolDefinition` |
| `ResourceTemplate` struct | `resource_template.rs` | `PromptDefinition` |
| `ResourceContents` enum | `resource_contents.rs` | `Content` enum |
| `ResourceAnnotations` | `resource_annotations.rs` | New |
| `ReadResourceParams` | `read_resource_params.rs` | `GetPromptParams` |
| `ReadResourceResult` | `read_resource_result.rs` | `GetPromptResult` |
| `ListResourcesResult` | `list_resources_result.rs` | `ListPromptsResult` |
| `ListTemplatesResult` | `list_resource_templates_result.rs` | `ListPromptsResult` |
| `SubscribeParams` | `subscribe_params.rs` | `GetPromptParams` |
| `UnsubscribeParams` | `unsubscribe_params.rs` | `GetPromptParams` |
| `ResourcesCapability` | `stand-in/src/server/resources_capability.rs` | `ToolsCapability` |

### New Dependency

- None. URI template matching uses simple `{param}` split-by-`/` detection (implemented in `mcp_resource.rs` and `resource_registry.rs`). The `uritemplate` crate was considered in the spec but deferred — the simple approach covers 95% of use cases without an external dependency.

## Milestones

- [x] M1 — Core resource types (10 files + `mod.rs`)
- [x] M2 — `McpResource` trait, `ResourceFactory`, `ResourceRegistry`
- [x] M3 — Server integration (`RequestHandler`, `ServerCapabilities`, `ResourcesCapability`, `Error`)
- [x] M4 — `#[mcp_resource]` macro (`stand-in-macros/src/mcp_resource.rs`)
- [x] M5 — `mcp_server` macro update (collect `ResourceFactory`, register in `ResourceRegistry`)
- [x] M6 — Subscribe notification wiring (SSE sender hookup in HTTP transport)
- [x] M7 — Prelude exports + library wiring
- [x] M8 — Integration tests + reference server + examples
- [x] M9 — Verify (quality gates, ARCHITECTURE.md, CHANGELOG, memory.md)

## Milestone Details

### M1 — Core resource types

**Files (10 + facade):** `resource_contents.rs`, `resource_annotations.rs`, `resource_definition.rs`, `resource_template.rs`, `read_resource_params.rs`, `read_resource_result.rs`, `list_resources_result.rs`, `list_resource_templates_result.rs`, `subscribe_params.rs`, `unsubscribe_params.rs`, `mod.rs`

**Tests (unit, co-located):**
| File | Test scenarios |
|------|---------------|
| `resource_contents.rs` | Text serializes with `type: "text"`; Blob serializes with `type: "blob"`; camelCase fields; optional `mimeType` skipped when None; round-trip deserialization |
| `resource_annotations.rs` | Optional fields skipped when None; deserialization |
| `resource_definition.rs` | Serialization/deserialization; optional field skipping |
| `resource_template.rs` | Serialization with `uriTemplate`, `name`; optional description/mimeType |
| `read_resource_params.rs` | Deserialization from JSON; missing `uri` field → error |
| `read_resource_result.rs` | Serialization with single/multiple contents |
| `list_resources_result.rs` | Empty list; populated list; `nextCursor` optional |
| `list_resource_templates_result.rs` | Empty list; populated list; `nextCursor` optional |
| `subscribe_params.rs` | Deserialization with `uri` |
| `unsubscribe_params.rs` | Deserialization with `uri` |

---

### M2 — McpResource trait + ResourceRegistry + ResourceFactory

**Files:** `resource_trait.rs`, `resource_registry.rs`, `resource_factory.rs`

**McpResource trait:**
```rust
#[async_trait]
pub trait McpResource: Send + Sync + Debug {
    fn uri(&self) -> &str;
    fn name(&self) -> &str;
    fn description(&self) -> Option<&str> { None }
    fn mime_type(&self) -> Option<&str> { None }
    fn is_template(&self) -> bool { false }
    async fn read(&self, uri: &str) -> Result<ReadResourceResult>;
    fn to_resource(&self) -> Option<Resource> { None }
    fn to_template(&self) -> Option<ResourceTemplate> { None }
}
```

**ResourceRegistry** — `register()` stores `Box<dyn McpResource>`; `list_resources()` returns concrete (`is_template() == false`); `list_templates()` returns templates; `read_resource(uri)` matches URI and dispatches `read()`; `subscribe(uri, session_id, sender)` tracks via `Arc<RwLock<HashMap<String, Vec<(String, broadcast::Sender<String>)>>>>`; `unsubscribe(uri, session_id)` removes; `notify(uri, contents)` sends to all subscribed senders.

**ResourceFactory** — `ResourceFactory(pub fn() -> Box<dyn McpResource>)` with `inventory::collect!`

**Tests (unit):**
| File | Test scenarios |
|------|---------------|
| `resource_trait.rs` | Default `description()` → None; default `mime_type()` → None; default `is_template()` → false; default `to_resource()` → None; default `to_template()` → None |
| `resource_registry.rs` | Register + list returns resources; register + list separates templates; read by URI returns contents; read unknown URI → `ResourceError`; subscribe adds subscription; unsubscribe removes; notify sends to all subscribed; notify no subscribers does not panic; template vs concrete separation |
| `resource_factory.rs` | Factory call produces `Box<dyn McpResource>` |

---

### M3 — Server integration

**Files:** `server/resources_capability.rs` (new), `server/server_capabilities.rs` (edit), `server/handler.rs` (edit), `server/mod.rs` (edit), `error.rs` (edit)

- `ResourcesCapability { subscribe: Option<bool>, list_changed: Option<bool> }` — camelCase, skip_if_none
- `resources: Option<ResourcesCapability>` added to `ServerCapabilities` + `with_resources()` builder
- `RequestHandler::new()` accepts `ResourceRegistry` as third parameter
- `RequestHandler` gains `wire_resource_subscription(&self, uri, session_id, sender)` and `wire_resource_unsubscribe(&self, uri, session_id)` methods
- Dispatch: `resources/list`, `resources/templates/list`, `resources/read`, `resources/subscribe`, `resources/unsubscribe`
- `Error::ResourceError(String)` variant
- `handle_initialize()` advertises `.with_resources(ResourcesCapability { subscribe: Some(true), list_changed: Some(true) })`

**Tests (unit):**
| Location | Test scenarios |
|----------|---------------|
| `handler.rs` | `resources/list` empty; `resources/list` populated; `resources/templates/list` returns templates; `resources/read` known URI; `resources/read` template with filled URI; `resources/read` unknown URI → error; `resources/subscribe` success; `resources/unsubscribe` success; initialize advertises resources capability; unknown method → `-32601` |
| `server_capabilities.rs` | `ResourcesCapability` serialization (camelCase); `ServerCapabilities` with/without resources |

---

### M4 — `#[mcp_resource]` macro

**Files:** `stand-in-macros/src/mcp_resource.rs` (new), `stand-in-macros/src/lib.rs` (edit)

Expansion follows `mcp_prompt.rs` pattern:
1. Parse attributes: `uri` (required), `name` (defaults to function name), `description` (optional), `mime_type` (optional)
2. Extract function signature: parameter names/types, return type, asyncness
3. Map `Option<T>` params → not required; non-Option → required
4. Detect template: URI contains `{param}` → template resource
5. Detect return type: `Result<String>` → wrap in `TextResourceContents`, `Result<Vec<u8>>` → wrap in `BlobResourceContents`
6. Generate `{FnName}Resource` struct with `McpResource` impl
7. In `read()`: if template → use `uritemplate` to extract params from URI, call function with extracted args
8. `inventory::submit! { ResourceFactory(|| Box::new({struct})) }`

**Tests (unit in `mcp_resource.rs`):**
| Test | Description |
|------|-------------|
| `test_to_pascal_case` | PascalCase conversion |
| `test_is_return_type_vec_u8` | Return-type detection (String vs Vec\<u8\>) |
| `test_resource_attrs_parse` | Full attribute parsing (uri, name, description) |
| `test_resource_attrs_parse_minimal` | Only uri attribute |
| `test_resource_attrs_missing_uri_errors` | Error on missing required uri |
| `test_template_detection` | `{param}` presence in URI |

Macro expansion compile tests (`test_concrete_resource_expansion`, `test_template_resource_expansion`, `test_optional_parameters`, `test_required_parameters`, `test_inventory_submit`) are deferred to integration-level testing via the reference server (`stand-in-reference`) and the HTTP test server, which exercise full macro expansion at compile time.

---

### M5 — `mcp_server` macro update

**File:** `stand-in-macros/src/mcp_server.rs` (edit)

- Collect `inventory::iter::<ResourceFactory>` in `serve()` and `serve_http()`
- Register each factory in `ResourceRegistry`
- Pass `resource_registry` to `RequestHandler::new(registry, prompt_registry, resource_registry, server_info)`

**Tests (unit):**
- Generated `serve()` includes `inventory::iter::<ResourceFactory>` loop
- `RequestHandler::new()` receives resource_registry as third argument

---

### M6 — Subscribe notification wiring

**Files:** `transport/http_transport.rs` (edit), `protocol/notification.rs` (edit)

- `handle_post`: after `resources/subscribe` succeeds, extract session's `notification_tx` from `SessionStore` and call `handler.wire_resource_subscription(uri, session_id, sender)`
- `handle_post`: after `resources/unsubscribe` succeeds, call `handler.wire_resource_unsubscribe(uri, session_id)`
- Add notification method constants: `notifications/resources/updated`, `notifications/resources/list_changed`
- `ResourceRegistry::notify()` formats `notifications/resources/updated` as JSON-RPC notification

**Tests (unit):**
| Location | Test scenarios |
|----------|---------------|
| `resource_registry.rs` | `notify()` sends formatted JSON to all subscribed senders; does not send to unsubscribed; does not panic on empty |
| `protocol/notification.rs` | Notification JSON structure matches MCP spec |

---

### M7 — Prelude exports + library wiring

**Files:** `stand-in/src/lib.rs` (add `pub mod resource`), `stand-in/src/resource/mod.rs` (facade), `stand-in/src/server/mod.rs` (add `ResourcesCapability` export)

**Prelude exports:** `Resource`, `ResourceTemplate`, `ResourceContents`, `McpResource`, `ResourceRegistry`, `ReadResourceParams`, `ReadResourceResult`, `ListResourcesResult`, `ListTemplatesResult`, `SubscribeParams`, `UnsubscribeParams`, `ResourceAnnotations`, `ResourcesCapability`

---

### M8 — Integration tests + reference server + examples

**Reference server (`stand-in-reference/src/main.rs`):** Add 2 concrete + 1 template resource functions using `#[mcp_resource]`.

**Stdio integration tests (`tests/stdio_server.rs`):**
| Test | Description |
|------|-------------|
| `test_resources_list` | Returns 2 concrete resources with correct URIs/names |
| `test_resources_templates_list` | Returns 1 template with `uriTemplate` |
| `test_resources_read_concrete` | Read `info://version`; verify text content and URI |
| `test_resources_read_template` | Read `docs://rust/readme`; verify interpolated content |
| `test_resources_read_unknown` | Unknown URI → error response |
| `test_resources_subscribe_unsubscribe` | Subscribe + unsubscribe round-trip |
| `test_resources_unknown_method` | `resources/invalid` → JSON-RPC error `-32601` |

**HTTP integration tests (`tests/http_server.rs`):**
| Test | Description |
|------|-------------|
| `test_resources_list_http` | Full lifecycle: initialize → `resources/list` |
| `test_resources_read_http` | Initialize → `resources/read` |
| `test_resources_subscribe_notify_http` | Initialize → subscribe → notify → verify SSE message |

---

### M9 — Verify

All quality gates must pass:
```bash
cargo fmt --check
cargo clippy --all-features -- -D warnings
cargo test --all-features
cargo build --all-features
cargo doc --all-features --no-deps
```

Documentation updates:
- `ARCHITECTURE.md` — add `resource/` module to structure diagram, add Resources section to key design decisions
- `CHANGELOG.md` — create `## [0.0.4]` section with "Added: `#[mcp_resource]` macro and full resources subsystem"
- `.stateful-spec/memory.md` — update Active Work, Recent Completions, History Index
- `.stateful-spec/history/009-resources.md` — mark as done

## Decisions

| Decision | Rationale | Date |
|----------|-----------|------|
| `Result<String>` → `TextResourceContents`, `Result<Vec<u8>>` → `BlobResourceContents` | Macro detects return type via syn at compile time; covers text and binary resources | 2026-04-25 |
| Simple `{param}` split-by-`/` for URI template matching | Covers 95% of use cases without an RFC 6570 dependency; implemented inline in macro and registry | 2026-04-25 |
| Full subscribe/unsubscribe with SSE notification | Transport wires senders post-dispatch; ResourceRegistry uses `Arc<RwLock<>>` for subscription map only | 2026-04-25 |
| Interior mutability only for subscription map | Rest of registry keeps plain `Vec<>` — same pattern as ToolRegistry/PromptRegistry | 2026-04-25 |
| Template detection via `{param}` in URI | Simple heuristic — URIs with `{...}` are templates, without are concrete | 2026-04-25 |

## Blockers & Risks

| Risk | Mitigation |
|------|-----------|
| Subscribe wiring touches HTTP transport internals | Single call to `wire_resource_subscription()` after dispatch — minimal diff |
| 10 new type files | One type per file per convention; facade `mod.rs` keeps navigation flat |

## References

- **Specification:** This plan document
- **Pattern reference:** Iteration 006 (mcp_prompt), `stand-in/src/prompt/`, `stand-in/src/tool/`
- **MCP spec:** Protocol version 2025-03-26, resources section
- **PR/MR:** —
- **Commits:** —
- **Related Issues:** —
