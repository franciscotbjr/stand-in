# forms — Field, KeyValueRow, SegmentedControl, Select

Este arquivo documenta o lado Rust/GPUI dos componentes do grupo `forms`. As regras
visuais vinculantes moram nos `.prompt.md` do canon, em
[`components/forms/`](../../../stand-in-client-prototipo/components/forms/).

Os três primeiros componentes são **stateless por contrato**: o caller é dono do estado
(`Entity<InputState>` para os campos, o índice selecionado para o segmented) e reage às
mudanças via assinatura de eventos ou handlers criados com `cx.listener`. O `Select`
guarda apenas o estado interno de popup (aberto/fechado); a seleção continua do caller.

![Grupo forms em contexto de app — Field, SegmentedControl, KeyValueRow e estado de erro](../../../stand-in-client-prototipo/screenshots/reference/components/forms-dark.png)

## Field

Fonte: [`src/forms/field.rs`](../src/forms/field.rs) · Canon:
[`Field.prompt.md`](../../../stand-in-client-prototipo/components/forms/Field.prompt.md)

Campo rotulado que envolve o `Input` do gpui-component com o estilo canônico. O caller
cria o `Entity<InputState>` (escolhendo single-line, multi-line ou auto-grow), assina
`InputEvent::Change` e passa a referência ao `Field`, que é só apresentação.

| Builder | Efeito |
|---------|--------|
| `Field::new(&state)` | Envolve o `InputState` do caller |
| `.label(s)` | Rótulo acima do campo (fs 12, peso 600, `text-2`) |
| `.required()` | Acrescenta o `*` vermelho ao rótulo |
| `.hint(s)` | Dica abaixo do campo (11,5 px, `text-3`) |
| `.invalid()` | Borda em `err` |
| `.mono(bool)` | Mono por padrão (`true`) — conteúdo técnico; `false` só para prosa humana |
| `.long()` | Variante textarea (min-height 78); o caller também configura o `InputState` como multi-line |
| `.id(id)` | Element id |

O fundo do input depende do modo: `background` no dark, `surface-2` no light. A borda
padrão é `border_2`; o anel de foco vem do tema (`ring` = OBY a 22%), aplicado pelo
próprio gpui-component.

```rust
// o caller é dono do InputState (single-line, multi-line ou auto-grow)
let cmd = cx.new(|cx| InputState::new(window, cx).placeholder("npx"));
Field::new(&cmd)
    .label("Comando")
    .required()
    .hint("Separe argumentos por espaço")
    .id("field-cmd")
```

## KeyValueRow

Fonte: [`src/forms/key_value_row.rs`](../src/forms/key_value_row.rs) · Canon:
[`KeyValueRow.prompt.md`](../../../stand-in-client-prototipo/components/forms/KeyValueRow.prompt.md)

Par chave-valor removível, para listas editáveis como variáveis de ambiente. A anatomia
foi revisada na 025 por decisão do dono do design: a chave fica **empilhada acima** do
valor (os dois em largura total, mono 12 px), com o `IconButton` X centralizado à direita
das duas linhas — o empilhamento venceu o lado-a-lado pela largura legível dos campos. A
coluna dos campos usa `flex_1` + `min_w(0)`, então a linha preenche o contêiner sem
colapsar para o min-content.

API: `KeyValueRow::new(&key_state, &value_state)` (os dois `InputState` são do caller),
`.on_remove(handler)` (encaminhado ao X via `on_click_boxed`) e `.id(id)` — use um
identificador **estável** por linha, nunca o índice volátil do vetor (o risco de
índice-obsoleto no remove foi documentado na 022). Linhas novas entram por um
`ToggleLink` "+ adicionar" abaixo da lista.

```rust
KeyValueRow::new(&key_state, &value_state)
    .id(("kv-row", row_id)) // id ESTÁVEL por linha, nunca o índice do vetor
    .on_remove(remove_handler)
```

## SegmentedControl

Fonte: [`src/forms/segmented_control.rs`](../src/forms/segmented_control.rs) · Canon:
[`SegmentedControl.prompt.md`](../../../stand-in-client-prototipo/components/forms/SegmentedControl.prompt.md)

Seletor de 2 a 4 opções mutuamente exclusivas, com rótulos de uma palavra (siglas em
maiúsculas para domínios técnicos, como STDIO/SSE/HTTP). Acima de 4 opções ou com rótulos
longos, o padrão certo é outro (lista ou `Select`).

API: `SegmentedControl::new(id, options, selected_ix)` — `options` é um vetor de pares
`(valor, rótulo)` — e `.handlers(vec)` com um `ClickHandler` por opção, na ordem,
criados pelo caller via `cx.listener` (handlers faltantes são preenchidos com `None`,
o que rende segmentos não-interativos, útil no modo captura). O segmento ativo recebe
fundo `primary` e texto `on-primary`.

**Delta consciente contra o CSS do canon:** o `.seg button[data-on]` canônico carrega um
`box-shadow` sutil; a regra de elevação da 024 (sombra só em overlays) prevalece, e o
segmento ativo aqui **não tem sombra** — o contraste primário/on-primary já o distingue.

```rust
let options = vec![
    ("stdio".into(), "STDIO".into()),
    ("sse".into(), "SSE".into()),
    ("http".into(), "HTTP".into()),
];
SegmentedControl::new("seg-transport", options, selected_ix)
    .handlers(handlers) // Vec<ClickHandler>, um por opção na ordem
```

## Select (extensão 025)

Fonte: [`src/forms/select.rs`](../src/forms/select.rs) · Canon:
[`Select.prompt.md`](../../../stand-in-client-prototipo/components/forms/Select.prompt.md)
(entrada escrita na 025 — decisões D9/D10, origem O-003: o seletor de idioma)

Dropdown compacto de escolha única: gatilho de 32 px com aparência de input (rótulo da
opção ativa + chevron rotacionado) e lista de opções em **overlay**. Para 2 a 10 opções
de rótulo curto; acima disso, ou com busca, o padrão certo é outro.

API: `Select::new(id, options, selected_index)` (pares `(valor, rótulo)`),
`.placeholder(s)`, `.mono(bool)` (`false` por padrão — rótulos humanos em sans; `true`
para valores técnicos) e `.on_change(handler)` com a assinatura
`(novo_índice, valor, &mut Window, &mut App)`. A seleção é do caller; o componente
fecha o popup e chama o handler.

Duas notas de implementação que valem para quem mantém:

- **Veredito construir, não compor:** o `Select`/`Combobox` do pin do gpui-component é
  profundamente acoplado ao tema dele (fundo, borda, raio, ícone e delegates próprios);
  alcançar o canon jandi através dele exigiria brigar com ~10 ganchos. O componente foi
  construído sobre `gpui::deferred(anchored())` com os nossos tokens — mais simples e
  mais fiel. A investigação completa está no rustdoc do arquivo.
- **A única sombra legítima da crate:** o popup é um overlay, então carrega `shadow_md()`
  (o mesmo método que o pin usa nos seus popups; o token `shadow_overlay` é um `Hsla`
  que a API de sombra do gpui não aceita diretamente). O estado interno
  (aberto/fechado + bounds do gatilho) persiste por instância via
  `window.use_keyed_state`; o clique fora fecha via `on_mouse_down_out`.

```rust
let options = vec![("pt".into(), "Português".into()), ("en".into(), "English".into())];
Select::new("select-lang", options, selected_index)
    .on_change(move |ix, _value, _window, _app| {
        // a seleção é do caller; o componente fecha o popup e chama o handler
    })
```
