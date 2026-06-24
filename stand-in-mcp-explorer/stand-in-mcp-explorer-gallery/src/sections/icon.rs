//! Icon section — 22-glyph catalog matching `guidelines/icons`.
//!
//! Renders every glyph in a labelled grid plus a multi-size row showing all
//! four canonical sizes of one glyph.

use gpui::{IntoElement, ParentElement, SharedString, Styled, div};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{ActiveTheme as _, h_flex, v_flex};
use stand_in_mcp_explorer_ds::core::{Icon, IconName, IconSize};
use stand_in_mcp_explorer_ds::theme::typography;

use super::util::{px, section_body, section_label};
use crate::shell::GalleryShell;

pub fn render_icon(
    _state: &str,
    _mode: &str,
    _this: &GalleryShell,
    cx: &mut gpui::Context<GalleryShell>,
) -> gpui::AnyElement {
    let t = cx.theme().clone();
    let mono = t.mono_font_family.clone();

    v_flex()
        .flex_1()
        .min_w(px(0.))
        .h_full()
        .overflow_y_scrollbar()
        .child(
            section_body()
                .child(section_label("Icons (22 glyphs)", &t, &mono))
                .child(render_glyph_grid(&t, &mono))
                .child(section_label("Sizes — 12 · 14 · 15 · 28", &t, &mono))
                .child(render_size_row(&t, &mono)),
        )
        .into_any_element()
}

fn render_glyph_grid(t: &gpui_component::Theme, mono: &SharedString) -> impl IntoElement + use<> {
    h_flex()
        .px(px(typography::FS_LG))
        .gap_3()
        .py_2()
        .flex_wrap()
        .children(IconName::ALL.iter().map(|name| {
            let label = SharedString::from(name.as_str());
            v_flex()
                .w(px(80.))
                .gap_1()
                .items_center()
                .child(
                    div()
                        .size(px(24.))
                        .flex_none()
                        .text_color(t.foreground)
                        .child(Icon::new(*name)),
                )
                .child(
                    div()
                        .text_size(px(typography::FS_2XS))
                        .text_color(t.muted_foreground)
                        .font_family(mono.clone())
                        .child(label),
                )
        }))
}

fn render_size_row(t: &gpui_component::Theme, mono: &SharedString) -> impl IntoElement + use<> {
    let sizes: &[(IconSize, &str)] = &[
        (IconSize::Xs, "12"),
        (IconSize::Sm, "14"),
        (IconSize::Md, "15"),
        (IconSize::Lg, "28"),
    ];

    h_flex()
        .px(px(typography::FS_LG))
        .gap_4()
        .py_2()
        .items_center()
        .children(sizes.iter().map(|(size, label)| {
            let lbl = SharedString::from(*label);
            v_flex()
                .gap_1()
                .items_center()
                .child(
                    div()
                        .text_size(px(typography::FS_2XS))
                        .text_color(t.muted_foreground)
                        .font_family(mono.clone())
                        .child(lbl),
                )
                .child(
                    div()
                        .text_color(t.foreground)
                        .child(Icon::new(IconName::Play).size(*size)),
                )
        }))
}
