//! Select section — language-picker demo proving the open/select/close cycle.
//!
//! M9: O-003 vindicated — a dropdown language selector that opens, picks an
//! option, closes, and echoes the result.

use gpui::{FontWeight, IntoElement, ParentElement, SharedString, Styled};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{ActiveTheme as _, h_flex, v_flex};
use stand_in_mcp_explorer_ds::forms::Select;
use stand_in_mcp_explorer_ds::theme::typography;

use super::util::{px, section_body, section_label};
use crate::shell::GalleryShell;

pub fn render_select(
    _state: &str,
    _mode: &str,
    this: &GalleryShell,
    cx: &mut gpui::Context<GalleryShell>,
) -> gpui::AnyElement {
    let t = cx.theme().clone();
    let mono = t.mono_font_family.clone();

    let lang_options: Vec<(SharedString, SharedString)> = vec![
        ("pt".into(), "Portugu\u{ea}s".into()),
        ("en".into(), "English".into()),
        ("es".into(), "Espa\u{f1}ol".into()),
    ];

    let current_value = lang_options[this.selected_lang].0.clone();

    let entity = cx.entity().downgrade();

    v_flex()
        .flex_1()
        .min_w(px(0.0))
        .h_full()
        .overflow_y_scrollbar()
        .child(
            section_body()
                .child(section_label("Select — language picker", &t, &mono))
                .child(
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
                                .child(SharedString::from(format!("lang: {}", current_value))),
                        ),
                )
                .child(
                    h_flex()
                        .px(px(typography::FS_LG))
                        .py_2()
                        .child(div().w(px(220.0)).child(
                            Select::new("select-lang", lang_options, this.selected_lang).on_change(
                                move |ix, _value, _window, app| {
                                    let _ = entity.update(app, |gallery, entity_cx| {
                                        gallery.selected_lang = ix;
                                        entity_cx.notify();
                                    });
                                },
                            ),
                        )),
                ),
        )
        .into_any_element()
}

fn div() -> gpui::Div {
    gpui::div()
}
