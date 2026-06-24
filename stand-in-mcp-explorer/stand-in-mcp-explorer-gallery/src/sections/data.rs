//! Data section — Panel + ListItem + ListSearch + PresetCard.
//!
//! Panel: with title + icon + right button + body (Field); a second bare card.
//! ListItem: 3 items (read_file / write_file / list_dir) with Badges read/write,
//! functional selection (clicking updates the bar via GalleryShell state), and
//! one item with a long description proving 2-line clamp.
//!
//! ListSearch (M14): sticky search input (no shadow — O-001 lesson) filtering
//! the 3 ListItems live (typing "rea" filters to read_file; capture seeds "re").
//! PresetCard (M14): 3 selectable cards (stdio/SSE/HTTP) with selection = oby
//! border + 2px composite ring; demoed alongside ListItems for semantic contrast.
//!
//! Selection semantics: ListItem = 2px oby left bar + surface-2 — never border+ring.
//! PresetCard = border oby + ring 2px oby-22% — never a filete.
//! Panel = no shadow (in-flow).

use gpui::{ClickEvent, IntoElement, ParentElement, SharedString, Styled, div};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{ActiveTheme as _, v_flex};
use stand_in_mcp_explorer_ds::core::button::{ButtonVariant, ClickHandler};
use stand_in_mcp_explorer_ds::core::{Badge, BadgeKind, Button, CopyButton, IconName};
use stand_in_mcp_explorer_ds::data::{JsonView, ListItem, ListSearch, Panel, PresetCard};
use stand_in_mcp_explorer_ds::forms::Field;
use stand_in_mcp_explorer_ds::theme::typography;

use super::util::{px, section_body, section_label};
use crate::shell::GalleryShell;

pub fn render_data(
    _state: &str,
    _mode: &str,
    this: &GalleryShell,
    cx: &mut gpui::Context<GalleryShell>,
) -> gpui::AnyElement {
    let t = cx.theme().clone();
    let mono = t.mono_font_family.clone();

    // Search value — read live every frame for immediate filter.
    let search_val = this
        .search_input
        .as_ref()
        .map(|e| e.read(cx).value())
        .unwrap_or_default();

    let filter = |s: &str| -> bool {
        if search_val.is_empty() {
            return true;
        }
        s.to_lowercase().contains(&search_val.to_lowercase())
    };

    // Panel with title + icon + right + body
    let panel_with_head = Panel::new()
        .id("panel-demo-head")
        .title("Par\u{e2}metros")
        .icon(IconName::Bolt)
        .right_children([Button::new("Exemplo")
            .variant(ButtonVariant::Ghost)
            .sm()
            .into_any_element()])
        .children([this
            .cmd_input
            .as_ref()
            .map(|e| Field::new(e).into_any_element())
            .unwrap_or_else(|| div().into_any_element())]);

    // Panel without title (bare card)
    let panel_bare = Panel::new().id("panel-demo-bare").children([div()
        .text_size(px(typography::FS_SM))
        .text_color(t.muted_foreground)
        .child(SharedString::from("Card solto sem cabe\u{e7}alho."))
        .into_any_element()]);

    // ListItem handlers — each toggles the selected item
    let sel = this.selected_data_item;
    let sel0 = sel;
    let sel1 = sel;
    let sel2 = sel;
    let sel3 = sel;

    let handler0: ClickHandler = Box::new(cx.listener(
        move |this: &mut GalleryShell, _: &ClickEvent, _window, cx| {
            this.selected_data_item = if this.selected_data_item == 0 {
                usize::MAX
            } else {
                0
            };
            cx.notify();
        },
    ));
    let handler1: ClickHandler = Box::new(cx.listener(
        move |this: &mut GalleryShell, _: &ClickEvent, _window, cx| {
            this.selected_data_item = if this.selected_data_item == 1 {
                usize::MAX
            } else {
                1
            };
            cx.notify();
        },
    ));
    let handler2: ClickHandler = Box::new(cx.listener(
        move |this: &mut GalleryShell, _: &ClickEvent, _window, cx| {
            this.selected_data_item = if this.selected_data_item == 2 {
                usize::MAX
            } else {
                2
            };
            cx.notify();
        },
    ));
    let handler3: ClickHandler = Box::new(cx.listener(
        move |this: &mut GalleryShell, _: &ClickEvent, _window, cx| {
            this.selected_data_item = if this.selected_data_item == 3 {
                usize::MAX
            } else {
                3
            };
            cx.notify();
        },
    ));

    let list_items = v_flex()
        .w_full()
        .child(
            ListItem::new("li-read_file", "read_file")
                .desc("L\u{ea} o conte\u{fa}do completo de um arquivo do disco como texto UTF-8.")
                .selected(sel0 == 0)
                .badge(
                    Badge::new("leitura", BadgeKind::Read).into_any_element(),
                )
                .on_click(move |ev, w, cx| handler0(ev, w, cx)),
        )
        .child(
            ListItem::new("li-write_file", "write_file")
                .desc("Grava dados no disco, sobrescrevendo ou criando um novo arquivo.")
                .selected(sel1 == 1)
                .badge(
                    Badge::new("escrita", BadgeKind::Write).into_any_element(),
                )
                .on_click(move |ev, w, cx| handler1(ev, w, cx)),
        )
        .child(
            ListItem::new("li-list_dir", "list_dir")
                .desc("Lista todos os arquivos e subdiret\u{f3}rios em um caminho. "
                    .to_owned() + "Suporta filtragem por padr\u{e3}o glob e ordena\u{e7}\u{e3}o por nome. "
                    + "Arquivos ocultos s\u{e3}o exib\u{ed}dos quando o modo de depura\u{e7}\u{e3}o est\u{e1} ativo.")
                .selected(sel2 == 2)
                .badge(
                    Badge::new("leitura", BadgeKind::Read).into_any_element(),
                )
                .on_click(move |ev, w, cx| handler2(ev, w, cx)),
        )
        // 031/M1: 4th item without desc (and without badge) — proves the
        // fixed-height regime: this row still has the same height as the
        // other three.
        .child(
            ListItem::new("li-no_desc", "no_desc_item")
                .selected(sel3 == 3)
                .on_click(move |ev, w, cx| handler3(ev, w, cx)),
        );

    // Filtered list items (live search) — each creates its own toggling closure.
    let filtered_items = {
        let mut col = v_flex().w_full();

        let items: [(usize, &str, BadgeKind, &str); 3] = [
            (0, "li-f-read_file", BadgeKind::Read, "read_file"),
            (1, "li-f-write_file", BadgeKind::Write, "write_file"),
            (2, "li-f-list_dir", BadgeKind::Read, "list_dir"),
        ];

        for (idx, id, kind, name) in items {
            if !filter(name) {
                continue;
            }
            let badge_label = match kind {
                BadgeKind::Read => "leitura",
                BadgeKind::Write => "escrita",
                _ => "muted",
            };
            let selected = match idx {
                0 => sel0 == 0,
                1 => sel1 == 1,
                2 => sel2 == 2,
                _ => false,
            };
            let handler: ClickHandler = Box::new(cx.listener(
                move |this: &mut GalleryShell, _: &ClickEvent, _window, cx| {
                    let current = this.selected_data_item;
                    this.selected_data_item = if current == idx { usize::MAX } else { idx };
                    cx.notify();
                },
            ));
            col = col.child(
                ListItem::new(id, name)
                    .desc(desc_for(name))
                    .selected(selected)
                    .badge(Badge::new(badge_label, kind).into_any_element())
                    .on_click(move |ev, w, cx| handler(ev, w, cx)),
            );
        }

        col
    };

    // Search input (from entity created in ensure_inputs)
    let search_input = this.search_input.as_ref().cloned();

    // PresetCard handlers
    let psel = this.selected_preset;
    let psel0 = psel;
    let psel1 = psel;
    let psel2 = psel;

    let preset0: ClickHandler = Box::new(cx.listener(
        move |this: &mut GalleryShell, _: &ClickEvent, _window, cx| {
            this.selected_preset = if this.selected_preset == 0 {
                usize::MAX
            } else {
                0
            };
            cx.notify();
        },
    ));
    let preset1: ClickHandler = Box::new(cx.listener(
        move |this: &mut GalleryShell, _: &ClickEvent, _window, cx| {
            this.selected_preset = if this.selected_preset == 1 {
                usize::MAX
            } else {
                1
            };
            cx.notify();
        },
    ));
    let preset2: ClickHandler = Box::new(cx.listener(
        move |this: &mut GalleryShell, _: &ClickEvent, _window, cx| {
            this.selected_preset = if this.selected_preset == 2 {
                usize::MAX
            } else {
                2
            };
            cx.notify();
        },
    ));

    v_flex()
        .flex_1()
        .min_w(px(0.))
        .h_full()
        .overflow_y_scrollbar().child(section_body()
        // Section status
        .child(section_label(
            &format!(
                "data sele\u{e7}\u{e3}o: item #{} | preset #{}",
                if sel0 < 4 {
                    sel0.to_string()
                } else {
                    "nenhum".into()
                },
                if psel0 < 3 {
                    psel0.to_string()
                } else {
                    "nenhum".into()
                },
            ),
            &t,
            &mono,
        ))
        // Panel — with head
        .child(section_label("Panel — com t\u{ed}tulo + direito", &t, &mono))
        .child(div().px(px(14.)).child(panel_with_head))
        // Panel — bare card
        .child(section_label(
            "Panel — card solto (sem cabe\u{e7}alho)",
            &t,
            &mono,
        ))
        .child(div().px(px(14.)).child(panel_bare))
        // ListSearch + filtered ListItems
        .child(section_label(
            "ListSearch + ListItem — filtro ao vivo (O-001 vingada: SEM sombra)",
            &t,
            &mono,
        ))
        .child(
            div()
                .mt(px(8.))
                .rounded(px(8.))
                .border_1()
                .border_color(t.border)
                .bg(t.background)
                .overflow_hidden()
                .child(
                    v_flex().w_full().child(
                        search_input
                            .map(|e| ListSearch::new(&e).id("list-search-demo").into_any_element())
                            .unwrap_or_else(|| div().into_any_element()),
                    ),
                )
                .child(filtered_items),
        )
        // ListItem — 3 items (original, non-filtered)
        .child(section_label(
            "ListItem (original, sem filtro) \u{b7} read_file / write_file / list_dir / no_desc_item (sele\u{e7}\u{e3}o funcional + clamp + altura fixa 031)",
            &t,
            &mono,
        ))
        .child(
            div()
                .mt(px(8.))
                .rounded(px(8.))
                .border_1()
                .border_color(t.border)
                .bg(t.background)
                .overflow_hidden()
                .child(list_items),
        )
        // PresetCard — 3 cards with functional selection
        .child(section_label(
            "PresetCard — borda oby + anel 2px (CONTRASTE com ListItem: anel vs filete)",
            &t,
            &mono,
        ))
        .child(
            v_flex()
                .mt(px(8.))
                .w_full()
                .gap(px(0.)) // cards bring their own mb(7)
                .child(
                    PresetCard::new("pc-stdio", "stand-in-reference")
                        .desc("Servidor de refer\u{ea}ncia local (stdio) com 3 ferramentas")
                        .badge(Badge::new("stdio", BadgeKind::Muted).into_any_element())
                        .selected(psel0 == 0)
                        .on_click(move |ev, w, cx| preset0(ev, w, cx)),
                )
                .child(
                    PresetCard::new("pc-sse", "server-sse-remote")
                        .desc("Servidor remoto via SSE com subscriptions ativas")
                        .badge(Badge::new("SSE", BadgeKind::Muted).into_any_element())
                        .selected(psel1 == 1)
                        .on_click(move |ev, w, cx| preset1(ev, w, cx)),
                )
                .child(
                    PresetCard::new("pc-http", "filesystem-prod")
                        .desc("Servidor de produ\u{e7}\u{e3}o HTTP com 22 ferramentas")
                        .badge(Badge::new("HTTP", BadgeKind::Muted).into_any_element())
                        .selected(psel2 == 2)
                        .on_click(move |ev, w, cx| preset2(ev, w, cx)),
                ),
        )
        // JsonView — Resultado com CopyButton (uso can\u{f4}nico)
        .child(section_label(
            "JsonView — Panel + CopyButton (uso can\u{f4}nico)",
            &t,
            &mono,
        ))
        .child(
            div()
                .mt(px(8.))
                .px(px(14.))
                .child(
                    Panel::new()
                        .title("Resultado")
                        .icon(IconName::Bolt)
                        .right_children([
                            CopyButton::new(
                                "copy-json-call-tool",
                                JSON_EXAMPLE,
                            )
                            .label("Copiar")
                            .copied_label("Copiado")
                            .into_any_element(),
                        ])
                        .children([JsonView::new(JSON_EXAMPLE)
                            .into_any_element()]),
                ),
        )
        // JsonView plain — texto cru sem highlight
        .child(section_label(
            "JsonView::plain — texto cru (sem highlight)",
            &t,
            &mono,
        ))
        .child(
            div()
                .mt(px(8.))
                .px(px(14.))
                .child(JsonView::plain(PLAIN_EXAMPLE).into_any_element()),
        ))
        .into_any_element()
}

/// Description strings for the filtered list items (same as the originals).
fn desc_for(name: &str) -> &'static str {
    match name {
        "read_file" => "L\u{ea} o conte\u{fa}do completo de um arquivo do disco como texto UTF-8.",
        "write_file" => "Grava dados no disco, sobrescrevendo ou criando um novo arquivo.",
        _ => "Lista todos os arquivos e subdiret\u{f3}rios em um caminho.",
    }
}

/// Rich JSON example for the JsonView demo — exercises all token kinds.
const JSON_EXAMPLE: &str = r#"{
  "name": "stand-in",
  "version": "0.0.4",
  "active": true,
  "port": null,
  "score": 95.5,
  "ratio": -0.25,
  "exponent": 1.5e3,
  "tools": [
    "greet",
    "weather",
    "read_file"
  ],
  "config": {
    "timeout": 5000,
    "retry": false,
    "nested": {
      "deep_key": "deep value"
    }
  }
}"#;

/// Plain text example — no syntax coloring.
const PLAIN_EXAMPLE: &str = "stdout: Connected to stand-in-reference (57ms)\nstderr: [INFO] tools list returned 3 tools\n\nReady.";
