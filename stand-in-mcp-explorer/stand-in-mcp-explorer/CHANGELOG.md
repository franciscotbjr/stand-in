# Changelog

All notable changes to `stand-in-mcp-explorer` (the MCP Explorer app).

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added (037 — Instrumentação de performance das listas)

Harness de medição de performance das listas para diagnosticar a "rolagem dura"
com milhares de itens. O agente instrumenta; o usuário roda e rola; o agente
analisa o JSONL e gera o backlog. Ver
[`docs/perf-measurement.md`](docs/perf-measurement.md).

Tudo fica atrás da feature **dev-only `perf`** (off por default): num build normal
o módulo `perf`, os hooks, os fixtures sintéticos e a dep `sysinfo` ficam **fora**
do binário (zero overhead). Build de medição: `--features perf`.

- `app/perf` — instrumentação gated (`--capture <region> perf` ou env `MCPX_PERF`,
  dentro de um build `--features perf`):
  evento `frame` (delta entre frames/fluidez, itens visíveis/não-renderizados/
  total, build-time total e por item, fase init/scroll) + evento `sample` (CPU%/
  RSS do processo via `sysinfo`, thread dedicada). Saída JSONL no caminho impresso
  no startup (`PERF LOG: <path>`), sobrescritível por `MCPX_PERF_LOG`.
- Instrumentação dos closures de windowing — Tools (`uniform_list`) e History
  (`gpui::list`). No-op quando perf está desligado (um load atômico + branch).
- Fixtures sintéticos determinísticos: `--capture tools perf` (N `ToolDefinition`)
  e `--capture history perf` (N pares request↔response). N configurável por
  `MCPX_PERF_N` (default 5000).
- `sysinfo` 0.33 como dependência **opcional**, habilitada só pela feature `perf`.

### Added (036 — Painel flutuante de Variáveis de ambiente STDIO)

Painel flutuante para gerenciar variáveis de ambiente no transporte STDIO,
sem esticar a sidebar. A lista inline de pares chave/valor vira um gatilho
recolhido (ícone + contagem + chevron) que abre um painel flutuante.

- M1: `CountPill` no DS (pill numérico redondo, bg OBY, fg branco, mono).
- M1: `env_trigger` na sidebar (Bolt icon + label + CountPill/summary + Chevron,
  STDIO-only, hover border_2, cursor pointer).
- M1: `env_panel` (overlay flutuante à direita, espelhando auth_panel; corpo
  reutiliza `render_env_rows`; header com título/subtítulo + X; footer com
  Salvar; click-outside fecha via `ClickOutsideHandler`; `.occlude()` no card).
- M1: gatilho `on_open_env` (STDIO only) + handlers open/close-x/close-save/
  close-outside + `env_panel_open` state + fixture de captura (`--capture env`).
- M1: i18n 5 chaves novas (envSubtitle, envNone, envCount, envEmpty, envDone)
  nos 3 idiomas.

### Fixed (036 — QA pós-DONE)

- Gatilho de env: a contagem aparecia **duplicada** (pill + texto "N variável(is)").
  Mantida só no **bullet** (pill); o texto redundante foi removido. Vazio segue
  mostrando "nenhuma".

### Changed (036 — addendum)

- Seletor de transporte: **HTTP** passa a ser o primeiro segmento e o transporte
  **padrão** (era STDIO). `Transport::selected_ix` (HTTP=0/STDIO=1), ordem dos
  segmentos e handlers, e o default do form ajustados. A captura `--capture env`
  força STDIO para o painel de env continuar renderizando.

### Added (035 — Autenticação e Autorização para conexões)

Suporte completo a autenticação e autorização no SDK `stand-in-client` + painel
de Autorização no MCP Explorer. 5 marcos (M1–M5), perfis `lib` (M1–M2) + `fe`
(M3–M5).

- M1: credencial por conexão (`NoAuth`/`Basic`/`Bearer`) + injeção do header
  `Authorization` no `HttpTransport` (POST/GET-SSE/DELETE); API `with_credential`;
  testes contra servidor-gravador axum.
- M2: motor OAuth 2.0 Authorization Code + PKCE S256 + listener loopback +
  troca/refresh; feature flag `oauth`; testes contra AS falso.
- M3: painel flutuante de Autorização (4 métodos, campos condicionais,
  `Field::secret()` com toggle de máscara, Redirect URI read-only + copiar,
  estado por-servidor, reset-no-preset, inativo em STDIO).
- M4: fiação ponta-a-ponta — abridor de navegador (`open::that`) + keychain
  (`keyring`) + `servers.json` + credencial no Connect + botão "Autorizar"
  fiado + refresh de token expirado.
- M5: barra de navegação (tabbar) reposicionada abaixo dos elementos de
  transporte na sidebar (§1.1) + auditoria de i18n (30 chaves `auth.*`,
  0 hardcoded, 0 morta) + CHANGELOG.

### Added (028 — Rebuild on the 025 Design System)

Complete rebuild of the MCP Explorer app composing the 025 Design System
(`stand-in-mcp-explorer-ds`). 17 milestones (M1–M17) on a dedicated feature branch.

**Foundations (M1–M4):**

- App scaffold: GPUI window + dark/light theme + `DsAssets` + CLI `--capture <region> <state> <mode>` + 023 file structure.
- O-006: light `text-3` contrast fixed (`#5D7F96` → `#56758A`, ≥4.5:1 AA) in the design canon + DS.
- Tokio ↔ SDK bridge: dedicated tokio thread with `Client` + 2 channels (`UiCommand`/`EngineEvent`) + pure `ConnState` reducer; zero `tokio::*` on the GPUI executor. Integration tests against `stand-in-reference` (connect, list, transport error).
- Persistence (`servers.json` via `ProjectDirs`, `ServerEntry` dedup by config) + i18n (`tr(key, lang)` mandatory `Lang`, PtBr/En/Es, ~115 keys, anti-footgun BUG-7).

**Regions (M5–M15):**

- Sidebar: connection form with real `InputState` fields, transport segmented control (Stdio/SSE/HTTP), env rows, presets, saved servers (collapse when empty), privacy badge. Composes the DS.
- Sidebar (live): Connect dispatches the typed config over all 3 transports; auto-save on `Connected` + dedup; pick-preset synchronises `InputState`; `const PRESETS` only under `--capture` (no-mock).
- Topbar: connection dot (4 states), name/meta, capability counts + latency, language switch (live, proves i18n), guided toggle, reconnect.
- Tabbar + Work-area: 5 tabs (icon + count, underline active, routing), split/single-column/disconnected modes, §5b flex-col chain (`flex_1`+`min_h(0)`+`overflow`) at every leaf (BUG-11).
- Tools: live search, `InputSchema`-to-`Field` params parser (pure + tested), Run with typed args + validation, result panel across 3 error planes (isError-data, protocol, transport). Integration test of the run loop.
- Resources: concrete + template lists, content (`classify_content` pure — text/JSON/binary-warning), metadata grid, subscribe/unsubscribe. Integration test of read and subscribe.
- Prompts: list + search, args form, `get_prompt` via bridge, message preview with role badge. Integration test of `get_prompt`.
- Notifications: full-width feed (newest-first) driven by real bridge events, 5-level filter + count + Clear, 5 distinct colours (warn ≠ err), §5b scroll.
- History: accordion of real bridge calls (newest-first), click-expand request/response `JsonView` side by side (`min_w(0)`).
- Settings: elevated overlay with shadow (`shadow_lg`, BUG-3 — the only shadow in the app), real theme/density/primary/guided controls, persisted.

**Consolidation (M16–M17):**

- Transversal pass (3rd gate): drove all 10 regions (click + type), BUG-1..11 regression checklist, long-text PT/ES, dark/light + density across all screens.
- App `README` + `CHANGELOG` + [design-system audit](docs/audit-8-prohibitions.md) (8 prohibitions, all PASS).

### Changed

- App rebuilt from zero on the 025 Design System. The previous app (020 + 022/023 fixes) is superseded.

### Fixed

- O-006: light `text-3` contrast ≥4.5:1 AA (canon + DS + test).
- O-009: headless `TestAppContext` bridge interaction test harness (`tests/app_interaction.rs`, 6 tests).
