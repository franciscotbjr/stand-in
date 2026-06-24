//! Authorization trigger — collapsed row in the sidebar that opens the auth panel.
//!
//! Anatomy: a single `h_flex` row (height ~36, `w_full`, `RADIUS_INPUT`,
//! bg `secondary`, border `border`): Lock icon + label left; method label +
//! Chevron right. Hover: border → `border_2`. Cursor pointer.
//!
//! When transport is STDIO (D0): no click handler, `cursor_default`,
//! `muted_foreground` content, and a note below (`auth.httpOnly`).

use crate::app::events::Transport;
use crate::app::i18n::{Lang, tr};
use gpui::{
    App, FontWeight, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement,
    Styled, Window, px,
};
use gpui_component::ActiveTheme as _;
use gpui_component::h_flex;
use stand_in_mcp_explorer_ds::core::button::ClickHandler;
use stand_in_mcp_explorer_ds::core::icon::{Icon, IconName, IconSize};
use stand_in_mcp_explorer_ds::theme::colors::JandiExt;
use stand_in_mcp_explorer_ds::theme::density::RADIUS_INPUT;

use super::auth_state::AuthMethod;

pub fn render_auth_trigger(
    method: AuthMethod,
    transport: Transport,
    lang: Lang,
    on_open: Option<ClickHandler>,
    _window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let colors = cx.theme().colors;
    let ext = cx.global::<JandiExt>().clone();
    let is_http = matches!(transport, Transport::Http);
    let fg = if is_http {
        colors.foreground
    } else {
        colors.muted_foreground
    };

    let mut col = gpui_component::v_flex()
        .id("auth-trigger-block")
        .w_full()
        .gap(px(4.));

    let mut row = h_flex()
        .id("auth-trigger")
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
        .text_size(px(12.)) // fs-sm for the label
        .text_color(fg);

    // Left: Lock icon + title label
    row = row.child(
        h_flex()
            .items_center()
            .gap(px(7.))
            .child(Icon::new(IconName::Lock).size(IconSize::Xs).color(fg))
            .child(
                gpui::div()
                    .text_size(px(12.))
                    .font_weight(FontWeight::MEDIUM)
                    .child(tr("auth.title", lang)),
            ),
    );

    // Right: method label + Chevron
    row = row.child(
        h_flex()
            .items_center()
            .gap(px(6.))
            .child(
                gpui::div()
                    .text_size(px(12.))
                    .text_color(colors.secondary_foreground)
                    .child(tr(method.label_key(), lang)),
            )
            .child(
                Icon::new(IconName::Chevron)
                    .size(IconSize::Xs)
                    .color(colors.secondary_foreground),
            ),
    );

    if is_http {
        row = row.cursor_pointer().hover(|h| h.border_color(ext.border_2));

        if let Some(handler) = on_open {
            row = row.on_click(handler);
        }
    } else {
        row = row.cursor_default();
    }

    col = col.child(row);

    // STDIO note
    if !is_http {
        col = col.child(
            gpui::div()
                .text_size(px(11.5))
                .text_color(colors.muted_foreground)
                .child(tr("auth.httpOnly", lang)),
        );
    }

    col
}
