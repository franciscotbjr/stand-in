# core — Icon, Button, IconButton, Badge, CopyButton, Spinner, StatusDot, ToggleLink

Este arquivo documenta o lado Rust/GPUI dos componentes do grupo `core`. As regras
visuais vinculantes (anatomia, estados, semântica fixa) moram nos `.prompt.md` do canon,
em [`components/core/`](../../../stand-in-client-prototipo/components/core/) — cada seção
aponta para a sua entrada.

![Grupo core em contexto de app — Button, IconButton, Badge, StatusDot, CopyButton e ToggleLink](../../../stand-in-client-prototipo/screenshots/reference/components/core-dark.png)

## Icon

Fonte: [`src/core/icon.rs`](../src/core/icon.rs) · Canon:
[`Icon.prompt.md`](../../../stand-in-client-prototipo/components/core/Icon.prompt.md)

O catálogo fechado de 22 glifos de traço (stroke 2, viewBox 24×24, pontas redondas). O
enum `IconName` torna a proibição 3 estrutural: não existe construtor para um glifo fora
do catálogo. Os SVGs são embarcados em tempo de compilação e servidos pelo `DsAssets`
sob a chave `icons/<nome>.svg`.

Catálogo (`IconName::ALL`, na ordem canônica): `plug`, `tool`, `doc`, `chat`, `bell`,
`history`, `play`, `plus`, `x`, `search`, `copy`, `check`, `lock`, `bolt`, `refresh`,
`chevron`, `sub`, `info`, `leaf`, `eye`, `file`, `globe`.

| Builder | Efeito |
|---------|--------|
| `Icon::new(name)` | Cria o ícone no tamanho padrão (15 px), cor herdada do contexto |
| `.size(IconSize)` | Tamanho canônico: `Xs` 12 (badges/chips), `Sm` 14 (panel-heads/IconButton), `Md` 15 (padrão), `Lg` 28 (EmptyState) |
| `.with_px(px)` | Tamanho livre (os canônicos preferem `IconSize`) |
| `.color(hsla)` | Cor explícita — só quando o pai não puder definir via `text_color` |
| `.rotate(Transformation)` | Rotação (o chevron-para-baixo canônico = `rotate(percentage(0.25))`) |

**Resolução de cor (idioma obrigatório):** o `svg()` do gpui só pinta quando a cor está
no próprio elemento — a cascata de `text_color` do pai **não** alcança o svg. O `Icon`
resolve a cor no render: o override explícito, senão `window.text_style().color` (a cor
contextual empurrada pelo ancestral). Sem essa resolução, todo ícone renderiza invisível
— a regressão foi pega pela conferência visual humana da 025.

```rust
// herda a cor do contexto — o pai empurra via text_color
div().text_color(cx.theme().foreground).child(Icon::new(IconName::Play));
// tamanho canônico (Sm = 14px, panel-head / IconButton)
Icon::new(IconName::Doc).size(IconSize::Sm);
```

## Button

Fonte: [`src/core/button.rs`](../src/core/button.rs) · Canon:
[`Button.prompt.md`](../../../stand-in-client-prototipo/components/core/Button.prompt.md)

O controle primário de ação, em três variantes e dois tamanhos. A semântica é fixa:
`Primary` no máximo uma vez por tela; `Danger` apenas para ações destrutivas reais;
`Ghost` para todo o resto. Não há variante "link" — esse papel é do `ToggleLink`.

| Builder | Efeito |
|---------|--------|
| `Button::new(label)` | Cria com `Primary` e `Md` por padrão |
| `.variant(ButtonVariant)` | `Primary` · `Ghost` · `Danger` |
| `.size(ButtonSize)` / `.sm()` | `Md` (40 px, fs 13, raio 9) ou `Sm` (32 px, fs 12, raio 8) |
| `.block()` | Largura total |
| `.disabled()` | Opacidade 0,5 e clique ignorado |
| `.loading()` | Substitui o ícone pelo `Spinner` e desabilita |
| `.icon(IconName)` | Ícone antes do rótulo (15 px no Md, 12 px no Sm) |
| `.id(id)` | Element id (necessário para interatividade) |
| `.on_click(handler)` | Handler de clique; o caller liga `cx.listener`/`cx.notify` quando precisa notificar a view |

Mapeamento de cor por variante (tokens do tema, nunca literais): `Primary` usa
`button_primary`/`button_primary_foreground`; `Ghost` usa `secondary` com borda `border`
(hover degrau para `surface_3`/`border_2`); `Danger` usa o fundo dim (`err_dim` do
`JandiExt`) com texto `danger`, e o hover inverte para fundo `danger` com texto claro. O
estado ativo desloca 1 px para baixo — embutido no componente, o caller nunca o adiciona.

```rust
Button::new("Conectar")
    .variant(ButtonVariant::Primary)
    .icon(IconName::Play)
    .id("btn-connect")
    .on_click(cx.listener(|_, _: &ClickEvent, _, cx| cx.notify()))
```

## IconButton

Fonte: [`src/core/icon_button.rs`](../src/core/icon_button.rs) · Canon:
[`IconButton.prompt.md`](../../../stand-in-client-prototipo/components/core/IconButton.prompt.md)

Botão quadrado de 32×32 só de ícone (14 px), para ações secundárias compactas. Nunca é a
ação primária de uma tela — esse papel é do `Button::Primary`. O rótulo é obrigatório no
construtor (`IconButton::new(icon, label)`) por acessibilidade; fica armazenado para o
futuro suporte de tooltip. O hover muda apenas a cor do ícone e a borda — sem salto de
fundo. Builders: `.id(id)`, `.on_click(handler)` e `.on_click_boxed(handler)` (este
último recebe um `ClickHandler` já em caixa, usado por componentes que encaminham
handlers, como o `KeyValueRow`).

```rust
IconButton::new(IconName::Refresh, "Reconectar") // rótulo obrigatório (a11y)
    .id("btn-refresh")
    .on_click(cx.listener(|_, _: &ClickEvent, _, cx| cx.notify()))
```

## Badge

Fonte: [`src/core/badge.rs`](../src/core/badge.rs) · Canon:
[`Badge.prompt.md`](../../../stand-in-client-prototipo/components/core/Badge.prompt.md)

Selo mono pequeno (fs 10,5, raio 6, padding 2×7) que codifica a natureza técnica de um
item. Os cinco `BadgeKind` são fixos — nunca reinterpretar nem criar novos:

| Kind | Fundo | Texto | Uso |
|------|-------|-------|-----|
| `Read` | `ok_dim` | `ok` | Operações de leitura, estados seguros |
| `Write` | `warn_dim` | `warn` | Operações de escrita, atenção |
| `Mime` | `surface_3` | `text-2` | Tipos de conteúdo |
| `Muted` | `surface-2` | `text-3` | Metadados sem carga |
| `Role` | OBY a 18% | OBY | Papéis e categorias |

API: `Badge::new(label, kind)`, `.icon(IconName)` (12 px, opcional), `.id(id)`. O texto
deve ser curto e minúsculo; o componente nunca é interativo. A derivação "OBY a 18%" é o
padrão documentado para valores canônicos sem campo no tema (ver
[theme.md](theme.md#derivações-documentadas-de-token)).

```rust
Badge::new("leitura", BadgeKind::Read).id("badge-read");
Badge::new("text/plain", BadgeKind::Mime).icon(IconName::File).id("badge-mime");
```

## CopyButton

Fonte: [`src/core/copy_button.rs`](../src/core/copy_button.rs) · Canon:
[`CopyButton.prompt.md`](../../../stand-in-client-prototipo/components/core/CopyButton.prompt.md)

Chip de copiar para a área de transferência com confirmação de 1,3 s (a constante
`COPY_DURATION_MS = 1300` é testável). Posição canônica: o slot direito de um
panel-head, pareado com todo bloco de código ou valor técnico copiável.

API: `CopyButton::new(id, value)`, `.label(s)` e `.copied_label(s)` (os rótulos seguem o
idioma da interface; os defaults "Copy"/"Copied" são apenas fallback), `.on_copied(h)`.

Comportamento interno, relevante para quem depura: o estado "copiado" persiste por
instância via `window.use_keyed_state` (uma entity por element id) com expiração baseada
em relógio — o visual limpa no próximo render mesmo que o pai atrase. O clique escreve no
clipboard via `cx.write_to_clipboard`, dispara `cx.notify()` e agenda a reversão com
`cx.background_executor().timer(…)` — nunca `tokio::time::sleep`, que panica no executor
do gpui. Durante a confirmação, novos cliques são ignorados.

```rust
CopyButton::new("copy-result", value)
    .label("Copiar")
    .copied_label("Copiado")
```

## Spinner

Fonte: [`src/core/spinner.rs`](../src/core/spinner.rs) · Canon:
[`Spinner.prompt.md`](../../../stand-in-client-prototipo/components/core/Spinner.prompt.md)

Indicador circular de 15 px — fixo em toda densidade (regra do canon). Um anel de 2 px:
trilha translúcida (25% de alfa da cor resolvida) sob um arco sólido que gira 360° a cada
0,6 s, linear e infinito. A cor segue o mesmo idioma do `Icon`: herda o texto contextual,
com `.color(hsla)` como override. O arco é um SVG de anatomia (`spinner/arc.svg`) servido
pelo `DsAssets` — não é um glifo, e o catálogo de 22 permanece fechado. Use `.id(id)`
quando dois spinners dividirem o mesmo pai (ids de elemento precisam ser únicos entre
irmãos). No uso, o spinner acompanha sempre o rótulo da ação com reticências
("Conectando…") — nunca um "Loading" seco; essa regra é do call site.

```rust
// herda a cor contextual; .color(hsla) sobrescreve
div().text_color(cx.theme().muted_foreground).child(Spinner::new().id("spinner"));
```

## StatusDot

Fonte: [`src/core/status_dot.rs`](../src/core/status_dot.rs) · Canon:
[`StatusDot.prompt.md`](../../../stand-in-client-prototipo/components/core/StatusDot.prompt.md)

O vocabulário fixo de estado de conexão: ponto de 9 px com halo translúcido de 4 px.

| `DotState` | Cor | Halo | Pulsa? |
|------------|-----|------|--------|
| `On` | `ok` (verde) | `ok_dim` | não |
| `Off` | `text-3` (cinza) | — | não |
| `Busy` | `warn` (âmbar) | `warn_dim` | **sim** (o único) |
| `Err` | `err` (vermelho) | `err_dim` | não |

O halo é um anel composto — um círculo absoluto de 17 px pintado atrás do ponto, que
transborda o footprint de 9 px sem afetar o layout (o equivalente do `box-shadow 0 0 0
4px` do canon, sem usar sombra). O pulso do `Busy` é uma onda triangular de opacidade
(1,0 nas bordas do ciclo, 0,35 no meio, ciclo de 1 s). API: `StatusDot::new(state)` e
`.id(id)` para múltiplos dots `Busy` sob o mesmo pai. O ponto nunca é o único indicador —
o call site sempre o acompanha de um rótulo textual.

```rust
StatusDot::new(DotState::On) // .id(...) p/ múltiplos dots Busy sob o mesmo pai
```

## ToggleLink

Fonte: [`src/core/toggle_link.rs`](../src/core/toggle_link.rs) · Canon: classe
[`.toggle-link` no `core.css`](../../../stand-in-client-prototipo/components/core/core.css)
(sem `.prompt.md` próprio — o card "Copy / link" do canon rende `CopyButton` e
`ToggleLink`; o `Button.prompt.md` o referencia como o substituto da variante "link").

Link discreto: sem fundo nem borda, cor `link` (OBY), hover com sublinhado e `link_hover`
(BRISA), fs 12, peso 600. API: `ToggleLink::new(id, label)` e `.on_click(handler)`. O uso
típico é a ação silenciosa de adicionar item ("+ adicionar variável") abaixo de uma lista
editável — nunca um botão grande.

```rust
ToggleLink::new("add-var", "+ adicionar variável")
    .on_click(cx.listener(|_, _: &ClickEvent, _, cx| cx.notify()))
```
