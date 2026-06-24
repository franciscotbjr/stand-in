//! Connection form — transport SegmentedControl + conditional fields
//! (command/args/env for Stdio, URL for remote).

use crate::app::conn_state::ConnState;
use crate::app::i18n::{Lang, tr};
use gpui::{App, InteractiveElement, IntoElement, ParentElement, Styled, Window, px};
use gpui_component::v_flex;
use stand_in_mcp_explorer_ds::core::button::ClickHandler;
use stand_in_mcp_explorer_ds::core::icon::IconName;
use stand_in_mcp_explorer_ds::data::hint_bar::HintBar;
use stand_in_mcp_explorer_ds::forms::field::Field;
use stand_in_mcp_explorer_ds::forms::segmented_control::SegmentedControl;
use stand_in_mcp_explorer_ds::navigation::section_label::SectionLabel;

use crate::app::events::Transport;

use super::env_trigger::render_env_trigger;
use super::sidebar_state::SidebarState;

#[allow(clippy::too_many_arguments)]
pub fn render_connection_form(
    state: &SidebarState,
    _conn_state: &ConnState,
    lang: Lang,
    guided: bool,
    transport_handlers: Vec<ClickHandler>,
    on_open_env: Option<ClickHandler>,
    _window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let mut col = v_flex().id("connection-form").w_full().gap(px(12.));

    col = col.child(SectionLabel::new(tr("sidebar.connection", lang)).icon(IconName::Plug));

    // Guided hint — shown before the transport selector
    if guided {
        let hint_key = match state.transport {
            Transport::Stdio => "sidebar.transportHintStdio",
            Transport::Http => "sidebar.transportHintRemote",
        };
        col = col.child(
            HintBar::new()
                .text(tr(hint_key, lang))
                .id("guided-transport-hint"),
        );
    }

    col = col.child(
        SegmentedControl::new(
            "seg-transport",
            vec![
                (
                    gpui::SharedString::from("http"),
                    gpui::SharedString::from("HTTP"),
                ),
                (
                    gpui::SharedString::from("stdio"),
                    gpui::SharedString::from("STDIO"),
                ),
            ],
            state.transport.selected_ix(),
        )
        .handlers(transport_handlers),
    );

    match state.transport {
        Transport::Stdio => {
            col = col.child(
                Field::new(&state.command_input)
                    .label(tr("sidebar.command", lang))
                    .required()
                    .id("cmd-field"),
            );
            col = col.child(
                Field::new(&state.args_input)
                    .label(tr("sidebar.args", lang))
                    .long()
                    .id("args-field"),
            );

            let env_count = state.env_rows.len();
            col = col.child(render_env_trigger(
                env_count,
                state.transport,
                lang,
                on_open_env,
                _window,
                cx,
            ));
        }
        Transport::Http => {
            col = col.child(
                Field::new(&state.url_input)
                    .label(tr("sidebar.serverUrl", lang))
                    .required()
                    .id("url-field"),
            );
        }
    }

    col
}
