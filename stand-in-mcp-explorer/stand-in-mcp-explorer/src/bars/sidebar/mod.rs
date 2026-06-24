//! Connection sidebar — 2-zone composition (scroll body + privacy footer)
//! using `SidebarShell` with `show_brand(false)`. The brand identity lives in
//! the app header row (grid 2×2 — 028 Item #13 releitura), not in the sidebar.
//!
//! Renders the connection sidebar: connection form (transport selector +
//! conditional fields + env rows), saved servers, connect/disconnect block,
//! and privacy badge.

pub mod auth_state;
pub mod auth_trigger;
pub mod brand_header;
pub mod connect_block;
pub mod connection_form;
pub mod env_rows;
pub mod env_trigger;
pub mod privacy_badge;
pub mod saved_servers;
pub mod sidebar_state;

use gpui::{App, IntoElement, Window};
use stand_in_mcp_explorer_ds::core::button::ClickHandler;

use crate::app::conn_state::ConnState;
use crate::app::i18n::Lang;
use crate::app::servers::ServerEntry;

use self::auth_state::AuthMethod;
use self::sidebar_state::SidebarState;

use auth_trigger::render_auth_trigger;
use connect_block::render_connect_block;
use connection_form::render_connection_form;
use privacy_badge::render_privacy_badge;
use saved_servers::render_saved_servers;

#[allow(clippy::too_many_arguments)]
pub fn render_sidebar(
    form: &SidebarState,
    conn_state: &ConnState,
    saved_servers: &[ServerEntry],
    lang: Lang,
    guided: bool,
    auth_method: AuthMethod,
    on_open_auth: Option<ClickHandler>,
    transport_handlers: Vec<ClickHandler>,
    on_connect: Option<ClickHandler>,
    on_disconnect: Option<ClickHandler>,
    on_open_env: Option<ClickHandler>,
    on_pick_preset: Vec<ClickHandler>,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    use stand_in_mcp_explorer_ds::navigation::sidebar_shell::SidebarShell;

    SidebarShell::new()
        .id("connection-sidebar")
        .show_brand(false)
        .children([
            render_connection_form(
                form,
                conn_state,
                lang,
                guided,
                transport_handlers,
                on_open_env,
                window,
                cx,
            )
            .into_any_element(),
            render_auth_trigger(auth_method, form.transport, lang, on_open_auth, window, cx)
                .into_any_element(),
            render_connect_block(
                conn_state,
                form.transport,
                lang,
                on_connect,
                on_disconnect,
                window,
                cx,
            )
            .into_any_element(),
            render_saved_servers(saved_servers, lang, on_pick_preset, window, cx)
                .into_any_element(),
        ])
        .footer(render_privacy_badge(lang, window, cx))
}
