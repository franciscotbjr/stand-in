# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- **Renderização virtualizada (windowed) das listas do MCP Explorer (iteração 031):** as seis
  regiões de lista deixaram de renderizar **todos** os itens de uma vez (`.children(filtered.iter()
  .enumerate().map(...))`) — um servidor com ~4.000 tools construía ~30–40k nós de layout antes do
  paint e esgotava a memória. Agora **Tools/Resources/Prompts** usam `gpui::uniform_list` (linha
  `ListItem` de **altura fixa** — `LIST_ROW_HEIGHT = 84`, extensão de DS) e **Notifications/History**
  usam `gpui::list`/`ListState` (altura variável; o acordeão do History re-mede via
  `ListState::splice` no toggle). Só o subconjunto visível (+ overscan) é materializado → **memória
  limitada pela viewport**, não pelo total de itens. O filtro roda **1×/render** (coleção pré-filtrada
  num `Arc<[T]>` que o closure indexa — nunca `filter_*` por quadro de scroll → rolagem fluida).
  Saved-servers segue eager (já limitada). Sem mudança de API; busca/seleção/clique preservados.
- **stand-in-mcp-explorer-ds consome a crate `jandi-colors` + alinhamento de toolchain
  (iteração 027):** a rampa de 8 degraus da `palette.rs` do DS passa a derivar **direto da
  crate publicada `jandi-colors 0.1.0`** (mesmos hexes do canon `tokens/colors.css`) via uma
  `const fn rgb8_to_hsla`, em vez de literais `Hsla` transcritos à mão — mesmas cores,
  agora computadas; um teste fixa os 8 hexes canônicos contra a crate (anti-drift de
  `cargo update`). As bordas escuras e a sombra clara (rampa-com-alfa) também derivam da
  crate. Os tokens de tema fora da paleta pública (semânticos, superfícies, JSON, `BRAND_RING`)
  seguem transcritos do `colors.css`. **Toolchain unificado** em todos os crates vivos:
  `edition = "2024"` + `rust-version = "1.95.0"` (explorer ds+gallery e o `flow` migrados de
  2021 via `cargo fix --edition` + let-chains da edition 2024; o workspace SDK já era 2024 e
  ganha o `rust-version`, inclusive nas publicadas). Sem mudança de comportamento; gate verde
  nos três workspaces.

### Added

- **stand-in-mcp-explorer-ds — documentação humana do Design System (iteração 026):**
  `README.md` da crate (o que é, o canon e as 8 proibições, a estrutura com o inventário
  rastreado e links, o tema, o uso, a gallery, os testes/gate e a pinagem de revisões) +
  `docs/{core,forms,navigation,data,theme}.md` documentando o lado Rust/GPUI de cada
  componente (API real, comportamento de densidade e tema) com ponteiros para os
  `.prompt.md` do canon — as regras visuais permanecem só no canon (anti-deriva). Redigido
  em pt-BR com as skills de prosa (`prosa-completa` + as 3 que ela agrega), agora
  versionadas em `.claude/skills/`. Para o leitor humano, cada `docs/<grupo>.md` abre com um
  preview visual (screenshot da referência canônica do protótipo — grupo nos quatro docs de
  componente; paleta e densidade no `theme.md`) e cada componente ganha um snippet Rust
  ilustrativo derivado do uso real na gallery (25 no total, conferidos contra as assinaturas
  do fonte).
- **stand-in-mcp-explorer-ds — Design System novo, fiel ao canon (iteração 025):**
  clean slate do crate `-ds` (substitui o DS pré-canon de 77 componentes atoms/molecules por 24
  componentes rastreados 1:1 ao Design System canônico em `stand-in-client-prototipo/`). Taxonomia
  `core/ forms/ navigation/ data/` espelhando `components/` do protótipo; 23 componentes canônicos
  + 1 extensão formal (Select, resolvendo o placeholder `<select>` nativo das refs); palette jandi
  mapeada a `gpui_component::ThemeColor` (dark+light, verificação de contraste); densidade
  compact/regular/comfy; catálogo de 22 glifos via assets GPUI; aderência às 8 proibições do canon
  (zero cor literal fora do tema, sombra só em overlays, mono/sans roteados). Gallery Storybook
  nativa (`stand-in-mcp-explorer-gallery`) com 10 seções, navegação interativa, toolbar dark/light
  + densidade, e modo captura determinístico via CLI. O app Explorer antigo (egui) foi deletado;
  o app GPUI renasce na 026 sobre este DS.

### Changed

- **Metodologia & skills — adoção do Design System do protótipo (iteração 024):**
  `stand-in-client-prototipo/` (tokens/*.css + 23 componentes em core/forms/navigation/data com
  `*.prompt.md` + regras de obediência + ui_kit clicável) vira a **fonte canônica de design**; as 6
  skills foram re-ancoradas (design-system: 8 proibições traduzidas p/ GPUI + invariantes
  DS-flat/icons/mono/density/reuse; ux/frontend-mapping/pm/architect/gpui re-apontadas) — **regra de
  elevação re-escopada: sombra só em overlays, cards in-flow separam por borda 1px**; os 22 PNGs de
  referência foram **recapturados do ui_kit** (`capture-reference.mjs` restaurado e re-apontado;
  conferência ~pixel-idêntica); `flow spawn` agora resolve o modo pelo `step` do `flow-state.md`
  (**APPROVED → spawn commit-only por construção**, zero skills de engenharia; estado ilegível →
  `SPAWN_FAILED`); review do PM/Architect ganhou régua de engenharia ("o gate é o piso, não a barra")
  e postura de descoberta contínua com backlog dedicado (`.stateful-spec/backlog.md`).
- **stand-in-mcp-explorer — refatoração do `main.rs` (iteração 023):** o monólito de ~5.800 linhas foi
  dividido em um componente por arquivo com subpasta por região — `app/` (estado `StudioApp`, engine
  loop, reducers de eventos, render raiz, fixture de captura), `bars/` (topbar, tabbar,
  sidebar/{form,presets,saved_servers}), `tabs/{tools,resources,prompts,notifications,history}/` e
  `screens/` (settings, onboarding, error) — `main.rs` final com ~150 linhas. Refactor puro
  (comportamento idêntico): blocos `impl StudioApp` distribuídos por módulo, campos `pub(crate)`,
  testes co-locados; 294 testes preservados, gate completo verde.

### Planned

1. **Content types** — expandir `Content` enum com `ImageContent`, `AudioContent`, `EmbeddedResource`
2. **Tool annotations** — adicionar campo `annotations` (`ToolAnnotations`) em `ToolDefinition` (`title`, `destructiveHint`, `readOnlyHint`, `idempotentHint`, `openWorldHint`)
3. **Server capabilities** — expandir `ServerCapabilities` com `logging`, `completions`, `experimental`
4. **Notifications** — `notifications/tools/list_changed`, `notifications/resources/updated`
5. **Ping** — endpoint `ping` para health check
6. **Completions** — `completion/complete` para autocomplete de argumentos
7. **Logging** — `logging/setLevel`, `LoggingMessageNotification`
8. **Sampling & Roots** — `sampling/createMessage`, `roots/list`

## [stand-in-mcp-studio 0.1.0] — 2026-06-09

### Added

- **Native desktop MCP client** (MCP Explorer) — egui/eframe with **wgpu** backend, consuming the `stand-in-client` SDK
- **Bridge layer** — async `UiCommand`/`EngineEvent` bridge over `stand-in-client::Client`; headless-testable with a `RepaintHook` trait
- **Connection** — Stdio + Streamable HTTP via SDK; `connect`/`disconnect`/`refresh`; capability discovery
- **Sidebar** — transport selection, command/args/env fields, preset cards, guided toggle, privacy badge
- **Topbar** — connection dot (off/busy/on/err), capability pills, language dropdown, dark/light toggle, refresh
- **Tab bar** — Tools, Resources, Prompts, Notifications, History with animated underline and count chips
- **Tools tab** — list+search, parameter form from `InputSchema`, Run with spinner, code viewer with `isError` highlighting
- **Resources tab** — concrete + template resources, Read/Subscribe/Unsubscribe, content viewer (text + blob)
- **Prompts tab** — prompt list, argument form, message preview with role badges
- **Notifications tab** — live feed with level-colored rows, time stamp, sticky toolbar, Clear
- **History tab** — accordion request/response, side-by-side JSON `CodeView`, status badges (Success/Error/EngineError)
- **Settings** — themed panel (dark/light, density compact/regular/comfy, primary color swatch, language PT/EN/ES)
- **Persistence** — settings, presets, last connection saved/restored via `directories::ProjectDirs`; zero telemetry
- **i18n** — PT-BR, English, Espanol; fallback to PT-BR; 32+ translation keys
- **`stand-in-mcp-studio-ds`** — Design System crate (path dep, `publish = false`): `Tokens` 34-field, semantic OKLCH colors, `Density` 5-vars, Hanken Grotesk + JetBrains Mono fonts, 19 hand-drawn icons, 28+ components (`DsWidget`/`DsStatefulWidget`), motion timings, JSON tokenizer `CodeView`, `apply_theme`/`ui.ds_tokens()`
- **Visual gate** — `--capture` mode via `ViewportCommand::Screenshot` (wgpu); deterministic fixture-based screenshot for design fidelity regression
- **Anti-facade boundary** — `clippy.toml` `disallowed-methods`/`disallowed-types` banning raw egui widgets and raw colors outside the `ds` crate
- **CI binaries workflow** — `binaries.yml` builds release artifacts per OS (Linux/macOS/Windows) with `wgpu`/Vulkan system deps; artifacts uploaded, release gated to human
- **Quality** — full workspace gate green: fmt, clippy, test, build, doc; 390+ tests total

## [0.0.4] — 2026-04-25

### Added

- **`#[mcp_resource]` macro** — declare resources with typed parameters and URI templates
  - Detects concrete resources (fixed URI) vs template resources (URI with `{param}`)
  - Infers parameters from function signature; `{param}` in URI become function arguments
  - Generates `McpResource` trait implementation and registers via `inventory`
  - Return type `Result<String>` auto-wrapped as `TextResourceContents`, `Result<Vec<u8>>` as `BlobResourceContents` (base64-encoded)
  - Optional `name`, `description`, and `mime_type` attributes
- **`resources/list`** dispatch in `RequestHandler` — returns all concrete resources
- **`resources/templates/list`** dispatch — returns all template resources
- **`resources/read`** dispatch — exact URI match on concrete, template pattern match via `{param}` splitting
- **`resources/subscribe`** and **`resources/unsubscribe`** dispatch — subscribe tracking in `ResourceRegistry`
- **`ResourcesCapability`** advertised in `initialize` response (`ServerCapabilities`)
- **`ResourceRegistry`** — holds registered resources, dispatches read, manages subscriptions with `Arc<RwLock<>>`
- **`ResourceError`** variant added to `Error` enum
- **SSE notification wiring** — HTTP transport wires subscription senders after `resources/subscribe`
- Resource types: `Resource`, `ResourceTemplate`, `ResourceContents` (Text/Blob), `ResourceAnnotations`
- Resource types re-exported from `stand_in::prelude`
- `tokio sync` feature enabled for `RwLock` and `broadcast::Sender` in resource subsystem
- **`examples/resource_server.rs`** — demonstrates 2 concrete + 1 template resources over stdio
- **`examples/all_features.rs`** — single HTTP server demonstrating all macros (tools, prompts, resources) with full curl documentation

## [0.0.3] - 2026-03-14

### Added

- **`#[mcp_prompt]` macro** — declare reusable prompt templates with typed arguments
  - Infers argument list from function signature (`Option<T>` → optional)
  - Generates `McpPrompt` trait implementation and registers via `inventory`
  - Return type `Prompt` with `Prompt::user(text)` and `Prompt::assistant(text)` constructors
- **`prompts/list`** dispatch in `RequestHandler` — returns all registered prompts
- **`prompts/get`** dispatch in `RequestHandler` — executes a prompt by name with arguments
- **`PromptsCapability`** advertised in `initialize` response (`ServerCapabilities`)
- **`PromptRegistry`** — holds registered prompts, dispatches `get_prompt`
- **`PromptError`** variant added to `Error` enum
- `Prompt` and `PromptMessage` re-exported from `stand_in::prelude`

## [0.0.2] - 2026-03-14

### Added

- **Tracing instrumentation** across the HTTP transport execution path
  - `http_transport.rs`: `debug`/`info`/`warn` on POST/GET/DELETE handlers
  - `session_store.rs`: `info` for session create/remove, `debug` for validate
  - `handler.rs`: `info` for method dispatch, `error`/`warn` for failures
  - `sse.rs`: `trace` for SSE events, `debug` for lagged messages
  - Client disconnect detection via `StreamDropGuard` (logs when SSE stream closes)
- **ASCII startup banner** — block-letter "STAND-IN" with dynamic version and bind address, printed on HTTP server start
- `tracing-subscriber` as dev-dependency with `EnvFilter` in `examples/http_server.rs`

## [0.0.1] - 2026-03-13

### Added

- **Streamable HTTP transport** (feature: `http`) — MCP 2025-03-26 spec
  - `HttpTransport` struct with `POST/GET/DELETE /mcp` handlers
  - Session management via `Mcp-Session-Id` header (`Session`, `SessionStore`)
  - SSE notification stream on `GET /mcp`
  - CORS support via `tower-http`
  - Graceful shutdown on Ctrl+C
  - `#[mcp_server(host = "...", port = N)]` macro attributes for HTTP config
  - `serve_http()` convenience method (feature-gated)
  - `examples/http_server.rs` — minimal HTTP server example
  - 10 HTTP integration tests (full lifecycle, error cases)
- Cargo workspace with two crates: `stand-in` (library) and `stand-in-macros` (proc macros)
- Stub macros: `#[mcp_server]`, `#[mcp_tool]`, `#[mcp_resource]`, `#[mcp_prompt]`
- Custom error types with `thiserror` (`Error` enum, `Result` alias)
- Prelude module (`use stand_in::prelude::*`)
- Feature flags: `stdio` (default), `http` (optional Streamable HTTP transport)
- GitHub Actions CI workflow (build, test, lint on Linux/macOS/Windows)
- GitHub Actions publish workflow (crates.io on version tags)
- MIT LICENSE file
- Design Source methodology in `impl/`
