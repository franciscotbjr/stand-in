//! Saved servers section — renders `PresetCard` entries from the persisted
//! server list. Collapses entirely when empty (BUG-8).
//!
//! The list is framed in a bordered, rounded box (DS `border` token +
//! `RADIUS_CARD`, tokens-only, no shadow). When there are more than
//! `SAVED_VISIBLE` entries the box is **capped to ~2.5 cards tall and scrolls**
//! (028 QA Item #14) — the Connect button sits above it (see
//! `bars/sidebar/mod.rs`), so a long list never pushes Connect or the privacy
//! footer out of view. The cap shows a **half-card peek** of the next entry as
//! the scroll cue, and the section label carries a **count bullet** with the
//! total number of saved servers.

use crate::app::events::Transport;
use crate::app::i18n::{Lang, tr};
use crate::app::servers::ServerEntry;
use gpui::{App, InteractiveElement, IntoElement, ParentElement, Styled, Window, div, px};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{ActiveTheme as _, v_flex};
use stand_in_mcp_explorer_ds::core::badge::{Badge, BadgeKind};
use stand_in_mcp_explorer_ds::core::button::ClickHandler;
use stand_in_mcp_explorer_ds::core::icon::IconName;
use stand_in_mcp_explorer_ds::data::preset_card::PresetCard;
use stand_in_mcp_explorer_ds::navigation::section_label::SectionLabel;
use stand_in_mcp_explorer_ds::theme::density::RADIUS_CARD;

/// Cards fully shown before the saved-servers list caps its height and scrolls.
const SAVED_VISIBLE: usize = 2;

/// Capped height of the bordered scroll box ≈ two `PresetCard`s plus a half-card
/// peek of the third (the scroll cue), including the box's inner padding. Each
/// card is ~58px content + the built-in 7px `mb` and the 4px inter-card gap.
/// Calibratable — the GPUI capture is blind on Windows (O-004), so the exact px
/// is tuned by human.
const SAVED_LIST_MAX_H: f32 = 190.0;

/// Inner padding of the bordered list box.
const LIST_PAD: f32 = 8.0;

pub fn render_saved_servers(
    saved_servers: &[ServerEntry],
    lang: Lang,
    mut on_pick: Vec<ClickHandler>,
    _window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    if saved_servers.is_empty() {
        return v_flex().id("saved-servers-empty").into_any_element();
    }

    let border_color = cx.theme().colors.border;

    let mut col = v_flex().id("saved-servers").w_full().gap(px(4.));
    col = col.child(
        SectionLabel::new(tr("sidebar.savedServers", lang))
            .icon(IconName::Bolt)
            .count(saved_servers.len()),
    );

    // Build the preset cards once.
    let mut cards = Vec::with_capacity(saved_servers.len());
    for (i, entry) in saved_servers.iter().enumerate() {
        let transport_label = match entry.config.transport {
            Transport::Stdio => "STDIO",
            Transport::Http => "HTTP",
        };
        let desc = match entry.config.transport {
            Transport::Stdio => entry.config.command.clone(),
            Transport::Http => entry.config.url.clone(),
        };

        // Id MUST be unique per row — keyed by INDEX, not name. Two saved
        // servers can share a name (e.g. an HTTP and a STDIO "stand-in"); a
        // name-based id collides and gpui drops the duplicate's click dispatch,
        // so the second card looks dead (028 QA Item #21).
        let mut card = PresetCard::new(format!("ps-{i}"), entry.name.clone())
            .desc(desc)
            .badge(Badge::new(transport_label, BadgeKind::Muted).into_any_element());

        if i < on_pick.len() {
            let handler = std::mem::replace(&mut on_pick[i], Box::new(|_, _, _| {}));
            card = card.on_click(handler);
        }
        cards.push(card);
    }

    // Few entries: a bordered box at natural height (no scroll).
    if saved_servers.len() <= SAVED_VISIBLE {
        let mut box_ = v_flex()
            .id("saved-list")
            .w_full()
            .border_1()
            .border_color(border_color)
            .rounded(px(RADIUS_CARD))
            .overflow_hidden()
            .p(px(LIST_PAD))
            .gap(px(4.));
        for card in cards {
            box_ = box_.child(card);
        }
        col = col.child(box_);
        return col.into_any_element();
    }

    // Many entries: a fixed-height bordered box with a scroll leaf inside.
    // `overflow_y_scrollbar` re-wraps the leaf in a `Scrollable` that copies ONLY
    // the element's `size` (width/height) — it drops `max_h`/`flex_1`/`min_h` AND
    // the border. So the border + fixed height live on the OUTER box; the leaf is
    // `size_full` and scrolls inside it. The half-card peek signals more to come.
    let mut leaf = v_flex()
        .id("saved-scroll")
        .size_full()
        .overflow_y_scrollbar()
        .p(px(LIST_PAD))
        .gap(px(4.));
    for card in cards {
        leaf = leaf.child(card);
    }
    col = col.child(
        div()
            .id("saved-list")
            .w_full()
            .h(px(SAVED_LIST_MAX_H))
            .border_1()
            .border_color(border_color)
            .rounded(px(RADIUS_CARD))
            .overflow_hidden()
            .child(leaf),
    );

    col.into_any_element()
}
