//! Environment variables trigger — collapsed row in the sidebar that opens the env panel.
//!
//! Mirrors `auth_trigger.rs` but inverted: active on STDIO (not HTTP).
//! Anatomy: a single `h_flex` row (height ~36, `w_full`, `RADIUS_INPUT`,
//! bg `secondary`, border `border`): Bolt icon + label left; CountPill (if >0)
//! + summary + Chevron right. Hover: border → `border_2`. Cursor pointer.

use crate::app::events::Transport;
use crate::app::i18n::{Lang, tr};
use gpui::{
    App, FontWeight, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement,
    Styled, Window, px,
};
use gpui_component::ActiveTheme as _;
use gpui_component::h_flex;
use stand_in_mcp_explorer_ds::core::button::ClickHandler;
use stand_in_mcp_explorer_ds::core::count_pill::CountPill;
use stand_in_mcp_explorer_ds::core::icon::{Icon, IconName, IconSize};
use stand_in_mcp_explorer_ds::theme::colors::JandiExt;
use stand_in_mcp_explorer_ds::theme::density::RADIUS_INPUT;

pub fn render_env_trigger(
    count: usize,
    transport: Transport,
    lang: Lang,
    on_open: Option<ClickHandler>,
    _window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let colors = cx.theme().colors;
    let ext = cx.global::<JandiExt>().clone();
    let is_stdio = matches!(transport, Transport::Stdio);

    if !is_stdio {
        return gpui::div().into_any_element();
    }

    let mut row = h_flex()
        .id("env-trigger")
        .w_full()
        .h(px(36.))
        .items_center()
        .justify_between()
        .bg(colors.secondary)
        .border_1()
        .border_color(colors.border)
        .rounded(px(RADIUS_INPUT))
        .px(px(11.))
        .gap(px(8.))
        .text_size(px(12.))
        .text_color(colors.foreground)
        .cursor_pointer()
        .hover(|h| h.border_color(ext.border_2));

    row = row.child(
        h_flex()
            .items_center()
            .gap(px(7.))
            .child(
                Icon::new(IconName::Bolt)
                    .size(IconSize::Xs)
                    .color(colors.foreground),
            )
            .child(
                gpui::div()
                    .text_size(px(12.))
                    .font_weight(FontWeight::MEDIUM)
                    .child(tr("sidebar.envVars", lang)),
            ),
    );

    let mut right = h_flex().items_center().gap(px(6.));

    // Count shown ONLY in the pill (bullet) when > 0 — the "N variável(is)" text
    // was redundant with the pill number (036 QA). Empty → the "nenhuma" label.
    if count > 0 {
        right = right.child(CountPill::new(count).id("env-count-pill"));
    } else {
        right = right.child(
            gpui::div()
                .text_size(px(12.))
                .text_color(colors.secondary_foreground)
                .child(tr("sidebar.envNone", lang)),
        );
    }
    right = right.child(
        Icon::new(IconName::Chevron)
            .size(IconSize::Xs)
            .color(colors.secondary_foreground),
    );

    row = row.child(right);

    if let Some(handler) = on_open {
        row = row.on_click(handler);
    }

    row.into_any_element()
}
