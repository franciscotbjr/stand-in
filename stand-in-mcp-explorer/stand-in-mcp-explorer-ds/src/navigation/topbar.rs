//! Topbar — 60px horizontal bar with connection state on the left and
//! optional action chips/buttons on the right.
//!
//! 1:1 with `navigation/Topbar.jsx` + `.topbar`/`.conn-state` in
//! `navigation/navigation.css`. The Topbar is a SEPARATE sibling from the
//! Tabbar — tabs never live inside the topbar (preserve the structural
//! boundary from the canon).
//!
//! Anatomy: h_flex (60px tall, flex_1 — fills the header row to the right of
//! the brand cell, grid 2×2), padding 0×20, gap 16, bg surface,
//! border-bottom 1px border. Left zone: StatusDot + column (title mono 14px
//! weight 600 + meta 11.5 text-3 mt 1). Right zone: `.right_children(…)`
//! rendered in an `.caps`-style h_flex (gap 7, margin-left auto).
//!
//! ```ignore
//! use stand_in_mcp_explorer_ds::navigation::Topbar;
//! use stand_in_mcp_explorer_ds::core::{DotState, StatusDot};
//! use stand_in_mcp_explorer_ds::core::{IconName};
//! use stand_in_mcp_explorer_ds::navigation::CapChip;
//!
//! Topbar::new(DotState::On, "server-filesystem", "STDIO \u{b7} v2026.4.1 \u{b7} 57ms")
//!     .right_children([
//!         CapChip::new("tools").count(6).icon(IconName::Tool).into_any_element(),
//!         Button::new("Modo guiado", ButtonVariant::Ghost, ButtonSize::Sm).into_any_element(),
//!     ]);
//! ```
//!
//! Rules (canon): title = connection context, NOT the app name. Metadata
//! joined by " \u{b7} " (middot). Never put tabs inside the Topbar.

use gpui::{
    AnyElement, App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, Styled, Window, div, px,
};
use gpui_component::{ActiveTheme as _, h_flex};

use crate::core::status_dot::{DotState, StatusDot};

// ---------------------------------------------------------------------------
// Topbar
// ---------------------------------------------------------------------------

/// The 60px top bar showing connection state and optional right-aligned
/// action content (CapChips, ghost buttons). A sibling of the Tabbar which
/// sits directly below.
#[derive(IntoElement)]
pub struct Topbar {
    state: DotState,
    title: SharedString,
    meta: SharedString,
    right_children: Vec<AnyElement>,
    id: ElementId,
}

impl Topbar {
    /// Create a top bar with the given connection-status dot state, context
    /// title (the connected server, never the app name), and metadata line
    /// (transport · version · latency, joined by " · ").
    pub fn new(
        state: DotState,
        title: impl Into<SharedString>,
        meta: impl Into<SharedString>,
    ) -> Self {
        Self {
            state,
            title: title.into(),
            meta: meta.into(),
            right_children: Vec::new(),
            id: ElementId::from("topbar"),
        }
    }

    /// Set the right-aligned children (CapChips, ghost buttons). Rendered
    /// inside a `.caps`-style h_flex (gap 7, margin-left auto).
    pub fn right_children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.right_children = children.into_iter().collect();
        self
    }

    /// Override the element id.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }
}

impl RenderOnce for Topbar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = cx.theme();

        let conn_column = div()
            .min_w(px(0.))
            .child(
                div()
                    .text_size(px(14.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .font_family(t.mono_font_family.clone())
                    .text_color(t.foreground)
                    .child(self.title),
            )
            .child(
                div()
                    .text_size(px(11.5))
                    .text_color(t.muted_foreground)
                    .mt(px(1.))
                    .child(self.meta),
            );

        let mut row = h_flex()
            .id(self.id)
            .h(px(60.))
            // Right cell of the app header row: grow to fill the width after the
            // brand cell so the internal spacer can push the caps to the edge
            // (grid 2×2 — 028 Item #13 releitura). min_w(0) lets the title
            // column absorb squeeze instead of overflowing.
            .flex_1()
            .min_w(px(0.))
            .px(px(20.))
            .gap(px(16.))
            .items_center()
            .bg(t.colors.sidebar)
            .border_b_1()
            .border_color(t.border)
            .child(StatusDot::new(self.state))
            .child(conn_column);

        if !self.right_children.is_empty() {
            row = row.child(div().flex_1());
            row = row.child(
                // Actions cluster must NOT shrink — otherwise a text button
                // (e.g. "Modo guiado") gets squashed and clips its label. The
                // title column (min_w 0) absorbs any squeeze instead.
                h_flex()
                    .flex_none()
                    .gap(px(7.))
                    .items_center()
                    .children(self.right_children),
            );
        }

        row
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topbar_new_fields() {
        let tb = Topbar::new(
            DotState::On,
            "server-filesystem",
            "STDIO \u{b7} v2026.4.1 \u{b7} 57ms",
        );
        assert_eq!(tb.state, DotState::On);
        assert_eq!(tb.title.as_ref(), "server-filesystem");
        assert_eq!(tb.meta.as_ref(), "STDIO \u{b7} v2026.4.1 \u{b7} 57ms");
        assert!(tb.right_children.is_empty());
    }

    #[test]
    fn test_topbar_new_different_states() {
        for state in [DotState::On, DotState::Off, DotState::Busy, DotState::Err] {
            let tb = Topbar::new(state, "ctx", "meta");
            assert_eq!(tb.state, state);
        }
    }

    #[test]
    fn test_topbar_right_children_empty() {
        let tb = Topbar::new(DotState::On, "title", "meta");
        assert!(tb.right_children.is_empty());
    }

    #[test]
    fn test_topbar_default_id() {
        let tb = Topbar::new(DotState::Off, "off", "idle");
        assert_eq!(tb.id, ElementId::from("topbar"));
    }

    #[test]
    fn test_topbar_custom_id() {
        let tb = Topbar::new(DotState::On, "x", "y").id("my-topbar");
        assert_eq!(tb.id, ElementId::from("my-topbar"));
    }

    #[test]
    fn test_topbar_title_is_context_not_app_name_documented() {
        let tb = Topbar::new(DotState::On, "server-filesystem", "STDIO \u{b7} 57ms");
        assert_ne!(tb.title.as_ref(), "MCP Explorer");
    }
}
