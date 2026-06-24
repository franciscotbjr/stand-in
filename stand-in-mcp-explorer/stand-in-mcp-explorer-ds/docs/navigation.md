# navigation — SectionLabel, CapChip, Topbar, Tabbar, SidebarShell

Este arquivo documenta o lado Rust/GPUI dos componentes do grupo `navigation`. As regras
visuais vinculantes moram nos `.prompt.md` do canon, em
[`components/navigation/`](../../../stand-in-client-prototipo/components/navigation/).

![Grupo navigation em contexto de app — SidebarShell, SectionLabel, PresetCard, Topbar e Tabbar](../../../stand-in-client-prototipo/screenshots/reference/components/navigation-dark.png)

## SectionLabel

Fonte: [`src/navigation/section_label.rs`](../src/navigation/section_label.rs) · Canon:
[`SectionLabel.prompt.md`](../../../stand-in-client-prototipo/components/navigation/SectionLabel.prompt.md)

Rótulo de seção de 11 px em maiúsculas, com ícone opcional de 13 px. A regra de
apresentação é do componente: o caller escreve em caixa normal ("Servidores salvos") e o
render aplica `to_uppercase()` — nunca exigir maiúsculas do caller, espelhando o
`text-transform` do CSS. API: `SectionLabel::new(text)`, `.icon(IconName)`, `.id(id)`.
Limite de 1 a 3 palavras; dentro de um `Panel`, o título do Panel cumpre esse papel —
nunca duplicar o cabeçalho. Nota de fidelidade: o GPUI pinado não expõe letter-spacing,
então o tracking largo do canon (0.08em) fica documentado no rustdoc à espera de API.

```rust
SectionLabel::new("Servidores salvos").icon(IconName::Bolt) // render aplica UPPERCASE
```

## CapChip

Fonte: [`src/navigation/cap_chip.rs`](../src/navigation/cap_chip.rs) · Canon:
[`CapChip.prompt.md`](../../../stand-in-client-prototipo/components/navigation/CapChip.prompt.md)

Chip mono pequeno que resume capacidade ou contagem na Topbar (slot direito, o `.caps`
do canon). Sempre mono; o número é negrito na cor de texto plena e o rótulo fica em
`text-2`. **Nunca é interativo** — para uma ação ao lado do chip, o padrão é um `Button`
ghost sm. API: `CapChip::new(label)`, `.count(n)`, `.icon(IconName)` (12 px), `.id(id)`.

```rust
CapChip::new("tools").count(6).icon(IconName::Tool) // nunca interativo
```

## Topbar

Fonte: [`src/navigation/topbar.rs`](../src/navigation/topbar.rs) · Canon:
[`Topbar.prompt.md`](../../../stand-in-client-prototipo/components/navigation/Topbar.prompt.md)

Barra horizontal fixa de 60 px: à esquerda o estado de conexão (um `StatusDot` + coluna
com o título mono e a linha de metadados), à direita o conteúdo de ação. O título é o
**contexto de conexão** (o servidor conectado), nunca o nome do app; os metadados são
unidos por " · " (middot). A Topbar e a Tabbar são irmãs — abas nunca moram dentro da
Topbar (fronteira estrutural do canon).

API: `Topbar::new(dot_state, title, meta)`,
`.right_children(impl IntoIterator<Item = AnyElement>)` (tipicamente `CapChip`s e
`Button`s ghost, em um h_flex de gap 7 alinhado à direita) e `.id(id)`.

```rust
Topbar::new(DotState::On, "server-filesystem", "STDIO · v2026.4.1 · 57ms")
    .right_children([
        CapChip::new("tools").count(6).icon(IconName::Tool).into_any_element(),
        Button::new("Modo guiado").variant(ButtonVariant::Ghost).sm().into_any_element(),
    ])
```

## Tabbar

Fonte: [`src/navigation/tabbar.rs`](../src/navigation/tabbar.rs) · Canon:
[`Tabbar.prompt.md`](../../../stand-in-client-prototipo/components/navigation/Tabbar.prompt.md)

Barra de abas de 46 px com sublinhado estático de 2 px em OBY na aba ativa (o canon não
anima o sublinhado — o orçamento de movimento é preservado) e contadores mono que somem
em zero. Regras do canon: de 3 a 6 abas fixas, sem abas fecháveis nem roláveis; ícones de
15 px opcionais, mas todos-ou-nenhum no conjunto (o struct documenta, o caller garante).

API em dois tipos:

- `TabItem::new(id, label)` + `.count(n)` + `.icon(IconName)` — a descrição de cada aba.
- `Tabbar::new(id, items, active_ix)` + `.handlers(Vec<Option<ClickHandler>>)` — o caller
  é dono do índice ativo e cria um handler por aba via `cx.listener` (o mesmo padrão do
  `SegmentedControl`); `None` rende uma aba não-interativa (modo captura). Handlers
  faltantes são preenchidos com `None` — uma `Tabbar` sem handlers renderiza, mas não
  navega (a armadilha "zip-trap" da 025: zipar com um vetor vazio descartaria as abas).

```rust
let items = vec![
    TabItem::new("tools", "Tools").icon(IconName::Tool).count(6),
    TabItem::new("history", "Histórico").icon(IconName::History),
];
Tabbar::new("main-tabs", items, active_ix)
    .handlers(handlers) // Vec<Option<ClickHandler>>, um por aba
```

## SidebarShell

Fonte: [`src/navigation/sidebar_shell.rs`](../src/navigation/sidebar_shell.rs) · Canon:
[`SidebarShell.prompt.md`](../../../stand-in-client-prototipo/components/navigation/SidebarShell.prompt.md)

A casca genérica da barra lateral, em três zonas: brand fixo no topo, corpo rolável no
meio e rodapé fixo opcional. A regra central do canon: é uma **casca** — o DS fornece a
estrutura e o projeto consumidor decide o conteúdo (não existe "Sidebar do MCP Explorer"
no DS). Duas consequências práticas:

- **A casca não tem largura própria.** O caller ou o grid do app a define — tipicamente
  `Density::sidebar_w()` (304 px regular, 280 px compact).
- **As seções do corpo abrem com `SectionLabel`** — convenção documentada, não imposta.

API: `SidebarShell::new()`, `.brand_mark(el)` (o elemento dentro do quadrado 34×34 —
tipicamente um `Icon` de 18 px; o slot herda `on-primary` como cor), `.brand_name(s)`
(fs 15, peso 700), `.brand_sub(s)`, `.children(iter)` (o corpo rolável: `flex_1` +
`min_h(0)` + scrollbar, padding pela densidade), `.footer(el)` (callouts persistentes,
nunca navegação) e `.id(id)`.

O brand-mark carrega **o primeiro dos dois gradientes legítimos** do DS (proibição 5) e o
anel interno `BRAND_RING` (branco a 8%, fixo nos dois modos). Delta documentado contra o
canon: o gradiente canônico tem três paradas (oby → genipina 60% → yandi), mas o gpui
pinado aceita duas — a realização usa oby → yandi, com a parada intermediária perdida
registrada como lacuna menor de fidelidade no rustdoc.

```rust
SidebarShell::new()
    .brand_mark(Icon::new(IconName::Leaf).with_px(px(18.)))
    .brand_name("MCP Explorer")
    .brand_sub("MCP · local-first")
    .children([/* SectionLabel + conteúdo da seção */])
    .footer(privacy_callout) // callouts persistentes, nunca navegação
```
