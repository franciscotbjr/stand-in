//! Topbar — 60px bar with connection dot, server title/meta, and
//! right-aligned caps cluster (language switcher, guided toggle, capability
//! chips, reconnect). Pure presentation functions are unit-testable without
//! a Window.

pub mod caps;

use gpui::{App, IntoElement, Window};
use stand_in_mcp_explorer_ds::core::button::ClickHandler;
use stand_in_mcp_explorer_ds::core::status_dot::DotState;
use stand_in_mcp_explorer_ds::forms::select::SelectHandler;
use stand_in_mcp_explorer_ds::navigation::topbar::Topbar;
use stand_in_mcp_explorer_ds::prelude::SharedString;

use crate::app::conn_state::ConnState;
use crate::app::events::ConnConfig;
use crate::app::i18n::{Lang, tr};

/// Map `ConnState` to `DotState` — pure, unit-testable.
pub fn dot_state(state: &ConnState) -> DotState {
    match state {
        ConnState::Disconnected => DotState::Off,
        ConnState::Connecting => DotState::Busy,
        ConnState::Connected(_) => DotState::On,
        ConnState::Error(_) => DotState::Err,
    }
}

/// Derive the topbar title from connection state — pure, unit-testable.
pub fn title_for(state: &ConnState, lang: Lang) -> SharedString {
    match state {
        ConnState::Connected(snap) => SharedString::from(snap.server_info.name.clone()),
        ConnState::Disconnected => SharedString::from(tr("topbar.states.disconnected", lang)),
        ConnState::Connecting => SharedString::from(tr("topbar.states.connecting", lang)),
        ConnState::Error(_) => SharedString::from(tr("topbar.states.error", lang)),
    }
}

/// Derive the topbar metadata line from connection state — pure, unit-testable.
pub fn meta_for(state: &ConnState, last_dispatched: Option<&ConnConfig>) -> SharedString {
    match state {
        ConnState::Connected(snap) => {
            let transport_label = last_dispatched
                .map(|c| c.transport.label())
                .unwrap_or("STDIO");
            let version = &snap.server_info.version;
            let latency = snap.latency_ms;
            SharedString::from(format!(
                "{} \u{b7} v{} \u{b7} {}ms",
                transport_label, version, latency
            ))
        }
        _ => SharedString::from(
            last_dispatched
                .map(|c| match c.transport {
                    crate::app::events::Transport::Stdio => {
                        format!("{} {}", c.command, c.args.join(" "))
                    }
                    crate::app::events::Transport::Http => c.url.clone(),
                })
                .unwrap_or_default(),
        ),
    }
}

/// Render the full topbar — dot, title, meta, and right-aligned caps cluster.
#[allow(clippy::too_many_arguments)]
pub fn render_topbar(
    state: &ConnState,
    last_dispatched: Option<&ConnConfig>,
    lang: Lang,
    guided: bool,
    on_lang_change: Option<SelectHandler>,
    on_guided_toggle: Option<ClickHandler>,
    on_reconnect: Option<ClickHandler>,
    on_settings: Option<ClickHandler>,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let dot = dot_state(state);
    let title = title_for(state, lang);
    let meta = meta_for(state, last_dispatched);

    let right_children = caps::build_caps(
        state,
        last_dispatched,
        lang,
        guided,
        on_lang_change,
        on_guided_toggle,
        on_reconnect,
        on_settings,
        window,
        cx,
    );

    Topbar::new(dot, title, meta)
        .id("app-topbar")
        .right_children(right_children)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::events::ServerSnapshot;
    use crate::app::events::{ConnConfig, Transport};
    use stand_in_client::prelude::{ServerCapabilities, ServerInfo};

    fn connected_snap(latency: u64) -> ConnState {
        ConnState::Connected(Box::new(ServerSnapshot {
            server_info: ServerInfo {
                name: "test-server".into(),
                version: "1.2.3".into(),
            },
            capabilities: ServerCapabilities::new(),
            tools: vec![],
            resources: vec![],
            templates: vec![],
            prompts: vec![],
            latency_ms: latency,
        }))
    }

    #[test]
    fn test_dot_state_mapping() {
        assert_eq!(dot_state(&ConnState::Disconnected), DotState::Off);
        assert_eq!(dot_state(&ConnState::Connecting), DotState::Busy);
        assert_eq!(dot_state(&connected_snap(42)), DotState::On);
        assert_eq!(dot_state(&ConnState::Error("fail".into())), DotState::Err);
    }

    #[test]
    fn test_title_for_disconnected() {
        assert_eq!(
            title_for(&ConnState::Disconnected, Lang::En).as_ref(),
            "Disconnected"
        );
        assert_eq!(
            title_for(&ConnState::Disconnected, Lang::PtBr).as_ref(),
            "Desconectado"
        );
    }

    #[test]
    fn test_title_for_connecting() {
        assert_eq!(
            title_for(&ConnState::Connecting, Lang::En).as_ref(),
            "Connecting…"
        );
    }

    #[test]
    fn test_title_for_connected() {
        let t = title_for(&connected_snap(42), Lang::En);
        assert_eq!(t.as_ref(), "test-server");
    }

    #[test]
    fn test_title_for_error() {
        assert_eq!(
            title_for(&ConnState::Error("fail".into()), Lang::En).as_ref(),
            "Connection error"
        );
    }

    #[test]
    fn test_meta_for_connected_with_config() {
        let config = ConnConfig {
            transport: Transport::Stdio,
            command: "cargo".into(),
            args: vec!["run".into()],
            url: String::new(),
            env: Vec::new(),
        };
        let meta = meta_for(&connected_snap(42), Some(&config));
        assert_eq!(meta.as_ref(), "STDIO · v1.2.3 · 42ms");
    }

    #[test]
    fn test_meta_for_connected_without_config() {
        let meta = meta_for(&connected_snap(57), None);
        assert_eq!(meta.as_ref(), "STDIO · v1.2.3 · 57ms");
    }

    #[test]
    fn test_meta_for_disconnected_shows_command() {
        let config = ConnConfig {
            transport: Transport::Stdio,
            command: "npx".into(),
            args: vec![
                "-y".into(),
                "@modelcontextprotocol/server-filesystem".into(),
            ],
            url: String::new(),
            env: Vec::new(),
        };
        let meta = meta_for(&ConnState::Disconnected, Some(&config));
        assert_eq!(
            meta.as_ref(),
            "npx -y @modelcontextprotocol/server-filesystem"
        );
    }

    #[test]
    fn test_meta_for_disconnected_shows_url() {
        let config = ConnConfig {
            transport: Transport::Http,
            command: String::new(),
            args: vec![],
            url: "https://example.com/mcp".into(),
            env: Vec::new(),
        };
        let meta = meta_for(&ConnState::Disconnected, Some(&config));
        assert_eq!(meta.as_ref(), "https://example.com/mcp");
    }

    #[test]
    fn test_meta_for_connecting_shows_command() {
        let config = ConnConfig {
            transport: Transport::Http,
            command: String::new(),
            args: vec![],
            url: "http://localhost:3000/sse".into(),
            env: Vec::new(),
        };
        let meta = meta_for(&ConnState::Connecting, Some(&config));
        assert_eq!(meta.as_ref(), "http://localhost:3000/sse");
    }

    #[test]
    fn test_meta_for_error_shows_command() {
        let config = ConnConfig {
            transport: Transport::Stdio,
            command: "node".into(),
            args: vec!["server.js".into()],
            url: String::new(),
            env: Vec::new(),
        };
        let meta = meta_for(&ConnState::Error("fail".into()), Some(&config));
        assert_eq!(meta.as_ref(), "node server.js");
    }
}
