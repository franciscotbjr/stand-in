//! Indicators section — StatusDot (the 4-state connection vocabulary) and
//! Spinner, mirroring the StatusDot / spinner rows of `core/core.card.html`.
//!
//! The busy dot pulses (the only animated state) and the spinners rotate —
//! the section is the live proof of the DS motion contract.

use gpui::{FontWeight, IntoElement, ParentElement, SharedString, Styled, div};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{ActiveTheme as _, h_flex, v_flex};
use stand_in_mcp_explorer_ds::core::{DotState, Spinner, StatusDot};
use stand_in_mcp_explorer_ds::theme::typography;

use super::util::{px, section_body, section_label};
use crate::shell::GalleryShell;

pub fn render_indicators(
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
                .child(section_label(
                    "StatusDot — on · off · busy · err",
                    &t,
                    &mono,
                ))
                .child(dot_row(&t, &mono))
                .child(section_label("Connection state (dot + label)", &t, &mono))
                .child(conn_example(&t, &mono))
                .child(section_label("Spinner — 15px, inherits color", &t, &mono))
                .child(spinner_row(&t, &mono)),
        )
        .into_any_element()
}

fn caption(
    label: &str,
    t: &gpui_component::Theme,
    mono: &SharedString,
) -> impl IntoElement + use<> {
    div()
        .text_size(px(typography::FS_2XS))
        .text_color(t.muted_foreground)
        .font_family(mono.clone())
        .child(SharedString::from(label))
}

fn dot_row(t: &gpui_component::Theme, mono: &SharedString) -> impl IntoElement + use<> {
    let states: &[(DotState, &str)] = &[
        (DotState::On, "on"),
        (DotState::Off, "off"),
        (DotState::Busy, "busy"),
        (DotState::Err, "err"),
    ];
    h_flex()
        .px(px(typography::FS_LG))
        .py_2()
        .gap_4()
        .items_center()
        .children(states.iter().map(|(state, label)| {
            h_flex()
                .gap_2()
                .items_center()
                .child(StatusDot::new(*state))
                .child(caption(label, t, mono))
        }))
}

fn conn_example(t: &gpui_component::Theme, mono: &SharedString) -> impl IntoElement + use<> {
    // The StatusDot.prompt.md example: dot + server name + mono metadata.
    h_flex()
        .px(px(typography::FS_LG))
        .py_2()
        .gap_3()
        .items_center()
        .child(StatusDot::new(DotState::On))
        .child(
            v_flex()
                .gap_1()
                .child(
                    div()
                        .text_size(px(typography::FS_MD))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(t.foreground)
                        .font_family(mono.clone())
                        .child("server-filesystem"),
                )
                .child(
                    div()
                        .text_size(px(typography::FS_XS))
                        .text_color(t.muted_foreground)
                        .font_family(mono.clone())
                        .child("STDIO \u{00B7} v2026.4.1 \u{00B7} 57ms"),
                ),
        )
}

fn spinner_row(t: &gpui_component::Theme, mono: &SharedString) -> impl IntoElement + use<> {
    h_flex()
        .px(px(typography::FS_LG))
        .py_2()
        .gap_6()
        .items_center()
        // Inherits the wrapping text colour (the default usage).
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(
                    div()
                        .text_color(t.foreground)
                        .child(Spinner::new().id("spinner-foreground")),
                )
                .child(caption("inherited (foreground)", t, mono)),
        )
        // Explicit colour override.
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(Spinner::new().color(t.primary).id("spinner-primary"))
                .child(caption("explicit (primary)", t, mono)),
        )
        // The canonical in-progress usage: spinner + action label + ellipsis.
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .text_color(t.muted_foreground)
                .child(Spinner::new().id("spinner-busy"))
                .child(
                    div()
                        .text_size(px(typography::FS_MD))
                        .child("Conectando\u{2026}"),
                ),
        )
}
