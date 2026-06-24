//! Onboarding / EmptyState screen shown when disconnected (ready or error).
//! Renders a branded EmptyState with a 3-step guide and a "Connect now" CTA
//! that dispatches the real `Connect` command — no façade.

use crate::app::conn_state::ConnState;
use crate::app::events::ConnConfig;
use crate::app::i18n::{Lang, tr};
use gpui::{IntoElement, SharedString};
use stand_in_mcp_explorer_ds::core::button::{Button, ButtonVariant, ClickHandler};
use stand_in_mcp_explorer_ds::core::icon::IconName;
use stand_in_mcp_explorer_ds::data::empty_state::{EmptyState, EmptyStep};

pub fn render_onboarding(
    state: &ConnState,
    last_dispatched: Option<&ConnConfig>,
    lang: Lang,
    connect_handler: Option<ClickHandler>,
) -> impl IntoElement {
    let (title_key, body_text): (&str, SharedString) = match state {
        ConnState::Error(err) => ("disconnected.titleError", SharedString::from(err.clone())),
        _ => (
            "disconnected.titleReady",
            SharedString::from(tr("disconnected.body", lang)),
        ),
    };

    let step2_sub = last_dispatched
        .map(|config| match config.transport {
            crate::app::events::Transport::Stdio => {
                format!("{} {}", config.command, config.args.join(" "))
            }
            crate::app::events::Transport::Http => config.url.clone(),
        })
        .unwrap_or_default();

    let steps = vec![
        EmptyStep::new(
            "1",
            tr("disconnected.step1Title", lang),
            tr("disconnected.step1Sub", lang),
        ),
        EmptyStep::new("2", tr("disconnected.step2Title", lang), step2_sub),
        EmptyStep::new(
            "3",
            tr("disconnected.step3Title", lang),
            tr("disconnected.step3Sub", lang),
        ),
    ];

    let cta_label = tr("disconnected.connectNow", lang);
    let cta = {
        let mut btn = Button::new(cta_label)
            .variant(ButtonVariant::Primary)
            .icon(IconName::Play);
        if let Some(handler) = connect_handler {
            btn = btn.on_click(handler);
        }
        btn
    };

    EmptyState::new(tr(title_key, lang))
        .icon(IconName::Plug)
        .body(body_text)
        .steps(steps)
        .action(cta)
}
