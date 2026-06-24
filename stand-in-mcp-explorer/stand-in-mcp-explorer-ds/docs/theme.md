# theme — paleta, cores, densidade, tipografia e assets

Este arquivo documenta a camada de tema da crate: como os tokens canônicos de
[`tokens/*.css`](../../../stand-in-client-prototipo/tokens/) viram valores Rust e como o
restante do DS os consome. Os módulos: [`palette.rs`](../src/theme/palette.rs),
[`colors.rs`](../src/theme/colors.rs), [`density.rs`](../src/theme/density.rs),
[`typography.rs`](../src/theme/typography.rs) e, na raiz da crate,
[`assets.rs`](../src/assets.rs).

## Bootstrap

A ordem importa e acontece uma vez, na inicialização do app:

```rust
stand_in_mcp_explorer_ds::init(cx);                     // 1. gpui_component::init + densidade padrão (Regular)
theme::apply_theme(ThemeMode::Dark, cx);                // 2. instala cores + fontes + raios no Theme global
// ou, para escolher também a densidade:
theme::apply_theme_and_density(ThemeMode::Dark, Density::Compact, cx);
```

O `apply_theme` escreve no `Theme` global do gpui-component (`mode`, `colors`,
`font_family`, `mono_font_family`, `radius`, `radius_lg`) e registra o `JandiExt` como
global do gpui. Os componentes leem tudo por `cx.theme()` e `cx.global::<JandiExt>()`;
a densidade, por `cx.global::<GlobalDensity>()`.

## Paleta (`palette.rs`)

O **único arquivo da crate com literais de cor** (proibição 1), com duas fontes para um só
canon:

- **A rampa de 8 degraus** vem **direto da crate publicada
  [`jandi-colors`](https://crates.io/crates/jandi-colors)** (`0.1.0`, `no_std`, sem
  features) — os mesmos hexes que o [`tokens/colors.css`](../../../stand-in-client-prototipo/tokens/colors.css)
  do canon codifica. A crate entrega `Rgb` de 8 bits; a `const fn rgb8_to_hsla(rgb, a)`
  converte cada degrau para `Hsla` do gpui **em contexto `const`** (mesma cor, agora
  computada em vez de transcrita à mão). Um teste fixa os 8 hexes contra `jandi_colors::*`,
  então um `cargo update` que altere a paleta quebra o gate.
- **Os tokens de tema que não existem na paleta pública** (estados semânticos, superfícies,
  tokens de sintaxe JSON, `BRAND_RING`) seguem transcritos do `colors.css`, com os valores
  OKLCH pré-convertidos para `Hsla` (oklch → sRGB linear → sRGB → HSL, verificado nos
  comentários). As bordas escuras e a sombra clara, que o canon define como uma cor da
  rampa com alfa, também derivam da crate via `rgb8_to_hsla(cor.rgb, alfa)`.

![Rampa jandi de 8 degraus (suco → guerra) com valores hexadecimais](../../../stand-in-client-prototipo/screenshots/reference/guidelines/colors-palette-dark.png)

- **A rampa nomeada de 8 degraus**, da mais clara à mais escura: `SUCO`, `BRISA`, `OBY`,
  `JANDI`, `GENIPINA`, `NHANDI`, `YANDI`, `GUERRA` (derivadas de `SUCO_VERDE`, `BRISA`,
  `OBY`, `PRIMARY`, `GENIPINA`, `NHANDI`, `YANDI`, `TINTA_GUERRA` da crate). O degrau
  primário (`JANDI`) é o mesmo nos dois modos.
- **Estados semânticos:** `OK` (verde), `WARN` (âmbar), `ERR` (vermelho), cada um com a
  variante dim a 16% de alfa (`OK_DIM`, `WARN_DIM`, `ERR_DIM`).
- **Tokens de sintaxe JSON:** `TOK_KEY` (= BRISA), `TOK_STR`, `TOK_NUM`, `TOK_BOOL`; a
  pontuação usa o `text-3` do modo corrente, por isso não é constante aqui.
- **`BRAND_RING`:** o anel interno do brand-mark (branco a 8%), fixo nos dois modos.
- **Submódulos `dark` e `light`:** BG, SURFACE, SURFACE_2, SURFACE_3, BORDER, BORDER_2,
  TEXT, TEXT_2, TEXT_3, PRIMARY, PRIMARY_H, ON_PRIMARY, CODE_BG e SHADOW de cada modo,
  espelhando os blocos `[data-theme]` do CSS.

O módulo também expõe os auxiliares de auditoria WCAG `relative_luminance(hsla)` e
`contrast_ratio(l1, l2)`, usados pelos testes de contraste. O `text-3` claro sobre
superfície branca atinge ~4,88:1 — passa AA (4,5) com margem, após correção do O-006
(oby escurecido `#56758A`, desacoplado do `--oby`).

## Mapeamento para o tema (`colors.rs`)

`jandi_theme(mode)` produz um `ThemeColor` completo do gpui-component por modo. Os campos
seguem a semântica do pin (`70d2c44b`): `background`/`foreground`/`muted_foreground`
recebem BG/TEXT/TEXT_3; `secondary` é a surface-2; `sidebar`, `tab_bar` e as famílias de
lista e tabela recebem as superfícies; `primary` e a família `button_primary` recebem
JANDI com o hover do modo; `success`/`warning`/`danger` recebem os semânticos; `link` é
OBY (hover BRISA no dark, JANDI no light); `ring` e `selection` são OBY a 22%. O que o
mapeamento não cobre cai no `..*ThemeColor::dark()/light()` do próprio pin.

O que o `ThemeColor` padrão **não** carrega vive no **`JandiExt`** (lacunas honestas, não
campos reinterpretados): `surface_3`, `border_2`, `ok_dim`/`warn_dim`/`err_dim`,
`code_bg`, os quatro `tok_*` de JSON e `shadow_overlay`. Acesso por
`cx.global::<JandiExt>()`; as variantes `JandiExt::dark()`/`light()` são instaladas pelo
`apply_theme`.

## Densidade (`density.rs`)

Três níveis governam cinco variáveis (proibição 7) — valores de
[`tokens/spacing.css`](../../../stand-in-client-prototipo/tokens/spacing.css):

![Os três níveis de densidade (compact · regular★ · comfy) com os 5 valores de espaçamento](../../../stand-in-client-prototipo/screenshots/reference/guidelines/density-dark.png)

| Variável | Compact | Regular (padrão) | Comfy |
|----------|---------|------------------|-------|
| `pad()` | 10 | 14 | 18 |
| `row_h()` | 32 | 38 | 46 |
| `gap()` | 8 | 12 | 16 |
| `fs()` | 13 | 14 | 15 |
| `radius()` | 8 | 10 | 12 |

Além delas, `sidebar_w()` encolhe a barra lateral no compact (304 → 280). Os **raios por
papel são fixos** e não escalam: input 8, chip 7, badge 6, botão 9, card 10, pílula 99.
O anel de foco é OBY a 22% com 3 px (`FOCUS_RING_COLOR`/`FOCUS_RING_WIDTH`). A densidade
corrente é o global `GlobalDensity` (instalado pelo `init` como Regular e trocado pelo
`apply_theme_and_density`); ela é deref para `Density`, então
`cx.global::<GlobalDensity>().pad()` funciona direto.

## Tipografia (`typography.rs`)

Duas famílias, e somente duas (proibição 2): `SANS` = Hanken Grotesk, `MONO` = JetBrains
Mono (helpers `sans_family()`/`mono_family()` devolvem `SharedString`). O roteamento
(proibição 6) é regra de componente: identificadores, caminhos, JSON, timestamps,
contadores e badges são sempre mono; prosa é sempre sans.

A escala de tamanhos, de [`tokens/typography.css`](../../../stand-in-client-prototipo/tokens/typography.css):
`FS_2XS` 10,5 (badges) · `FS_XS` 11 (metadados, section labels) · `FS_SM` 12 (rótulos de
campo) · `FS_MD` 13 (inputs, botões, itens de lista) · `FS_LG` 14 (corpo padrão) ·
`FS_XL` 15 (nome da marca) · `FS_TITLE` 20 (títulos de detalhe). Pesos: `W_REGULAR`,
`W_MEDIUM`, `W_SEMIBOLD`, `W_BOLD`. As constantes de tracking (`TRACK_TIGHT`,
`TRACK_WIDE`, `TRACK_WIDER`) registram os valores do canon — o GPUI pinado não expõe
letter-spacing, então elas aguardam API futura.

## Derivações documentadas de token

Quando o canon usa um valor que não tem campo direto no tema, a crate o deriva da
constante de paleta com o alfa documentado — sempre com um comentário apontando a origem.
As derivações em uso: **OBY a 18%** (fundo do `Badge::Role`), **OBY a 22%** (anel de
seleção do `PresetCard`, `ring` e `selection` do tema), **OBY a 10%** (fundo do
`HintBar`) e os **dims a 16%** dos estados semânticos. Esse é o padrão a seguir para
novos valores canônicos sem campo: derivar da constante, nunca inventar literal.

## Assets (`assets.rs`)

`DsAssets` é o `AssetSource` combinado que o app (e a gallery) instala na inicialização:
serve os 22 SVGs do catálogo (embarcados via `include_bytes!`, chave `icons/<nome>.svg`),
o SVG de anatomia do Spinner (`spinner/arc.svg` — anatomia de componente, fora do
catálogo de glifos) e delega todo o resto (as fontes, principalmente) ao
`gpui_component_assets::Assets`. Os testes do módulo provam que os 22 assets resolvem e
não estão vazios — a defesa contra o ícone-façade.
