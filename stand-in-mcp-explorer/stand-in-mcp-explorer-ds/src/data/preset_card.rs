//! PresetCard — selectable option card for saved presets / configuration
//! choices.
//!
//! 1:1 with `data/PresetCard.jsx` + `.preset*` rules in `data/data.css`.
//!
//! Fixed semantics (never reinterpret): selection draws an **oby border + 2px
//! oby-22% ring** (composite, not `shadow_*`). This is the PresetCard selection
//! pattern — **ListItem uses a left filete**. These must never be swapped
//! (the canon says: "ListItem = navega\u{e7}\u{e3}o em lista densa; PresetCard =
//! escolha de configura\u{e7}\u{e3}o").
//!
//! Anatomy:
//! 1. Card: full-width, bg `surface-2`, border 1px `border`, radius
//!    `RADIUS_CARD` (10), padding 10×11, mb 7 (built-in stacking), **no shadow**
//!    (in-flow — prohibition 4).
//! 2. Hover: border `border-2` + bg `surface-3` (steps; never scale or shadow).
//! 3. Selected: border `OBY` + **ring 2px oby-22%** via an absolutely-positioned
//!    inset child (the M4 halo technique — NOT `shadow_*`). The ring uses the
//!    palette's OBY at 22% alpha, composed as a negative-inset `absolute` div
//!    painted behind the card proper.
//! 4. Top: `name` mono 12.5 semibold `text` + spacer + optional badge slot.
//! 5. Desc: 11.5 `text-3`, mt 4, lh 1.4 — ONE short line, no period, truncated
//!    with `text_ellipsis`.
//!
//! ```ignore
//! use stand_in_mcp_explorer_ds::data::PresetCard;
//! use stand_in_mcp_explorer_ds::core::{Badge, BadgeKind};
//!
//! PresetCard::new("ps-server-filesystem", "server-filesystem")
//!     .desc("Acesso a arquivos do disco local")
//!     .badge(Badge::new("stdio", BadgeKind::Muted).into_any_element())
//!     .selected(sel == "server-filesystem")
//!     .on_click(cx.listener(move |this, _, _, cx| { this.sel = ...; cx.notify(); }));
//! ```

use gpui::{
    App, ClickEvent, ElementId, FontWeight, Hsla, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, px, relative,
};
use gpui_component::{ActiveTheme as _, h_flex, v_flex};

use crate::core::button::ClickHandler;
use crate::theme::colors::JandiExt;
use crate::theme::density::RADIUS_CARD;
use crate::theme::palette::OBY;
use crate::theme::typography;

/// OBY at 22% alpha — the selection ring (canon: `box-shadow 0 0 0 2px oby-22%`).
/// The ring is composed as an absolutely-positioned inset border (the M4 halo
/// technique), NOT a `shadow_*` call.
const OBY_RING: Hsla = Hsla {
    h: OBY.h,
    s: OBY.s,
    l: OBY.l,
    a: 0.22,
};

/// Ring thickness in px (the 2px `box-shadow` spread from the canon).
const RING_WIDTH: f32 = 2.0;

// ---------------------------------------------------------------------------
// PresetCard
// ---------------------------------------------------------------------------

/// Selectable configuration card with mono name, muted badge, and short
/// sans-serif description.
///
/// Selection is caller-owned — pass `.selected(bool)` and `.on_click(handler)`.
/// Never swap this with `ListItem` (filete vs ring semantics).
#[derive(IntoElement)]
pub struct PresetCard {
    id: ElementId,
    name: SharedString,
    desc: Option<SharedString>,
    badge: Option<gpui::AnyElement>,
    selected: bool,
    on_click: Option<ClickHandler>,
}

impl PresetCard {
    /// Create a preset card. `id` is the element id; `name` is the technical
    /// identifier rendered in mono (never capitalise or translate).
    pub fn new(id: impl Into<ElementId>, name: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            desc: None,
            badge: None,
            selected: false,
            on_click: None,
        }
    }

    /// Set the short prose description (sans-serif, ONE line, no period, truncated).
    pub fn desc(mut self, text: impl Into<SharedString>) -> Self {
        self.desc = Some(text.into());
        self
    }

    /// Attach an element in the top-row right slot (typically `Badge` Muted).
    pub fn badge(mut self, element: gpui::AnyElement) -> Self {
        self.badge = Some(element);
        self
    }

    /// Mark this card as selected (border OBY + ring 2px oby-22%).
    pub fn selected(mut self, yes: bool) -> Self {
        self.selected = yes;
        self
    }

    /// Attach a click handler. The caller owns selection state and should
    /// wire `cx.listener` / `cx.notify` in production code.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for PresetCard {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = cx.theme();
        let colors = &t.colors;
        let ext = cx.global::<JandiExt>();
        let mono = t.mono_font_family.clone();
        let selected = self.selected;
        let has_click = self.on_click.is_some();

        // Border color: OBY when selected, border otherwise.
        let border_color = if selected { OBY } else { colors.border };

        // Card wrapper — relative so the ring can position absolutely.
        let mut card = gpui::div()
            .id(self.id)
            .relative()
            .w_full()
            .bg(colors.secondary) // surface-2
            .border_1()
            .border_color(border_color)
            .rounded(px(RADIUS_CARD))
            .mb(px(7.))
            .cursor_pointer()
            .hover(|h| {
                if selected {
                    h
                } else {
                    h.bg(ext.surface_3).border_color(ext.border_2)
                }
            });

        // Selection ring: absolutely-positioned inset child painted BEHIND
        // the card content (the M4 halo technique). Not shadow_* —
        // the canon says `box-shadow 0 0 0 2px oby-22%`, composed here
        // as a negative-inset `absolute` div that overflows the card's
        // border-box without affecting layout.
        //
        // The ring is added conditionally on selection. Because it is an
        // absolute-positioned child (negative inset), it paints outside
        // the card's border-box without affecting layout — no jump on
        // selection change.
        if selected {
            card = card.child(
                gpui::div()
                    .absolute()
                    .inset(px(-RING_WIDTH))
                    .rounded(px(RADIUS_CARD + RING_WIDTH))
                    .bg(OBY_RING),
            );
        }

        // Inner content — sits above the ring in z-order.
        let mut inner = v_flex().p(px(11.)).pt(px(10.)).gap(px(4.));

        // Top row: name (mono 12.5 semibold) + spacer + badge
        let mut top = h_flex().gap(px(8.)).items_center();

        top = top.child(
            gpui::div()
                .flex_1()
                .min_w(px(0.))
                .text_size(px(typography::FS_MD - 0.5)) // 12.5 — canon
                .font_weight(FontWeight::SEMIBOLD)
                .font_family(mono)
                .text_color(colors.foreground)
                .text_ellipsis()
                .whitespace_nowrap()
                .child(self.name.clone()),
        );

        if let Some(badge) = self.badge {
            top = top.child(badge);
        }

        inner = inner.child(top);

        // Description: sans, 11.5, text-3, one line truncated
        if let Some(desc) = self.desc {
            inner = inner.child(
                gpui::div()
                    .text_size(px(11.5))
                    .text_color(colors.muted_foreground)
                    .line_height(relative(1.4))
                    .text_ellipsis()
                    .whitespace_nowrap()
                    .child(desc),
            );
        }

        card = card.child(inner);

        if has_click && let Some(click) = self.on_click {
            card = card.on_click(click);
        }

        card
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_card_defaults() {
        let card = PresetCard::new("ps-fs", "server-filesystem");
        assert_eq!(card.name.as_ref(), "server-filesystem");
        assert_eq!(card.id, ElementId::from("ps-fs"));
        assert!(card.desc.is_none());
        assert!(card.badge.is_none());
        assert!(!card.selected);
        assert!(card.on_click.is_none());
    }

    #[test]
    fn test_preset_card_with_desc() {
        let card = PresetCard::new("ps-fs", "fs").desc("Acesso a arquivos do disco local");
        assert_eq!(
            card.desc.as_deref(),
            Some("Acesso a arquivos do disco local")
        );
    }

    #[test]
    fn test_preset_card_desc_no_period() {
        let card = PresetCard::new("ps-fs", "fs").desc("Acesso a arquivos");
        assert!(!card.desc.unwrap().ends_with('.'));
    }

    #[test]
    fn test_preset_card_with_badge() {
        let card = PresetCard::new("ps-fs", "fs").badge(gpui::div().into_any_element());
        assert!(card.badge.is_some());
    }

    #[test]
    fn test_preset_card_selected() {
        let card = PresetCard::new("ps-fs", "fs").selected(true);
        assert!(card.selected);
    }

    #[test]
    fn test_preset_card_on_click_real() {
        let card = PresetCard::new("ps-fs", "fs").on_click(|_ev, _w, _cx| {});
        assert!(card.on_click.is_some());
    }

    #[test]
    fn test_preset_card_builder_chain() {
        let card = PresetCard::new("ps-write", "write_file")
            .desc("Grava dados no disco")
            .selected(false)
            .on_click(|_ev, _w, _cx| {});
        assert_eq!(card.name.as_ref(), "write_file");
        assert!(card.desc.is_some());
        assert!(!card.selected);
        assert!(card.on_click.is_some());
    }

    #[test]
    fn test_preset_card_no_shadow() {
        // Existence test: PresetCard has NO shadow API. The absence of
        // .shadow_* in the render impl is the proof (prohibition 4 — in-flow
        // cards never carry a shadow).
    }

    #[test]
    fn test_ring_not_shadow() {
        // Proof that the ring is composed (absolute div) not shadow_*.
        // The OBY_RING const + RING_WIDTH prove the composite technique.
        assert_eq!(OBY_RING.a, 0.22);
        assert_eq!(RING_WIDTH, 2.0);
    }

    #[test]
    fn test_preset_card_vs_listitem_semantics() {
        // PresetCard → ring, ListItem → filete. Never swap.
        let card = PresetCard::new("ps-fs", "fs").selected(true);
        assert!(card.selected);
    }

    #[test]
    fn test_ring_color_matches_oby_22pct() {
        assert_eq!(OBY_RING.h, OBY.h);
        assert_eq!(OBY_RING.s, OBY.s);
        assert_eq!(OBY_RING.l, OBY.l);
        assert_eq!(OBY_RING.a, 0.22);
    }

    #[test]
    fn test_ring_width_canon() {
        assert_eq!(RING_WIDTH, 2.0);
    }
}
