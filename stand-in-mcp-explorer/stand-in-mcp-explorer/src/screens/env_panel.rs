//! Environment variables panel — floating overlay to the right.
//!
//! Mirrors `auth_panel.rs` but simpler: no Select, no secret fields, no OAuth.
//! The overlay is a transparent click-catcher (no scrim — floating panel,
//! app remains visible) + a card anchored to the right. The card uses `.occlude()`
//! so clicks inside don't bubble to the catcher. Close: X button / Save button /
//! click outside. Body reuses `render_env_rows`.

use crate::app::i18n::{Lang, tr};
use crate::bars::sidebar::env_rows::render_env_rows;
use crate::bars::sidebar::sidebar_state::SidebarState;
use gpui::{
    App, FontWeight, InteractiveElement, IntoElement, MouseButton, MouseDownEvent, ParentElement,
    Styled, Window, div, prelude::FluentBuilder, px, relative,
};
use gpui_component::ActiveTheme as _;
use gpui_component::scroll::ScrollableElement as _;
use stand_in_mcp_explorer_ds::core::button::{Button, ButtonVariant, ClickHandler};
use stand_in_mcp_explorer_ds::core::icon::IconName;
use stand_in_mcp_explorer_ds::core::icon_button::IconButton;
use stand_in_mcp_explorer_ds::theme::density::RADIUS_CARD;

pub use crate::screens::auth_panel::ClickOutsideHandler;

#[allow(clippy::too_many_arguments)]
pub fn render_env_panel(
    state: &SidebarState,
    lang: Lang,
    on_add: Option<ClickHandler>,
    on_remove: Vec<ClickHandler>,
    on_close_x: ClickHandler,
    on_close_save: ClickHandler,
    on_click_outside: ClickOutsideHandler,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let colors = cx.theme().colors;

    div()
        .id("env-overlay")
        .absolute()
        .inset_0()
        .flex()
        .justify_end()
        .items_start()
        .on_mouse_down(
            MouseButton::Left,
            move |_ev: &MouseDownEvent, window: &mut Window, app: &mut App| {
                on_click_outside(window, app);
            },
        )
        .child(
            div().id("env-panel-wrapper").pt(px(72.)).pr(px(12.)).child(
                div()
                    .id("env-panel")
                    .occlude()
                    .flex()
                    .flex_col()
                    .w(px(320.))
                    .max_h(relative(0.85))
                    .bg(colors.secondary)
                    .border_1()
                    .border_color(colors.border)
                    .rounded(px(RADIUS_CARD))
                    .shadow_lg()
                    .child(env_header(lang, on_close_x, &colors))
                    .child(env_body(
                        state, lang, on_add, on_remove, window, cx, &colors,
                    ))
                    .child(env_footer(lang, on_close_save, &colors)),
            ),
        )
}

fn env_header(
    lang: Lang,
    on_close: ClickHandler,
    colors: &gpui_component::ThemeColor,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .p(px(18.))
        .pb(px(14.))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(4.))
                .child(
                    div()
                        .text_size(px(15.))
                        .font_weight(FontWeight::BOLD)
                        .text_color(colors.foreground)
                        .child(tr("sidebar.envVars", lang)),
                )
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(colors.secondary_foreground)
                        .child(tr("sidebar.envSubtitle", lang)),
                ),
        )
        .child(
            IconButton::new(IconName::X, tr("tweaks.close", lang))
                .id("env-close")
                .on_click_boxed(on_close),
        )
}

fn env_body(
    state: &SidebarState,
    lang: Lang,
    on_add: Option<ClickHandler>,
    on_remove: Vec<ClickHandler>,
    window: &mut Window,
    cx: &mut App,
    colors: &gpui_component::ThemeColor,
) -> impl IntoElement {
    let empty_note = if state.env_rows.is_empty() {
        Some(
            div()
                .text_size(px(11.5))
                .text_color(colors.muted_foreground)
                .line_height(relative(1.4))
                .pb(px(8.))
                .child(tr("sidebar.envEmpty", lang)),
        )
    } else {
        None
    };

    div().flex_1().overflow_hidden().child(
        div()
            .size_full()
            .overflow_y_scrollbar()
            .px(px(18.))
            .when_some(empty_note, |el, note| el.child(note))
            .child(render_env_rows(state, lang, on_add, on_remove, window, cx)),
    )
}

fn env_footer(
    lang: Lang,
    on_close: ClickHandler,
    colors: &gpui_component::ThemeColor,
) -> impl IntoElement {
    div()
        .flex()
        .justify_end()
        .p(px(18.))
        .pt(px(14.))
        .border_t_1()
        .border_color(colors.border)
        .child(
            Button::new(tr("sidebar.envDone", lang))
                .variant(ButtonVariant::Primary)
                .id("env-save")
                .on_click(move |ev, w, cx| on_close(ev, w, cx)),
        )
}
