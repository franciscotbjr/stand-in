//! Connect / Disconnect button block — dispatches `UiCommand` via
//! the engine bridge (STDIO only in M5; remote transport disabled).

use crate::app::conn_state::ConnState;
use crate::app::i18n::{Lang, tr};
use gpui::{App, IntoElement, ParentElement, Styled, Window};
use gpui_component::v_flex;
use stand_in_mcp_explorer_ds::core::button::{Button, ButtonVariant, ClickHandler};
use stand_in_mcp_explorer_ds::core::icon::IconName;

pub fn render_connect_block(
    conn_state: &ConnState,
    _transport: crate::app::events::Transport,
    lang: Lang,
    on_connect: Option<ClickHandler>,
    on_disconnect: Option<ClickHandler>,
    _window: &mut Window,
    _cx: &mut App,
) -> impl IntoElement {
    match conn_state {
        ConnState::Disconnected => {
            let mut btn = Button::new(tr("sidebar.connect", lang))
                .variant(ButtonVariant::Primary)
                .block()
                .icon(IconName::Play)
                .id("connect-btn");

            if let Some(h) = on_connect {
                btn = btn.on_click(move |ev, w, cx| h(ev, w, cx));
            }

            v_flex().w_full().child(btn)
        }
        ConnState::Connecting => v_flex().w_full().child(
            Button::new(tr("sidebar.connecting", lang))
                .variant(ButtonVariant::Primary)
                .block()
                .disabled()
                .loading()
                .id("connect-btn-busy"),
        ),
        ConnState::Connected(_) => {
            let mut btn = Button::new(tr("sidebar.disconnect", lang))
                .variant(ButtonVariant::Danger)
                .block()
                .id("disconnect-btn");

            if let Some(h) = on_disconnect {
                btn = btn.on_click(move |ev, w, cx| h(ev, w, cx));
            }

            v_flex().w_full().child(btn)
        }
        ConnState::Error(_) => {
            let mut btn = Button::new(tr("sidebar.connect", lang))
                .variant(ButtonVariant::Primary)
                .block()
                .icon(IconName::Play)
                .id("connect-btn");

            if let Some(h) = on_connect {
                btn = btn.on_click(move |ev, w, cx| h(ev, w, cx));
            }

            v_flex().w_full().child(btn)
        }
    }
}
