# data — Panel, ListItem, ListSearch, PresetCard, LogRow, EmptyState, HintBar, JsonView

Este arquivo documenta o lado Rust/GPUI dos componentes do grupo `data` — os primitivos
do padrão lista→detalhe. As regras visuais vinculantes moram nos `.prompt.md` do canon,
em [`components/data/`](../../../stand-in-client-prototipo/components/data/).

Uma distinção de semântica fixa atravessa o grupo e nunca pode ser trocada: **`ListItem`
seleciona com filete** (a barra de 2 px à esquerda — navegação em lista densa);
**`PresetCard` seleciona com borda + anel** (escolha de configuração).

![Grupo data em contexto de app — ListSearch, ListItem, HintBar, Panel, JsonView e LogRow](../../../stand-in-client-prototipo/screenshots/reference/components/data-dark.png)

## Panel

Fonte: [`src/data/panel.rs`](../src/data/panel.rs) · Canon:
[`Panel.prompt.md`](../../../stand-in-client-prototipo/components/data/Panel.prompt.md)

O contêiner canônico de qualquer bloco da coluna de detalhe: superfície com borda de
1 px, raio que **escala com a densidade** (`Density::radius()` — proibição 7) e margem
inferior de 16 px embutida (os Panels empilham direto). Sem sombra — card in-flow
(proibição 4). O cabeçalho só existe quando há título; o título é curto e nominal
("Parâmetros", nunca uma frase) e o componente o põe em maiúsculas — o caller escreve em
caixa normal.

API: `Panel::new()`, `.title(s)`, `.icon(IconName)` (14 px, antes do título),
`.right_children(iter)` (o slot direito do cabeçalho — `Button` ghost sm, `CopyButton`),
`.children(iter)` (o corpo, com padding de 16 px) e `.id(id)`.

```rust
Panel::new()
    .id("panel-params")
    .title("Parâmetros") // o componente aplica UPPERCASE
    .icon(IconName::Bolt)
    .right_children([copy_button.into_any_element()])
    .children([field.into_any_element()])
```

## ListItem

Fonte: [`src/data/list_item.rs`](../src/data/list_item.rs) · Canon:
[`ListItem.prompt.md`](../../../stand-in-client-prototipo/components/data/ListItem.prompt.md)

Linha interativa de duas linhas: em cima o `name` mono 13 px (identificador técnico —
nunca capitalizar nem traduzir) com o slot de badge à direita; embaixo a descrição sans
12 px em `text-3`, limitada a duas linhas. Hover aplica `surface-2`; a seleção soma o
filete de 2 px OBY à esquerda — um elemento filho, não uma sombra. O filete está sempre
presente (transparente quando inativo) para o layout não pular na seleção.

API: `ListItem::new(id, name)`, `.desc(s)` (prosa, sentence case, termina com ponto),
`.badge(AnyElement)`, `.selected(bool)` e `.on_click(handler)` — a seleção é do caller.

Nota de fidelidade: o GPUI não expõe `line-clamp`; o corte em duas linhas é aproximado
com `max_h(35 px)` (duas vezes a line-height de 17,4 px) + `overflow_hidden`. Uma API
nativa de line-clamp superaria essa aproximação.

### DS-extension 031/M1 — altura fixa (linha uniforme)

**O que mudou.** O `ListItem` passou de altura **variável** (o contêiner de `desc` era
opcional e clampado por `max_h`) para altura **fixa** (`LIST_ROW_HEIGHT` = 84 px),
reservando sempre a área de 2 linhas de `desc` mesmo quando o campo não é informado.

**Por quê.** Habilita o primitivo de windowing `gpui::uniform_list` (031 — renderização
virtualizada das listas do MCP Explorer). O `uniform_list` **exige altura de linha
uniforme** — mede a primeira e assume que as demais têm a mesma altura. Sem essa
extensão, a barra lateral de ~4.000 tools esgotaria a memória (OOM) sob o regime
eager `.children(map)`.

**Constantes públicas** (exportadas como API do grupo `data`):

| Const | Valor | Descrição |
|---|---|---|
| `NAME_LINE_H` | `20.0` | Altura reservada da linha do `name` (13 px semibold com respiro) |
| `DESC_RESERVED_H` | `35.0` | Altura reservada da área de `desc` (2 × 17,4 px) |
| `LIST_ROW_HEIGHT` | `84.0` | Altura externa garantida: `12 + 20 + 5 + 35 + 12` |

**Retrocompatibilidade.** A API pública do `ListItem` (`new` / `.desc` / `.badge` /
`.selected` / `.on_click`) **não mudou**. Linhas com `desc` de 2 linhas renderizam
pixel-idênticas ao regime anterior (83 → 84 px, imperceptível); linhas sem `desc`
ganham respiro inferior (intencional, decidido em 031 D3).

```rust
ListItem::new("li-read_file", "read_file")
    .desc("Lê o conteúdo de um arquivo do disco como texto UTF-8.")
    .badge(Badge::new("leitura", BadgeKind::Read).into_any_element())
    .selected(selected_ix == 0)
    .on_click(cx.listener(|this, _: &ClickEvent, _, cx| { this.selected = 0; cx.notify(); }))
```

## ListSearch

Fonte: [`src/data/list_search.rs`](../src/data/list_search.rs) · Canon:
[`ListSearch.prompt.md`](../../../stand-in-client-prototipo/components/data/ListSearch.prompt.md)

Busca fixa no topo de uma coluna de lista: contêiner com borda inferior envolvendo um
`Input` do gpui-component com a lupa de 14 px como prefixo. O texto é **sans** — busca é
interação humana, a exceção à regra do mono-por-padrão. Não há botão de buscar: o filtro
é imediato, via assinatura de `InputEvent::Change` no `InputState` do caller. A fixação
no topo (stickiness) é responsabilidade do caller: o contêiner de rolagem deve manter
este elemento antes da região rolável.

API: `ListSearch::new(&state)` (o caller cria o `InputState` com o placeholder nomeando a
coleção — "Filtrar tools…") e `.id(id)`.

**Sem sombra, nunca** — o `ListSearch` antigo do `-ds` pré-canon carregava `shadow_md` em
um input in-flow, a violação que originou a O-001; o componente atual nasceu limpo e o
rustdoc preserva a lição.

```rust
// o caller cria o InputState; o placeholder nomeia a coleção
let search = cx.new(|cx| InputState::new(window, cx).placeholder("Filtrar tools…"));
ListSearch::new(&search).id("list-search")
```

## PresetCard

Fonte: [`src/data/preset_card.rs`](../src/data/preset_card.rs) · Canon:
[`PresetCard.prompt.md`](../../../stand-in-client-prototipo/components/data/PresetCard.prompt.md)

Card selecionável de configuração: nome mono 12,5, badge opcional (tipicamente `Badge`
Muted) e descrição de **uma** linha sem ponto final, truncada. Hover degrau para
`surface-3` com `border-2` — passos de superfície, nunca escala nem sombra. A seleção
desenha a borda OBY mais o **anel composto** de 2 px em OBY a 22%: um filho absoluto de
inset negativo pintado atrás do card (a técnica de halo do M4), que transborda a
border-box sem afetar o layout — o equivalente do `box-shadow 0 0 0 2px` do canon sem
usar `shadow_*`.

API: `PresetCard::new(id, name)`, `.desc(s)`, `.badge(AnyElement)`, `.selected(bool)`,
`.on_click(handler)` — a seleção é do caller.

```rust
PresetCard::new("pc-stdio", "stand-in-reference")
    .desc("Servidor de referência local (stdio) com 3 ferramentas")
    .badge(Badge::new("stdio", BadgeKind::Muted).into_any_element())
    .selected(selected_ix == 0)
    .on_click(cx.listener(|this, _: &ClickEvent, _, cx| { this.selected = 0; cx.notify(); }))
```

## LogRow

Fonte: [`src/data/log_row.rs`](../src/data/log_row.rs) · Canon:
[`LogRow.prompt.md`](../../../stand-in-client-prototipo/components/data/LogRow.prompt.md)

Linha de log estilo terminal em grade fixa: hora em 86 px, nível em 74 px, mensagem em
`1fr` — tudo mono 12,5, alinhado pela linha de base. As cores de nível são fixas e nunca
se reinterpretam:

| `LogLevel` | Cor |
|------------|-----|
| `Info` | OBY |
| `Ok` | `ok` |
| `Warn` | `warn` |
| `Error` | `err` |
| `Debug` | `text-3` |

API: `LogRow::new(time, level, level_label, message)` e `.id(id)`. O rótulo de nível é
texto do caller (tipicamente minúsculo, voz de máquina) — o componente só o pinta. O
contêiner `.log` e a toolbar fixa do feed são composição do app consumidor, assim como a
ordenação (mais recente no topo).

```rust
LogRow::new("14:02:32", LogLevel::Ok, "ok", "conectado a server-filesystem (57ms)")
```

## EmptyState

Fonte: [`src/data/empty_state.rs`](../src/data/empty_state.rs) · Canon:
[`EmptyState.prompt.md`](../../../stand-in-client-prototipo/components/data/EmptyState.prompt.md)

Painel de vazio centrado que **ensina o próximo passo** — nunca um beco sem saída. O
glifo de 64×64 carrega **o segundo (e último) gradiente legítimo** do DS (surface-2 →
surface, proibição 5) com o ícone de 28 px centrado; seguem o título (20 px, peso 700), o
corpo, os passos numerados e uma única ação primária.

API: `EmptyState::new(title)`, `.icon(IconName)`, `.body(s)`,
`.steps(Vec<EmptyStep>)` (`EmptyStep::new(n, title, sub)`; três passos é o canônico, zero
para vazios triviais), `.action(el)` e `.id(id)`.

Para vazios pequenos **dentro** de um `Panel`, o padrão é a função livre
`result_empty(text)` — uma linha discreta com o padding canônico. Ela devolve um `Div`
sem cor definida (não tem acesso ao tema); o caller encadeia
`.text_color(cx.theme().muted_foreground)`.

```rust
EmptyState::new("Pronto para inspecionar")
    .icon(IconName::Plug)
    .body("Escolha um servidor salvo ou ajuste a conexão na barra lateral.")
    .steps(vec![
        EmptyStep::new("1", "Escolha o transporte", "STDIO p/ locais; SSE/HTTP p/ remotos."),
        EmptyStep::new("2", "Conecte ao servidor", "npx -y @modelcontextprotocol/server-…"),
    ])
    .action(Button::new("Conectar agora").variant(ButtonVariant::Primary).id("empty-action"))
```

## HintBar

Fonte: [`src/data/hint_bar.rs`](../src/data/hint_bar.rs) · Canon:
[`HintBar.prompt.md`](../../../stand-in-client-prototipo/components/data/HintBar.prompt.md)

Dica inline do modo guiado: fundo OBY a 10% (derivação documentada de token, como o
`Badge::Role` a 18%), ícone `info` de 14 px e texto didático 12,5 em `text-2`. **Nunca é
permanente** — o caller a renderiza condicionada ao toggle de modo guiado. A voz é de
professor paciente, em uma ou duas frases.

API: `HintBar::new()`, depois `.text(s)` para texto simples **ou** `.children(iter)` para
conteúdo com destaques em negrito inline (quando os dois são definidos, `children`
vence); `.id(id)`.

```rust
HintBar::new().text("Tools são funções que o servidor expõe.");
// ou, com destaques inline em negrito (children vence text):
HintBar::new().children(bold_children)
```

## JsonView

Fonte: [`src/data/json_view.rs`](../src/data/json_view.rs) (+ o tokenizador puro em
[`src/data/json_tokens.rs`](../src/data/json_tokens.rs)) · Canon:
[`JsonView.prompt.md`](../../../stand-in-client-prototipo/components/data/JsonView.prompt.md)

A superfície canônica de exibição de JSON: bloco `.code` (mono 12,5, fundo `code_bg`,
borda 1 px, raio 8, pre-wrap) com sintaxe colorida. As cores de token são fixas: chave →
`tok_key` (BRISA), string → `tok_str`, número → `tok_num`, booleano e null → `tok_bool`,
pontuação → `text-3` — todas via `JandiExt` e tema.

API: `JsonView::new(json)` (o caller pretty-printa; o componente só colore),
`JsonView::plain(text)` (a mesma superfície sem coloração — logs, traces, saída crua) e
`.id(id)`.

Implementação: o realce usa `gpui::StyledText` com `with_highlights` — o mecanismo nativo
do gpui pinado para texto inline multicolorido (a investigação do pin está no rustdoc). O
tokenizador é uma função pura e tolerante: JSON malformado degrada para texto sem cor em
vez de panicar, e o conjunto de testes o cobre exaustivamente.

```rust
JsonView::new(pretty_json); // o caller faz o pretty-print; o componente colore
JsonView::plain(raw_text)   // mesma superfície, sem coloração
```
