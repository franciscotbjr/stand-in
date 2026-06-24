//! Navigation section — SectionLabel + CapChip + Topbar + Tabbar + SidebarShell.
//!
//! SectionLabel: with and without icon, proving automatic uppercase (caller
//! writes lowercase). CapChip: count variants ("6 tools", "3 resources") +
//! no-count variant ("Portugu\u{ea}s") — all in a `.caps`-style h_flex (gap 7).
//! Topbar + Tabbar: the canonical composition — StatusDot::On + title mono
//! 14 weight 600 + meta "STDIO \u{b7} v2026.4.1 \u{b7} 57ms" + CapChips +
//! ghost button in the right slot; Tabbar below with 3 tabs (Tools 6 /
//! Resources 4 / Hist\u{f3}rico no count), interactive tab switching (state
//! in GalleryShell + notify; eco "tab: tools" on the display).
//! SidebarShell (M12): canonical 3-zone shell — brand (Leaf icon + name/sub),
//! scroll body (2 sections: Servidores salvos / Conex\u{e3}o with Field +
//! Button primary block), footer (privacy callout). In a fixed-height box
//! proving body scroll.

use gpui::{ClickEvent, FontWeight, IntoElement, ParentElement, SharedString, Styled, div};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{ActiveTheme as _, h_flex, v_flex};
use stand_in_mcp_explorer_ds::core::button::{ButtonSize, ButtonVariant, ClickHandler};
use stand_in_mcp_explorer_ds::core::{Button, DotState, IconName};
use stand_in_mcp_explorer_ds::forms::Field;
use stand_in_mcp_explorer_ds::navigation::{
    CapChip, SectionLabel, SidebarShell, TabItem, Tabbar, Topbar,
};
use stand_in_mcp_explorer_ds::theme::typography;

use super::util::{px, section_body, section_label};
use crate::shell::GalleryShell;

pub fn render_navigation(
    _state: &str,
    _mode: &str,
    this: &GalleryShell,
    cx: &mut gpui::Context<GalleryShell>,
) -> gpui::AnyElement {
    let t = cx.theme().clone();
    let mono = t.mono_font_family.clone();
    let ext = cx
        .global::<stand_in_mcp_explorer_ds::theme::colors::JandiExt>()
        .clone();

    // --- Topbar + Tabbar composition ---
    let tab_items = vec![
        TabItem::new("tools", "Tools").icon(IconName::Tool).count(6),
        TabItem::new("resources", "Resources")
            .icon(IconName::Doc)
            .count(4),
        TabItem::new("history", "Hist\u{f3}rico").icon(IconName::History),
    ];
    let tab_handlers: Vec<Option<ClickHandler>> = (0..tab_items.len())
        .map(|i| {
            Some(
                Box::new(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                    this.active_tab = i;
                    cx.notify();
                })) as ClickHandler,
            )
        })
        .collect();

    let active_label = &tab_items[this.active_tab].label;

    // --- SidebarShell demo: fake saved-server rows ---
    let fake_server_rows: Vec<gpui::AnyElement> = vec![
        fake_server_row("server-filesystem", "STDIO", &t, &mono),
        fake_server_row("prod-postgres", "SSE", &t, &mono),
        fake_server_row("sandbox-api", "HTTP", &t, &mono),
    ];

    // --- SidebarShell: privacy footer callout ---
    let privacy_callout = v_flex()
        .px(px(10.))
        .py(px(8.))
        .gap(px(4.))
        .rounded(px(10.))
        .bg(ext.ok_dim)
        .border_1()
        .border_color(ext.ok_dim)
        .child(
            h_flex()
                .gap(px(6.))
                .items_center()
                .child(
                    stand_in_mcp_explorer_ds::core::icon::Icon::new(IconName::Lock)
                        .with_px(px(12.)),
                )
                .child(
                    div()
                        .text_size(px(typography::FS_XS))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(stand_in_mcp_explorer_ds::theme::palette::OK)
                        .child(SharedString::from("100% local")),
                ),
        )
        .child(
            div()
                .text_size(px(10.5))
                .text_color(t.muted_foreground)
                .child(SharedString::from(
                    "Dados processados na sua m\u{e1}quina. Nada \u{e9} enviado.",
                )),
        );

    // --- SidebarShell: full canon composition ---
    let sidebar_demo = SidebarShell::new()
        .brand_mark(
            stand_in_mcp_explorer_ds::core::icon::Icon::new(IconName::Leaf).with_px(px(18.)),
        )
        .brand_name("MCP Explorer")
        .brand_sub("MCP \u{b7} local-first")
        .children([
            // Section 1: Saved servers
            SectionLabel::new("Servidores salvos")
                .icon(IconName::Bolt)
                .into_any_element(),
            v_flex()
                .gap(px(8.))
                .children(fake_server_rows)
                .into_any_element(),
            // Section 2: Connection form
            SectionLabel::new("Conex\u{e3}o")
                .icon(IconName::Plug)
                .into_any_element(),
            this.cmd_input
                .as_ref()
                .map(|e| Field::new(e).into_any_element())
                .unwrap_or_else(|| div().into_any_element()),
            Button::new("Conectar")
                .variant(ButtonVariant::Primary)
                .block()
                .into_any_element(),
        ])
        .footer(privacy_callout);

    v_flex()
        .flex_1()
        .min_w(px(0.))
        .h_full()
        .overflow_y_scrollbar().child(section_body()
        // SectionLabel — with and without icon
        .child(section_label("SectionLabel", &t, &mono))
        .child(section_labels_demo(&t, &mono))
        // CapChip — count variants + no-count
        .child(section_label("CapChip", &t, &mono))
        .child(cap_chips_demo(&t, &mono))
        // Topbar + Tabbar — canonical composition
        .child(section_label("Topbar + Tabbar", &t, &mono))
        .child(
            div()
                .text_size(px(typography::FS_XS))
                .text_color(t.muted_foreground)
                .font_family(mono.clone())
                .px(px(typography::FS_LG))
                .py(px(4.))
                .child(SharedString::from(format!(
                    "composi\u{e7}\u{e3}o can\u{f4}nica   \u{b7}   tab: {}",
                    active_label
                ))),
        )
        .child(
            v_flex()
                .mt(px(8.))
                .mb(px(12.))
                .rounded(px(8.))
                .border_1()
                .border_color(t.border)
                .bg(t.background)
                .overflow_hidden()
                .child(
                    Topbar::new(
                        DotState::On,
                        "server-filesystem",
                        "STDIO \u{b7} v2026.4.1 \u{b7} 57ms",
                    )
                    .right_children([
                        CapChip::new("tools")
                            .count(6)
                            .icon(IconName::Tool)
                            .into_any_element(),
                        CapChip::new("resources")
                            .count(4)
                            .icon(IconName::Doc)
                            .into_any_element(),
                        Button::new("Modo guiado")
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::Sm)
                            .into_any_element(),
                    ]),
                )
                .child(
                    Tabbar::new("gallery-tabs", tab_items, this.active_tab).handlers(tab_handlers),
                ),
        )
        // SidebarShell — canonical 3-zone shell
        .child(section_label("SidebarShell — casca can\u{f4}nica", &t, &mono))
        .child(
            div()
                .text_size(px(typography::FS_XS))
                .text_color(t.muted_foreground)
                .font_family(mono.clone())
                .px(px(typography::FS_LG))
                .py(px(4.))
                .child(SharedString::from(
                    "3 zonas: brand fixo + corpo rol\u{e1}vel + rodap\u{e9} fixo. Caixa limitada a 480px p/ provar scroll.",
                )),
        )
        .child(
            div()
                .mt(px(8.))
                .mb(px(24.))
                .w(px(304.))
                .h(px(480.))
                .rounded(px(8.))
                .border_1()
                .border_color(t.border)
                .overflow_hidden()
                .child(sidebar_demo),
        ))
        .into_any_element()
}

// ---------------------------------------------------------------------------
// SectionLabel demo
// ---------------------------------------------------------------------------

fn section_labels_demo(t: &gpui_component::Theme, mono: &SharedString) -> impl IntoElement + use<> {
    v_flex()
        .px(px(typography::FS_LG))
        .py_2()
        .gap(px(12.))
        .child(caption(
            "with icon + text (caller writes lowercase)",
            t,
            mono,
        ))
        .child(
            h_flex()
                .gap(px(7.))
                .child(SectionLabel::new("Servidores salvos").icon(IconName::Bolt)),
        )
        .child(caption("without icon", t, mono))
        .child(h_flex().gap(px(7.)).child(SectionLabel::new("Transporte")))
}

// ---------------------------------------------------------------------------
// CapChip demo
// ---------------------------------------------------------------------------

fn cap_chips_demo(t: &gpui_component::Theme, mono: &SharedString) -> impl IntoElement + use<> {
    v_flex()
        .px(px(typography::FS_LG))
        .py_2()
        .gap(px(12.))
        .child(caption("count + label (the .caps row, gap 7)", t, mono))
        .child(h_flex().gap(px(7.)).children([
            CapChip::new("tools").count(6).icon(IconName::Tool),
            CapChip::new("resources").count(3).icon(IconName::Doc),
        ]))
        .child(caption("no count — label only", t, mono))
        .child(
            h_flex()
                .gap(px(7.))
                .child(CapChip::new("Portugu\u{ea}s").icon(IconName::Globe)),
        )
}

// ---------------------------------------------------------------------------
// Fake server row (for SidebarShell demo)
// ---------------------------------------------------------------------------

fn fake_server_row(
    name: &str,
    transport: &str,
    t: &gpui_component::Theme,
    mono: &SharedString,
) -> gpui::AnyElement {
    h_flex()
        .gap(px(6.))
        .items_center()
        .child(
            div()
                .text_size(px(typography::FS_SM))
                .text_color(t.foreground)
                .font_weight(FontWeight::MEDIUM)
                .child(SharedString::from(name)),
        )
        .child(
            div()
                .text_size(px(10.))
                .text_color(t.muted_foreground)
                .font_family(mono.clone())
                .child(SharedString::from(transport)),
        )
        .into_any_element()
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn caption(text: &str, t: &gpui_component::Theme, mono: &SharedString) -> impl IntoElement + use<> {
    div()
        .text_size(px(10.))
        .text_color(t.muted_foreground)
        .font_family(mono.clone())
        .font_weight(FontWeight::NORMAL)
        .child(SharedString::from(text))
}
