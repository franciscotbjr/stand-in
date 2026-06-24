//! Topbar right-aligned caps cluster — language switcher, guided toggle,
//! capability chips (only when connected), and reconnect button.

use gpui::{
    AnyElement, App, FontWeight, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, Window, div, px,
};
use gpui_component::ActiveTheme as _;
use stand_in_mcp_explorer_ds::core::button::ClickHandler;
use stand_in_mcp_explorer_ds::core::icon::{Icon, IconName, IconSize};
use stand_in_mcp_explorer_ds::core::icon_button::IconButton;
use stand_in_mcp_explorer_ds::forms::select::{Select, SelectHandler};
use stand_in_mcp_explorer_ds::navigation::CapChip;
use stand_in_mcp_explorer_ds::prelude::SharedString;
use stand_in_mcp_explorer_ds::theme::density::RADIUS_CHIP;
use stand_in_mcp_explorer_ds::theme::palette::OBY;
use stand_in_mcp_explorer_ds::theme::typography;

use crate::app::conn_state::ConnState;
use crate::app::events::ConnConfig;
use crate::app::i18n::{Lang, tr};

/// Build the right-aligned children for the topbar.
///
/// Always: settings gear + language switcher + guided toggle.
/// When Connected: 3 CapChips (tools, resources, prompts) + reconnect button.
#[allow(clippy::too_many_arguments)]
pub fn build_caps(
    state: &ConnState,
    last_dispatched: Option<&ConnConfig>,
    lang: Lang,
    guided: bool,
    on_lang_change: Option<SelectHandler>,
    on_guided_toggle: Option<ClickHandler>,
    on_reconnect: Option<ClickHandler>,
    on_settings: Option<ClickHandler>,
    _window: &mut Window,
    cx: &mut App,
) -> Vec<AnyElement> {
    let mut children: Vec<AnyElement> = Vec::new();

    // --- Settings gear (always visible, first in cluster) ---
    if let Some(handler) = on_settings {
        children.push(
            IconButton::new(IconName::Tool, "Settings")
                .id("topbar-settings-btn")
                .on_click_boxed(handler)
                .into_any_element(),
        );
    }

    // --- Language switcher (always visible) ---
    let lang_options: Vec<(SharedString, SharedString)> = Lang::ALL
        .iter()
        .map(|l| (SharedString::from(l.code()), SharedString::from(l.label())))
        .collect();
    let lang_ix = match lang {
        Lang::PtBr => 0,
        Lang::En => 1,
        Lang::Es => 2,
    };

    // Fixed width so the control does not float per language label (the topbar
    // caps cluster is content-sized — 028 QA Item #16). Fits the widest label
    // ("Português"); shorter labels keep the chevron right-aligned.
    let mut lang_select = Select::new("topbar-lang", lang_options, lang_ix).width(px(128.));
    if let Some(handler) = on_lang_change {
        lang_select = lang_select.on_change(move |ix, value, window, cx| {
            handler(ix, value, window, cx);
        });
    }
    children.push(lang_select.into_any_element());

    // --- Guided mode toggle ---
    // Follows the topbar cap-chip standard (surface-2 + border + mono, like the
    // capability chips) — NOT a navy Primary pill. Active = oby accent
    // (border + icon + text), mirroring the prototype's oby checkbox (Item #9).
    let guided_label = tr("topbar.guidedMode", lang);
    let t = cx.theme();
    let mut guided_chip = div()
        .id("topbar-guided-toggle")
        .flex()
        .flex_shrink_0()
        .items_center()
        .gap(px(6.))
        .px(px(9.))
        .py(px(5.))
        .rounded(px(RADIUS_CHIP))
        .bg(t.secondary)
        .border_1()
        .border_color(if guided { OBY } else { t.border })
        .text_size(px(typography::FS_XS))
        .font_weight(FontWeight::SEMIBOLD)
        .font_family(t.mono_font_family.clone())
        .text_color(if guided {
            t.foreground
        } else {
            t.secondary_foreground
        })
        .cursor_pointer()
        .child(
            Icon::new(IconName::Info)
                .size(IconSize::Xs)
                .color(if guided { OBY } else { t.secondary_foreground }),
        )
        .child(SharedString::from(guided_label));
    if let Some(handler) = on_guided_toggle {
        guided_chip = guided_chip.on_click(move |ev, window, cx| {
            handler(ev, window, cx);
        });
    }
    children.push(guided_chip.into_any_element());

    // --- Capability chips + reconnect (only when Connected) ---
    if let ConnState::Connected(snap) = state {
        children.push(
            CapChip::new(tr("tabs.tools", lang))
                .count(snap.tools.len())
                .icon(IconName::Tool)
                .id("cap-tools")
                .into_any_element(),
        );
        let res_count = snap.resources.len() + snap.templates.len();
        children.push(
            CapChip::new(tr("tabs.resources", lang))
                .count(res_count)
                .icon(IconName::Doc)
                .id("cap-resources")
                .into_any_element(),
        );
        children.push(
            CapChip::new(tr("tabs.prompts", lang))
                .count(snap.prompts.len())
                .icon(IconName::Chat)
                .id("cap-prompts")
                .into_any_element(),
        );

        // Reconnect (refresh) — disabled if no last_dispatched
        if let Some(handler) = on_reconnect {
            let disabled = last_dispatched.is_none();
            let btn = if disabled {
                IconButton::new(IconName::Refresh, tr("topbar.reconnect", lang))
                    .id("topbar-reconnect-disabled")
            } else {
                IconButton::new(IconName::Refresh, tr("topbar.reconnect", lang))
                    .id("topbar-reconnect")
                    .on_click_boxed(handler)
            };
            children.push(btn.into_any_element());
        }
    }

    children
}
