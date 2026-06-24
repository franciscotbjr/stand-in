# stand-in-mcp-explorer — MCP Explorer

Native desktop MCP client built on [GPUI](https://github.com/zed-industries/zed) and
[gpui-component](https://github.com/longbridge/gpui-component). Compose the
[025 Design System](https://github.com/franciscotbjr/stand-in/tree/feature/025-stand-in-mcp-explorer-new-version/stand-in-mcp-explorer/stand-in-mcp-explorer-ds)
to inspect, invoke, and debug any MCP server over Stdio, SSE, or Streamable HTTP.

## How to run

```bash
# Launch the app (requires a display — WGPU backend)
cargo run --manifest-path stand-in-mcp-explorer/Cargo.toml -p stand-in-mcp-explorer

# Capture mode — renders a deterministic fixture state and takes an OS-level screenshot
cargo run --manifest-path stand-in-mcp-explorer/Cargo.toml -p stand-in-mcp-explorer -- --capture <region> <state> <mode>

# Release build
cargo build --release --manifest-path stand-in-mcp-explorer/Cargo.toml -p stand-in-mcp-explorer
```

## Architecture

| Layer | What it does |
|-------|-------------|
| **Tokio ↔ UI bridge** | A dedicated tokio thread owns the `stand-in-client` SDK (connect, list, call, read, subscribe). Two channels (`UiCommand` / `EngineEvent`) talk to the GPUI main thread; state changes trigger `cx.notify()`. Zero `tokio::*` on the GPUI executor (M38 class). |
| **Transports** | Stdio (subprocess launch), SSE (HTTP `GET /mcp` SSE stream), Streamable HTTP (`POST`/`GET`/`DELETE /mcp` with `Mcp-Session-Id`). |
| **i18n** | `tr(key, lang)` with mandatory `Lang` parameter (PtBr / En / Es). PtBr fallback; no convenience function with a hardcoded default (anti-footgun BUG-7). ~115 keys per language across 14 namespaces. |
| **Persistence** | `servers.json` and `settings.json` in the OS project directory (`directories::ProjectDirs`). Auto-save on `Connected` with dedup by config. |
| **Design System** | The app composes the 025 DS (`stand-in-mcp-explorer-ds`, path dep). All rendering goes through `cx.theme()` tokens and DS components — zero raw GPUI in screens, zero literal colours. |
| **File organization** | One component per file, one folder per region (`app/`, `bars/{sidebar,topbar}/`, `tabs/{tools,resources,prompts,notifications,history}/`, `screens/`). |

## 10 regions (M5–M15)

| Region | What it holds |
|--------|--------------|
| **Sidebar** | Connection form with real inputs (command, args, url, env rows), transport segmented control (Stdio/SSE/HTTP), presets, saved servers (collapse when empty), privacy badge. |
| **Topbar** | Connection dot (on/busy/off/err) with name/meta, capability counts + latency, language switch, guided toggle, reconnect. |
| **Tabbar** | 5 tabs (Tools, Resources, Prompts, Notifications, History) with icon + count; underline active; routing. |
| **Work-area** | Split (list+detail) or single-column modes; explicit flex-col chain with `min_h(0)` + `overflow` at every leaf (§5b/BUG-11). |
| **Tools** | List with live search, params form generated from `InputSchema` (real `Field`+`InputState`), Run with typed args, result panel across 3 error planes (isError-data, protocol, transport). |
| **Resources** | Concrete + template lists, content viewer (text via `JsonView`, binary warning), metadata grid, subscribe/unsubscribe with notification indicator. |
| **Prompts** | List with search, args form, `get_prompt` via bridge, message preview with role badge. |
| **Notifications** | Full-width feed (newest-first), 5-level filter + count + Clear, 5 distinct colours (warn ≠ err), §5b scroll. |
| **History** | Accordion of real bridge calls (newest-first), click-expand request/response `JsonView` side by side. |
| **Settings** | Elevated overlay with shadow (BUG-3 — the only shadow in the app), real theme/density/primary/guided controls, persisted. |

## Quality gate

```bash
# From the repo root:
cargo fmt --manifest-path stand-in-mcp-explorer/Cargo.toml --all --check
cargo clippy --manifest-path stand-in-mcp-explorer/Cargo.toml --all-targets -D warnings
cargo test --manifest-path stand-in-mcp-explorer/Cargo.toml
cargo build --manifest-path stand-in-mcp-explorer/Cargo.toml
cargo doc --manifest-path stand-in-mcp-explorer/Cargo.toml --no-deps

# Lint-Flex (GPUI layout lint):
pwsh .stateful-spec/scripts/lint-flex.ps1

# Smoke (app boots without panic, 8 s):
pwsh .stateful-spec/scripts/smoke-open.ps1 -Region shell -StateArg disconnected -Mode dark
```

Test suite: **163 unit** + **12 bridge integration** (against `stand-in-reference` stdio) + **6 app-interaction** (headless `TestAppContext`) = **181 app tests**. Plus 213 DS + 5 geometry + 7 gallery = **406 total** in the explorer workspace.

## Pinned dependencies

| Crate | Rev | Mechanism |
|-------|-----|-----------|
| `gpui` | `#3f5705b9` | `Cargo.lock` (lock-only; O-005) |
| `gpui-component` | `#70d2c44b` | `rev=` in `Cargo.toml` |
| `gpui-component-assets` | `#70d2c44b` | `rev=` in `Cargo.toml` |

Toolchain: Rust 1.95.0 (`rust-toolchain.toml` at repo root), edition 2024.

## Design System audit

The app composes the 025 Design System. An [audit of the 8 prohibitions](docs/audit-8-prohibitions.md)
confirms zero violations — all rendering goes through `cx.theme()` tokens and DS components,
icons are on-catalog (`IconName::*`), fonts are Hanken Grotesk + JetBrains Mono (via
`DsAssets`), the only shadow is the legitimate Settings overlay, and the DS crate was
touched only by the M2 contrast fix (a deliberate canon update, per iteration D-4).
