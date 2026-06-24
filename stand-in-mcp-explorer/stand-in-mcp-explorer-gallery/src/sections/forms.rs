//! Forms section — Field (single-line · long/textarea · invalid · mono/sans)
//! + typing proof (live echo of typed content).
//!
//! Mirrors the Field rows of `forms/forms.card.html`. The live echo caption
//! "value: `typed`" proves real input (gate funcional — 022 BUG-5).
//! In capture mode the command input is seeded deterministically ("npx").

use gpui::{Entity, FontWeight, IntoElement, ParentElement, SharedString, Styled, div};
use gpui_component::input::InputState;
use gpui_component::scroll::ScrollableElement;
use gpui_component::{ActiveTheme as _, h_flex, v_flex};
use stand_in_mcp_explorer_ds::forms::Field;
use stand_in_mcp_explorer_ds::theme::typography;

use super::util::{px, section_body, section_label};
use crate::shell::GalleryShell;

pub fn render_forms(
    _state: &str,
    _mode: &str,
    this: &GalleryShell,
    cx: &mut gpui::Context<GalleryShell>,
) -> gpui::AnyElement {
    let t = cx.theme().clone();
    let mono = t.mono_font_family.clone();

    // Unwrap — ensure_inputs was called before the first render.
    let cmd = this.cmd_input.as_ref().unwrap().clone();
    let args = this.args_input.as_ref().unwrap().clone();
    let err = this.err_input.as_ref().unwrap().clone();
    let prose = this.prose_input.as_ref().unwrap().clone();

    let cmd_val = read_cmd_val(this, cx);

    v_flex()
        .flex_1()
        .min_w(px(0.))
        .h_full()
        .overflow_y_scrollbar()
        .child(
            section_body()
                .child(section_label("Field — typing proof", &t, &mono))
                .child(echo_row(&t, &mono, &cmd_val))
                .child(section_label("Field — single-line", &t, &mono))
                .child(single_row(&cmd))
                .child(section_label("Field — long / textarea", &t, &mono))
                .child(long_row(&args))
                .child(section_label("Field — invalid", &t, &mono))
                .child(invalid_row(&err))
                .child(section_label("Field — mono(false) · sans prosa", &t, &mono))
                .child(prose_row(&prose)),
        )
        .into_any_element()
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn read_cmd_val(this: &GalleryShell, cx: &mut gpui::Context<GalleryShell>) -> SharedString {
    this.cmd_input
        .as_ref()
        .map(|e| e.read(cx).value())
        .unwrap_or_default()
}

fn echo_row(t: &gpui_component::Theme, mono: &SharedString, val: &str) -> impl IntoElement + use<> {
    h_flex()
        .px(px(typography::FS_LG))
        .py_2()
        .gap_2()
        .items_center()
        .child(
            div()
                .text_size(px(typography::FS_XS))
                .text_color(t.muted_foreground)
                .font_family(mono.clone())
                .child("value:"),
        )
        .child(
            div()
                .text_size(px(typography::FS_MD))
                .text_color(t.foreground)
                .font_family(mono.clone())
                .font_weight(FontWeight::BOLD)
                .child(SharedString::from(val.to_string())),
        )
}

fn single_row(cmd: &Entity<InputState>) -> impl IntoElement + use<> {
    v_flex()
        .px(px(typography::FS_LG))
        .py_2()
        .w(px(320.))
        .child(Field::new(cmd).label("Command").id("field-cmd"))
}

fn long_row(args: &Entity<InputState>) -> impl IntoElement + use<> {
    v_flex().px(px(typography::FS_LG)).py_2().w(px(320.)).child(
        Field::new(args)
            .label("Arguments")
            .long()
            .hint("Separated by spaces; use quotes for composite values")
            .id("field-args"),
    )
}

fn invalid_row(err: &Entity<InputState>) -> impl IntoElement + use<> {
    v_flex().px(px(typography::FS_LG)).py_2().w(px(320.)).child(
        Field::new(err)
            .label("Port")
            .invalid()
            .hint("Port must be between 1 and 65535")
            .id("field-err"),
    )
}

fn prose_row(prose: &Entity<InputState>) -> impl IntoElement + use<> {
    v_flex().px(px(typography::FS_LG)).py_2().w(px(320.)).child(
        Field::new(prose)
            .label("Description")
            .mono(false)
            .long()
            .hint("Free-form human-readable description")
            .id("field-prose"),
    )
}
