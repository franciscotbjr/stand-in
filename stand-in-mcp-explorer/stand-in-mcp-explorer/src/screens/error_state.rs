//! Error state screen — renders the same EmptyState structure as the
//! onboarding screen but with the `titleError` title and the actual error
//! message as the body. The CTA still dispatches `Connect`, allowing retry.

use crate::app::conn_state::ConnState;
use crate::app::events::ConnConfig;
use crate::app::i18n::Lang;
use stand_in_mcp_explorer_ds::core::button::ClickHandler;

/// Render the error screen. Thin wrapper — delegates to the onboarding
/// EmptyState with the `Error` variant and the real error message.
pub fn render_error_state(
    state: &ConnState,
    last_dispatched: Option<&ConnConfig>,
    lang: Lang,
    connect_handler: Option<ClickHandler>,
) -> impl gpui::IntoElement {
    crate::screens::onboarding::render_onboarding(state, last_dispatched, lang, connect_handler)
}
