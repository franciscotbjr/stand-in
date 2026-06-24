//! Badge + CopyButton + ToggleLink section — mirrors the core card rows.
//!
//! Renders: 5 badge examples (the canon's demo values: "leitura", "escrita",
//! "text/plain", "stdio", "assistant"), a live CopyButton (copies a demo JSON
//! value — the "copied" visual state is the functional gate), and a ToggleLink
//! ("+ adicionar variável" from the card) that ADDS a visible demo row per
//! click (and bumps the GlobalDemoClicks counter) — a click with no visible
//! result reads as a façade (human conference finding, 025 post-DONE).

use gpui::{FontWeight, IntoElement, ParentElement, SharedString, Styled, div};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{ActiveTheme as _, h_flex, v_flex};
use stand_in_mcp_explorer_ds::core::{Badge, BadgeKind, CopyButton, IconName, ToggleLink};
use stand_in_mcp_explorer_ds::theme::typography;

use super::util::{px, section_body, section_label};
use crate::shell::{GalleryShell, GlobalDemoClicks};

pub fn render_badges(
    _state: &str,
    _mode: &str,
    this: &GalleryShell,
    cx: &mut gpui::Context<GalleryShell>,
) -> gpui::AnyElement {
    let t = cx.theme().clone();
    let mono = t.mono_font_family.clone();
    let clicks = cx.global::<GlobalDemoClicks>().load();
    let vars_added = this.demo_vars_added;

    v_flex()
        .flex_1()
        .min_w(px(0.))
        .h_full()
        .overflow_y_scrollbar()
        .child(
            section_body()
                // Clicks counter
                .child(section_label("Clicks counter (functional gate)", &t, &mono))
                .child(clicks_display(clicks, &t, &mono))
                // Badge — 5 kinds
                .child(section_label(
                    "Badge \u{b7} read \u{b7} write \u{b7} mime \u{b7} muted \u{b7} role",
                    &t,
                    &mono,
                ))
                .child(badge_row())
                .child(badge_row_with_icon())
                // CopyButton
                .child(section_label("CopyButton (live)", &t, &mono))
                .child(copy_button_row())
                // ToggleLink
                .child(section_label("ToggleLink (adds a demo row)", &t, &mono))
                .child(toggle_link_row(vars_added, &t, &mono, cx)),
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

// ---------------------------------------------------------------------------
// Badge rows
// ---------------------------------------------------------------------------

fn badge_row() -> impl IntoElement {
    h_flex()
        .px(px(typography::FS_LG))
        .py_2()
        .gap_3()
        .items_center()
        .child(Badge::new("leitura", BadgeKind::Read).id("badge-read"))
        .child(Badge::new("escrita", BadgeKind::Write).id("badge-write"))
        .child(
            Badge::new("text/plain", BadgeKind::Mime)
                .icon(IconName::File)
                .id("badge-mime"),
        )
        .child(Badge::new("stdio", BadgeKind::Muted).id("badge-muted"))
        .child(
            Badge::new("assistant", BadgeKind::Role)
                .icon(IconName::Chat)
                .id("badge-role"),
        )
}

fn badge_row_with_icon() -> impl IntoElement {
    h_flex()
        .px(px(typography::FS_LG))
        .py_2()
        .gap_3()
        .items_center()
        .child(Badge::new("read", BadgeKind::Read).id("badge-read-icon"))
        .child(
            Badge::new("write", BadgeKind::Write)
                .icon(IconName::Bolt)
                .id("badge-write-icon"),
        )
}

// ---------------------------------------------------------------------------
// CopyButton row
// ---------------------------------------------------------------------------

fn copy_button_row() -> impl IntoElement {
    let demo_value = r#"{"name":"stand-in","version":"0.0.4"}"#;
    h_flex()
        .px(px(typography::FS_LG))
        .py_2()
        .gap_3()
        .items_center()
        .child(
            CopyButton::new("copy-btn-demo", demo_value)
                .label("Copiar JSON")
                .copied_label("Copiado"),
        )
}

// ---------------------------------------------------------------------------
// ToggleLink row
// ---------------------------------------------------------------------------

fn toggle_link_row(
    vars_added: usize,
    t: &gpui_component::Theme,
    mono: &SharedString,
    cx: &mut gpui::Context<GalleryShell>,
) -> impl IntoElement + use<> {
    // Each click adds a visible demo row above the link — the click must
    // produce a result the eye can see, not only the counter.
    let mut col = v_flex().px(px(typography::FS_LG)).py_2().gap(px(6.));

    for i in 0..vars_added {
        col = col.child(
            div()
                .text_size(px(typography::FS_SM))
                .font_family(mono.clone())
                .text_color(t.secondary_foreground)
                .child(SharedString::from(format!("VAR_{}=valor", i + 1))),
        );
    }

    col.child(h_flex().gap_3().items_center().child(
        ToggleLink::new("add-var-gallery", "+ adicionar vari\u{00e1}vel").on_click(cx.listener(
            |this, _: &gpui::ClickEvent, _window, cx| {
                this.demo_vars_added += 1;
                cx.global::<GlobalDemoClicks>().inc();
                cx.notify();
            },
        )),
    ))
}
