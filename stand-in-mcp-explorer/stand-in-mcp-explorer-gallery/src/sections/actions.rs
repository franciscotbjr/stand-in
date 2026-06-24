//! Actions section — Button (primary/ghost/danger × md/sm, block, disabled,
//! loading) + IconButton (x/refresh/copy), mirroring the Button rows of
//! `core/core.card.html`.
//!
//! The click counter ("clicks: N") is the functional gate proof: every demo
//! button attaches a `cx.listener` directly on the `Button::on_click` pipeline,
//! not on a wrapper div. Disabled buttons also get the handler (the F1 guard
//! prevents the fire — negative proof). Loading buttons skip the handler (they
//! are implicitly disabled). In capture mode the counter renders
//! deterministically (0).

use gpui::{ClickEvent, FontWeight, IntoElement, ParentElement, SharedString, Styled, div};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{ActiveTheme as _, h_flex, v_flex};
use stand_in_mcp_explorer_ds::core::{Button, ButtonSize, ButtonVariant, IconButton, IconName};
use stand_in_mcp_explorer_ds::theme::typography;

use super::util::{px, section_body, section_label};
use crate::shell::{GalleryShell, GlobalDemoClicks};

pub fn render_actions(
    _state: &str,
    _mode: &str,
    _this: &GalleryShell,
    cx: &mut gpui::Context<GalleryShell>,
) -> gpui::AnyElement {
    let t = cx.theme().clone();
    let mono = t.mono_font_family.clone();
    let clicks = cx.global::<GlobalDemoClicks>().load();

    v_flex()
        .flex_1()
        .min_w(px(0.))
        .h_full()
        .overflow_y_scrollbar()
        .child(
            section_body()
                .child(section_label("Clicks counter (functional gate)", &t, &mono))
                .child(clicks_display(clicks, &t, &mono))
                .child(section_label(
                    "Button — primary · ghost · danger × md",
                    &t,
                    &mono,
                ))
                .child(variant_row_md(cx))
                .child(section_label("Button — sm", &t, &mono))
                .child(variant_row_sm(cx))
                .child(section_label("Button — block", &t, &mono))
                .child(block_row(cx))
                .child(section_label("Button — disabled (primary md)", &t, &mono))
                .child(disabled_row(cx))
                .child(section_label("Button — loading (ghost md)", &t, &mono))
                .child(loading_row())
                .child(section_label("IconButton — x · refresh · copy", &t, &mono))
                .child(icon_button_row(cx)),
        )
        .into_any_element()
}

// ---------------------------------------------------------------------------
// Click counter display
// ---------------------------------------------------------------------------

fn clicks_display(
    n: usize,
    t: &gpui_component::Theme,
    mono: &SharedString,
) -> impl IntoElement + use<> {
    h_flex()
        .px(px(typography::FS_LG))
        .py_2()
        .gap_2()
        .items_center()
        .child(caption("clicks:", t, mono))
        .child(
            div()
                .text_size(px(typography::FS_MD))
                .text_color(t.foreground)
                .font_family(mono.clone())
                .font_weight(FontWeight::BOLD)
                .child(SharedString::from(n.to_string())),
        )
}

// ---------------------------------------------------------------------------
// Click helper — returns a closure suitable for cx.listener
// ---------------------------------------------------------------------------

#[allow(clippy::type_complexity)]
fn clicker(
    cx: &mut gpui::Context<GalleryShell>,
) -> Box<dyn Fn(&ClickEvent, &mut gpui::Window, &mut gpui::App)> {
    Box::new(cx.listener(|_this, _: &ClickEvent, _window, cx| {
        cx.global::<GlobalDemoClicks>().inc();
        cx.notify();
    }))
}

// ---------------------------------------------------------------------------
// Row helpers
// ---------------------------------------------------------------------------

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

fn btn_md(
    variant: ButtonVariant,
    label: &str,
    cx: &mut gpui::Context<GalleryShell>,
) -> impl IntoElement + use<> {
    let name = format!("btn-{:?}-md", variant).to_lowercase();
    let handler = clicker(cx);
    Button::new(label)
        .variant(variant)
        .size(ButtonSize::Md)
        .id(SharedString::from(name.clone()))
        .on_click(handler)
}

fn btn_sm(
    variant: ButtonVariant,
    label: &str,
    cx: &mut gpui::Context<GalleryShell>,
) -> impl IntoElement + use<> {
    let name = format!("btn-{:?}-sm", variant).to_lowercase();
    let handler = clicker(cx);
    Button::new(label)
        .variant(variant)
        .size(ButtonSize::Sm)
        .id(SharedString::from(name.clone()))
        .on_click(handler)
}

// ---------------------------------------------------------------------------
// Rows
// ---------------------------------------------------------------------------

fn variant_row_md(cx: &mut gpui::Context<GalleryShell>) -> impl IntoElement + use<> {
    h_flex()
        .px(px(typography::FS_LG))
        .py_2()
        .gap_3()
        .items_center()
        .child(btn_md(ButtonVariant::Primary, "Primary", cx))
        .child(btn_md(ButtonVariant::Ghost, "Ghost", cx))
        .child(btn_md(ButtonVariant::Danger, "Danger", cx))
        .child(
            Button::new("Conectar")
                .variant(ButtonVariant::Primary)
                .size(ButtonSize::Md)
                .icon(IconName::Play)
                .id("btn-primary-icon-md")
                .on_click(clicker(cx)),
        )
        .child(
            Button::new("Desconectar")
                .variant(ButtonVariant::Danger)
                .size(ButtonSize::Md)
                .icon(IconName::X)
                .id("btn-danger-icon-md")
                .on_click(clicker(cx)),
        )
}

fn variant_row_sm(cx: &mut gpui::Context<GalleryShell>) -> impl IntoElement + use<> {
    h_flex()
        .px(px(typography::FS_LG))
        .py_2()
        .gap_3()
        .items_center()
        .child(btn_sm(ButtonVariant::Primary, "Primary sm", cx))
        .child(btn_sm(ButtonVariant::Ghost, "Ghost sm", cx))
        .child(btn_sm(ButtonVariant::Danger, "Danger sm", cx))
}

fn block_row(cx: &mut gpui::Context<GalleryShell>) -> impl IntoElement + use<> {
    v_flex().px(px(typography::FS_LG)).py_2().gap_2().child(
        v_flex()
            .w(px(300.))
            .gap_2()
            .child(
                Button::new("Primary block")
                    .variant(ButtonVariant::Primary)
                    .block()
                    .id("btn-primary-block")
                    .on_click(clicker(cx)),
            )
            .child(
                Button::new("Ghost block")
                    .variant(ButtonVariant::Ghost)
                    .block()
                    .id("btn-ghost-block")
                    .on_click(clicker(cx)),
            ),
    )
}

fn disabled_row(cx: &mut gpui::Context<GalleryShell>) -> impl IntoElement + use<> {
    h_flex()
        .px(px(typography::FS_LG))
        .py_2()
        .gap_3()
        .items_center()
        .child(
            Button::new("Disabled")
                .variant(ButtonVariant::Primary)
                .disabled()
                .id("btn-primary-disabled")
                .on_click(clicker(cx)),
        )
        .child(
            Button::new("Disabled ghost")
                .variant(ButtonVariant::Ghost)
                .disabled()
                .id("btn-ghost-disabled")
                .on_click(clicker(cx)),
        )
}

fn loading_row() -> impl IntoElement {
    h_flex()
        .px(px(typography::FS_LG))
        .py_2()
        .gap_3()
        .items_center()
        .child(
            Button::new("Carregando\u{2026}")
                .variant(ButtonVariant::Ghost)
                .loading()
                .id("btn-ghost-loading"),
        )
        .child(
            Button::new("Enviando\u{2026}")
                .variant(ButtonVariant::Primary)
                .loading()
                .id("btn-primary-loading"),
        )
}

fn icon_button_row(cx: &mut gpui::Context<GalleryShell>) -> impl IntoElement + use<> {
    h_flex()
        .px(px(typography::FS_LG))
        .py_2()
        .gap_3()
        .items_center()
        .child(
            IconButton::new(IconName::X, "Remover")
                .id("iconbtn-x")
                .on_click(clicker(cx)),
        )
        .child(
            IconButton::new(IconName::Refresh, "Reconectar")
                .id("iconbtn-refresh")
                .on_click(clicker(cx)),
        )
        .child(
            IconButton::new(IconName::Copy, "Copiar")
                .id("iconbtn-copy")
                .on_click(clicker(cx)),
        )
}
