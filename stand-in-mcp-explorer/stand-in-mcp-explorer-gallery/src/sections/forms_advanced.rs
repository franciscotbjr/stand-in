//! Forms Advanced section — SegmentedControl (transport) + KeyValueRow (env vars).
//!
//! M8: Exercise the interactive areas of both new components.
//! SegmentedControl: 3-option transport picker wiring state changes.
//! KeyValueRow: dynamic env-var list with add/remove via stable row ids.

use gpui::{AppContext, ClickEvent, FontWeight, IntoElement, ParentElement, SharedString, Styled};
use gpui_component::input::InputState;
use gpui_component::scroll::ScrollableElement;
use gpui_component::{ActiveTheme as _, h_flex, v_flex};
use stand_in_mcp_explorer_ds::core::ToggleLink;
use stand_in_mcp_explorer_ds::core::button::ClickHandler;
use stand_in_mcp_explorer_ds::forms::{KeyValueRow, SegmentedControl};
use stand_in_mcp_explorer_ds::theme::typography;

use super::util::{px, section_body, section_label};
use crate::shell::GalleryShell;

pub fn render_forms_advanced(
    _state: &str,
    _mode: &str,
    this: &GalleryShell,
    cx: &mut gpui::Context<GalleryShell>,
) -> gpui::AnyElement {
    let t = cx.theme().clone();
    let mono = t.mono_font_family.clone();

    // --- SegmentedControl: transport picker ---
    let transport_options: Vec<(SharedString, SharedString)> = vec![
        ("stdio".into(), "STDIO".into()),
        ("sse".into(), "SSE".into()),
        ("http".into(), "HTTP".into()),
    ];
    let sel = this.selected_transport;
    let transport_handlers: Vec<ClickHandler> = (0..3)
        .map(|i| {
            let h = cx.listener(move |this, _: &ClickEvent, _window, cx| {
                this.selected_transport = i;
                cx.notify();
            });
            Box::new(h) as ClickHandler
        })
        .collect();

    let transport_label = transport_options[sel].0.clone();

    // --- KeyValueRow: env vars ---
    let n_rows = this.env_var_keys.len();
    let mut rows = Vec::new();
    for i in 0..n_rows {
        let row_id = this.env_var_row_ids[i];
        let key = &this.env_var_keys[i];
        let val = &this.env_var_values[i];

        let remove_handler: ClickHandler =
            Box::new(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                // Find and remove by stable row id (not by index, which shifts).
                if let Some(pos) = this.env_var_row_ids.iter().position(|rid| *rid == row_id) {
                    this.env_var_row_ids.remove(pos);
                    this.env_var_keys.remove(pos);
                    this.env_var_values.remove(pos);
                }
                cx.notify();
            }));

        rows.push(
            // w_full so the row receives the demo width (a hugging wrapper
            // starves flex_1 fields into min-content collapse).
            h_flex().w_full().mb(px(6.)).child(
                KeyValueRow::new(key, val)
                    .id(("kv-row", row_id))
                    .on_remove(remove_handler),
            ),
        );
    }

    v_flex()
        .flex_1()
        .min_w(px(0.))
        .h_full()
        .overflow_y_scrollbar()
        .child(
            section_body()
                // ---- SegmentedControl demo ----
                .child(section_label("SegmentedControl — transport", &t, &mono))
                .child(caption_row(
                    &t,
                    &mono,
                    &format!("Selected: {}", transport_label),
                ))
                .child(
                    h_flex().px(px(typography::FS_LG)).py_2().w(px(320.)).child(
                        SegmentedControl::new("seg-transport", transport_options, sel)
                            .handlers(transport_handlers),
                    ),
                )
                // ---- KeyValueRow demo ----
                .child(section_label("KeyValueRow — env vars", &t, &mono))
                .child(
                    // 304px = canon sidebar width, the component's real home.
                    v_flex()
                        .px(px(typography::FS_LG))
                        .py_2()
                        .w(px(304.))
                        .children(rows)
                        .child(
                            ToggleLink::new("add-env-var", "+ adicionar variável").on_click(
                                cx.listener({
                                    move |this, _: &ClickEvent, window, cx| {
                                        let id = this.next_env_row_id;
                                        this.next_env_row_id += 1;
                                        this.env_var_row_ids.push(id);
                                        this.env_var_keys.push(cx.new(|cx| {
                                            InputState::new(window, cx).placeholder("CHAVE")
                                        }));
                                        this.env_var_values.push(cx.new(|cx| {
                                            InputState::new(window, cx).placeholder("valor")
                                        }));
                                        cx.notify();
                                    }
                                }),
                            ),
                        ),
                ),
        )
        .into_any_element()
}

fn caption_row(
    t: &gpui_component::Theme,
    mono: &SharedString,
    text: &str,
) -> impl IntoElement + use<> {
    h_flex()
        .px(px(typography::FS_LG))
        .py_2()
        .gap_2()
        .items_center()
        .child(
            gpui::div()
                .text_size(px(typography::FS_XS))
                .text_color(t.muted_foreground)
                .font_family(mono.clone())
                .font_weight(FontWeight::SEMIBOLD)
                .child(SharedString::from(text.to_string())),
        )
}
