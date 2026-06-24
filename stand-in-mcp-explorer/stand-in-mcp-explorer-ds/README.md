# stand-in-mcp-explorer-ds

Design System nativo do MCP Explorer, construído em Rust sobre **GPUI** e
**gpui-component**. Esta crate é a implementação canônica do Design System descrito em
[`stand-in-client-prototipo/`](../../stand-in-client-prototipo/readme.md): 23 componentes do
catálogo do protótipo mais uma extensão formal (o `Select`), todos rastreados 1:1 a uma
entrada do canon. A crate entrega apenas a camada visual — o aplicativo MCP Explorer que a
consome é reconstruído em uma iteração própria sobre esta base.

A crate é `publish = false`: o consumo é interno ao repositório, pelo app e pela gallery.

## O canon

A fonte vinculante de design é a pasta [`stand-in-client-prototipo/`](../../stand-in-client-prototipo/):

| Artefato | O que define |
|----------|--------------|
| [`tokens/*.css`](../../stand-in-client-prototipo/tokens/) | Os valores canônicos — cores (OKLCH), espaçamento, tipografia, fontes |
| `components/<grupo>/<Nome>.prompt.md` | As regras vinculantes de cada componente (anatomia, estados, semântica fixa) |
| [`readme.md`](../../stand-in-client-prototipo/readme.md) | Os fundamentos de conteúdo e visuais |
| [`SKILL.md`](../../stand-in-client-prototipo/SKILL.md) | O modo de obediência — as 8 proibições absolutas |

As **8 proibições**, como esta crate as realiza:

1. **Nunca inventar cores** — todo componente lê tokens via `cx.theme()` e `JandiExt`;
   o único arquivo com literais de cor é [`src/theme/palette.rs`](src/theme/palette.rs),
   transcrito de `tokens/colors.css`.
2. **Só duas fontes** — Hanken Grotesk (sans) e JetBrains Mono (mono), servidas por
   `DsAssets`.
3. **Só o catálogo de 22 ícones** — o enum `IconName` é fechado por construção; não há
   biblioteca de ícones nem emoji.
4. **Sem sombra em cards in-flow** — separação por borda de 1px e degrau de superfície;
   a sombra existe apenas em overlays (o popup do `Select` é o único caso na crate).
5. **Sem gradientes decorativos** — os dois gradientes legítimos são o brand-mark
   (`SidebarShell`) e o glifo do `EmptyState`.
6. **Roteamento mono/sans** — identificadores, valores técnicos, JSON e contadores em
   mono; prosa humana em sans.
7. **Valores dirigidos por densidade** — `pad`, `row-h`, `gap`, `fs` e `radius` vêm de
   `Density`; raios por papel são fixos.
8. **Reutilizar antes de criar** — todo componente rastreia a uma entrada do canon; a
   extensão `Select` ganhou a sua própria entrada
   ([`Select.prompt.md`](../../stand-in-client-prototipo/components/forms/Select.prompt.md)).

## Estrutura

O `src/` espelha a taxonomia de `components/` do canon:

```
src/
├── theme/        tokens → ThemeColor (dark+light), densidade, tipografia, paleta
├── core/         Icon, Button, IconButton, Badge, CopyButton, Spinner, StatusDot, ToggleLink
├── forms/        Field, KeyValueRow, SegmentedControl, Select
├── navigation/   SectionLabel, CapChip, Topbar, Tabbar, SidebarShell
├── data/         Panel, ListItem, ListSearch, PresetCard, LogRow, EmptyState, HintBar, JsonView
└── assets/       os 22 SVGs do catálogo + a anatomia do Spinner (servidos por DsAssets)
```

O inventário rastreado — 24 entradas (23 canônicas + a extensão `Select`), realizadas em
25 structs públicas, porque o card "Copy / link" do canon rende duas (`CopyButton` e
`ToggleLink`):

| Componente | Grupo | Documentação | Canon |
|------------|-------|--------------|-------|
| `Icon` | core | [docs/core.md](docs/core.md) | [Icon.prompt.md](../../stand-in-client-prototipo/components/core/Icon.prompt.md) |
| `Button` | core | [docs/core.md](docs/core.md) | [Button.prompt.md](../../stand-in-client-prototipo/components/core/Button.prompt.md) |
| `IconButton` | core | [docs/core.md](docs/core.md) | [IconButton.prompt.md](../../stand-in-client-prototipo/components/core/IconButton.prompt.md) |
| `Badge` | core | [docs/core.md](docs/core.md) | [Badge.prompt.md](../../stand-in-client-prototipo/components/core/Badge.prompt.md) |
| `CopyButton` | core | [docs/core.md](docs/core.md) | [CopyButton.prompt.md](../../stand-in-client-prototipo/components/core/CopyButton.prompt.md) |
| `Spinner` | core | [docs/core.md](docs/core.md) | [Spinner.prompt.md](../../stand-in-client-prototipo/components/core/Spinner.prompt.md) |
| `StatusDot` | core | [docs/core.md](docs/core.md) | [StatusDot.prompt.md](../../stand-in-client-prototipo/components/core/StatusDot.prompt.md) |
| `ToggleLink` | core | [docs/core.md](docs/core.md) | [`core.css`](../../stand-in-client-prototipo/components/core/core.css) (classe `.toggle-link`; sem `.prompt.md` próprio) |
| `Field` | forms | [docs/forms.md](docs/forms.md) | [Field.prompt.md](../../stand-in-client-prototipo/components/forms/Field.prompt.md) |
| `KeyValueRow` | forms | [docs/forms.md](docs/forms.md) | [KeyValueRow.prompt.md](../../stand-in-client-prototipo/components/forms/KeyValueRow.prompt.md) |
| `SegmentedControl` | forms | [docs/forms.md](docs/forms.md) | [SegmentedControl.prompt.md](../../stand-in-client-prototipo/components/forms/SegmentedControl.prompt.md) |
| `Select` | forms | [docs/forms.md](docs/forms.md) | [Select.prompt.md](../../stand-in-client-prototipo/components/forms/Select.prompt.md) |
| `SectionLabel` | navigation | [docs/navigation.md](docs/navigation.md) | [SectionLabel.prompt.md](../../stand-in-client-prototipo/components/navigation/SectionLabel.prompt.md) |
| `CapChip` | navigation | [docs/navigation.md](docs/navigation.md) | [CapChip.prompt.md](../../stand-in-client-prototipo/components/navigation/CapChip.prompt.md) |
| `Topbar` | navigation | [docs/navigation.md](docs/navigation.md) | [Topbar.prompt.md](../../stand-in-client-prototipo/components/navigation/Topbar.prompt.md) |
| `Tabbar` | navigation | [docs/navigation.md](docs/navigation.md) | [Tabbar.prompt.md](../../stand-in-client-prototipo/components/navigation/Tabbar.prompt.md) |
| `SidebarShell` | navigation | [docs/navigation.md](docs/navigation.md) | [SidebarShell.prompt.md](../../stand-in-client-prototipo/components/navigation/SidebarShell.prompt.md) |
| `Panel` | data | [docs/data.md](docs/data.md) | [Panel.prompt.md](../../stand-in-client-prototipo/components/data/Panel.prompt.md) |
| `ListItem` | data | [docs/data.md](docs/data.md) | [ListItem.prompt.md](../../stand-in-client-prototipo/components/data/ListItem.prompt.md) |
| `ListSearch` | data | [docs/data.md](docs/data.md) | [ListSearch.prompt.md](../../stand-in-client-prototipo/components/data/ListSearch.prompt.md) |
| `PresetCard` | data | [docs/data.md](docs/data.md) | [PresetCard.prompt.md](../../stand-in-client-prototipo/components/data/PresetCard.prompt.md) |
| `LogRow` | data | [docs/data.md](docs/data.md) | [LogRow.prompt.md](../../stand-in-client-prototipo/components/data/LogRow.prompt.md) |
| `EmptyState` | data | [docs/data.md](docs/data.md) | [EmptyState.prompt.md](../../stand-in-client-prototipo/components/data/EmptyState.prompt.md) |
| `HintBar` | data | [docs/data.md](docs/data.md) | [HintBar.prompt.md](../../stand-in-client-prototipo/components/data/HintBar.prompt.md) |
| `JsonView` | data | [docs/data.md](docs/data.md) | [JsonView.prompt.md](../../stand-in-client-prototipo/components/data/JsonView.prompt.md) |

A camada de tema (paleta, cores, densidade, tipografia, assets) está documentada em
[docs/theme.md](docs/theme.md).

> **Regra anti-deriva:** os arquivos em `docs/` documentam o **lado Rust/GPUI** de cada
> componente — a API real, os builders, o comportamento de densidade e tema. As regras
> visuais vinculantes (anatomia, estados, semântica fixa) moram **só** nos `.prompt.md`
> do canon; os docs apontam para elas, nunca as duplicam.

## Tema

A paleta jandi (a rampa de 8 degraus, de `suco` a `guerra`) é mapeada para o
`ThemeColor` do gpui-component em dois modos (dark e light), com o degrau primário
(`jandi`) idêntico nos dois. O que o `ThemeColor` padrão não carrega — `surface-3`,
`border-2`, os estados dim, o fundo de código, os tokens de JSON e a cor de sombra de
overlay — vive em `JandiExt`, um global acessado por `cx.global::<JandiExt>()`. A
densidade tem três níveis (compact, regular, comfy) e governa cinco variáveis; os raios
por papel (input, chip, badge, botão, card, pílula) são fixos. O detalhe completo, com as
tabelas de valores e as derivações documentadas de token, está em
[docs/theme.md](docs/theme.md).

## Uso

O bootstrap acontece uma vez, na inicialização do app, antes de construir qualquer
componente:

```rust
// 1. Inicializa o gpui-component (widgets, overlays) e a densidade padrão.
stand_in_mcp_explorer_ds::init(cx);

// 2. Instala o tema jandi no modo desejado (e, opcionalmente, a densidade).
stand_in_mcp_explorer_ds::theme::apply_theme(ThemeMode::Dark, cx);
// ou: theme::apply_theme_and_density(ThemeMode::Dark, Density::Compact, cx);
```

A composição usa o prelude para os itens essenciais do gpui e importa os componentes
pelos seus módulos:

```rust
use stand_in_mcp_explorer_ds::prelude::*;
use stand_in_mcp_explorer_ds::core::{Button, ButtonVariant, IconName};
use stand_in_mcp_explorer_ds::data::Panel;

Panel::new()
    .title("Parâmetros")
    .children([Button::new("Conectar")
        .variant(ButtonVariant::Primary)
        .icon(IconName::Play)
        .id("connect-btn")
        .into_any_element()]);
```

O app que consome a crate também deve instalar o `DsAssets` como `AssetSource` da
aplicação — é ele que serve os 22 SVGs do catálogo e delega as fontes ao
gpui-component (ver [docs/theme.md](docs/theme.md)).

## Gallery (Storybook nativo)

A crate irmã [`stand-in-mcp-explorer-gallery`](../stand-in-mcp-explorer-gallery/) é o
Storybook do DS: 10 seções interativas (Foundations, Icon, Indicators, Actions, Badges,
Forms, Forms Advanced, Select, Navigation, Data) com alternância de modo dark/light e de
densidade na toolbar.

```bash
# Navegação interativa (da raiz do repositório):
cargo run --manifest-path stand-in-mcp-explorer/Cargo.toml -p stand-in-mcp-explorer-gallery

# Modo determinístico para captura: [--capture] <seção> <estado> <modo>
cargo run --manifest-path stand-in-mcp-explorer/Cargo.toml -p stand-in-mcp-explorer-gallery -- --capture core overview dark
```

Os scripts de processo `smoke-open.ps1` e `capture-os.ps1` (em `.stateful-spec/scripts/`)
usam esse contrato de linha de comando.

## Testes e gate de qualidade

A crate carrega três camadas de verificação:

- **Testes unitários co-locados** (`#[cfg(test)]` em cada componente): construção,
  builders, mapeamentos de cor por variante, constantes de geometria e os invariantes do
  canon (catálogo de 22, os cinco `BadgeKind` distintos, dims a 16% de alfa, contraste
  WCAG da paleta).
- **Testes de geometria headless** ([`tests/geometry.rs`](tests/geometry.rs)): renderizam
  componentes reais via `TestAppContext` e medem `Bounds` — os asserts diferenciais de
  linha pegam a classe de regressão em que um contêiner volta a `display:block` e os
  filhos empilham em vez de alargar. Rodam sem display:
  `cargo test --manifest-path stand-in-mcp-explorer/Cargo.toml -p stand-in-mcp-explorer-ds --test geometry`.
- **Lint sintático de flex** (`.stateful-spec/scripts/lint-flex.ps1`): acusa `div()` puro
  com propriedades de flex inertes (a família FE-G16).

O gate completo (fmt, clippy, test, build, doc) roda **da raiz do repositório** — o
`rust-toolchain.toml` na raiz pina a toolchain, e o rustup resolve o pin pelo diretório
corrente, não pelo `--manifest-path`.

## Pinagem de revisões

A crate é `edition = "2024"` e declara `rust-version = "1.95.0"` (a mesma toolchain que o
`rust-toolchain.toml` da raiz pina). Todo o código GPUI alveja revisões exatas; a paleta
vem de uma dependência semver do crates.io:

| Dependência | Versão | Revisão | Mecanismo de pin |
|-------------|--------|---------|------------------|
| `gpui` (Zed) | 0.2.2 | `3f5705b9` | `Cargo.lock` commitado (pin canônico) |
| `gpui-component` | 0.5.2 | `70d2c44b` | `rev` no manifesto |
| `jandi-colors` | 0.1.0 | — | semver do crates.io (`no_std`, sem features, zero transitivas) |

O `gpui` é pinado **só** pelo lock, de propósito: declarar o `rev` no manifesto criaria
um diamante com a dependência não-pinada que o próprio `gpui-component` tem do `gpui`
(o Cargo trata `git+url?rev=X` e `git+url` como fontes distintas, mesmo no mesmo commit).
A explicação completa está no comentário do
[`Cargo.toml` do workspace](../Cargo.toml). Qualquer `cargo update` deve ser deliberado e
auditado. Exemplos de API encontrados na web frequentemente misturam revisões — em caso
de dúvida, a fonte da verdade é o código do pin, nunca um exemplo externo. A `jandi-colors`
é a fonte da rampa de 8 degraus (ver [docs/theme.md](docs/theme.md)); um teste fixa os 8
hexes canônicos para travar a paleta contra um `cargo update` silencioso.
