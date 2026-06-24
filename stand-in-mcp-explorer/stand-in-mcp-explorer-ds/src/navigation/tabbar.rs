//! Tabbar — 46px horizontal tab bar with a 2px oby underline on the active
//! tab and mono counts. Always sits directly below the Topbar as a separate
//! sibling.
//!
//! 1:1 with `navigation/Tabbar.jsx` + `.tabs`/`.tab` in
//! `navigation/navigation.css`. The underline is static (no animation —
//! the canon does not animate the tab underline so the motion budget is
//! preserved). Icons are 15px, optional but consistent (all or none per the
//! canon).
//!
//! Anatomy: h_flex (46px fixed, flex_none), padding 0×16, gap 2, bg surface,
//! border-bottom 1px border. Items are aligned flex-end. Tab: 13.5 weight 600,
//! padding 0×14, height 45px, gap 8, text-2 (hover→text). Active: text +
//! underline 2px oby (absolute bottom child, left 10 right 10, radius 2 2 0 0,
//! static). Count: FS_XS mono text-3 on surface-2, RADIUS_BADGE 6, pad 1×6;
//! disappears at 0 or None; in active tab → text colour.
//!
//! Rules (canon): 3–6 fixed tabs; no closeable or scrollable tabs; counter
//! hides at 0; icons 15px optional but all-or-none.
//!
//! Caller owns the active index. One `ClickHandler` per tab item is created
//! via `cx.listener` (same pattern as `SegmentedControl`/M8).
//!
//! ```ignore
//! use stand_in_mcp_explorer_ds::navigation::{Tabbar, TabItem};
//!
//! let items = vec![
//!     TabItem::new("tools", "Tools").icon(IconName::Tool).count(6),
//!     TabItem::new("history", "Hist\u{f3}rico"),
//! ];
//! Tabbar::new("main-tabs", items, active_ix)
//!     .handlers(handlers);
//! ```

use gpui::{
    App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, div, px,
};
use gpui_component::{ActiveTheme as _, h_flex};

use crate::core::button::ClickHandler;
use crate::core::icon::{Icon, IconName};
use crate::theme::density::RADIUS_BADGE;
use crate::theme::typography;

// ---------------------------------------------------------------------------
// TabItem
// ---------------------------------------------------------------------------

/// A single tab in the Tabbar — label, optional icon (15px), optional count.
///
/// The count is rendered in a mono badge and **disappears** when `0` or `None`
/// (canon rule). Icons are all-or-none across the tab set, but the struct
/// leaves that enforcement to the caller (rustdoc).
#[derive(Debug, Clone)]
pub struct TabItem {
    pub id: SharedString,
    pub label: SharedString,
    pub count: Option<usize>,
    pub icon: Option<IconName>,
}

impl TabItem {
    /// Create a tab with an id and label.
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            count: None,
            icon: None,
        }
    }

    /// Attach an optional count badge (mono, hidden at 0).
    pub fn count(mut self, n: usize) -> Self {
        self.count = Some(n);
        self
    }

    /// Attach an icon (15px, shown before the label).
    pub fn icon(mut self, name: IconName) -> Self {
        self.icon = Some(name);
        self
    }
}

// ---------------------------------------------------------------------------
// Tabbar
// ---------------------------------------------------------------------------

/// A 46px tab bar with an oby underline on the active tab and mono counters.
///
/// The caller owns the active index and creates one `ClickHandler` per tab
/// item (via `cx.listener`), matching the `SegmentedControl`/M8 pattern.
/// Handlers may be `None` for non-interactive rendering (capture mode).
#[derive(IntoElement)]
pub struct Tabbar {
    id: ElementId,
    items: Vec<TabItem>,
    active_ix: usize,
    handlers: Vec<Option<ClickHandler>>,
}

impl Tabbar {
    /// Create a tab bar. Panics if `items` is empty (debug_assert).
    pub fn new(id: impl Into<ElementId>, items: Vec<TabItem>, active_ix: usize) -> Self {
        debug_assert!(!items.is_empty(), "Tabbar requires at least one tab");
        Self {
            id: id.into(),
            items,
            active_ix,
            handlers: Vec::new(),
        }
    }

    /// Set one `ClickHandler` per tab (same order as items). Use `None` to
    /// mark non-interactive rendering (capture mode). The handlers are indexed
    /// by position and attached to each tab via `on_click`.
    pub fn handlers(mut self, handlers: Vec<Option<ClickHandler>>) -> Self {
        self.handlers = handlers;
        self
    }
}

fn paired(
    items: Vec<TabItem>,
    handlers: Vec<Option<ClickHandler>>,
) -> Vec<(TabItem, Option<ClickHandler>)> {
    let mut h = handlers;
    while h.len() < items.len() {
        h.push(None);
    }
    items.into_iter().zip(h).collect()
}

impl RenderOnce for Tabbar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let Self {
            id,
            items,
            active_ix,
            handlers,
        } = self;
        let t = cx.theme();
        let mono = t.mono_font_family.clone();

        let mut tab_divs: Vec<gpui::AnyElement> = Vec::new();

        for (ix, (item, handler)) in paired(items, handlers).into_iter().enumerate() {
            let active = ix == active_ix;
            let tab_id = ElementId::from(format!("tab-{}", item.id));

            let fg = if active {
                t.foreground
            } else {
                t.secondary_foreground
            };

            // gpui's Div defaults to display:block — flex must be explicit or
            // gap/items_center are inert and icon/label/count stack vertically.
            let mut tab = div()
                .id(tab_id)
                .flex()
                .relative()
                .h(px(45.))
                .px(px(14.))
                .gap(px(8.))
                .text_size(px(13.5))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(fg)
                .cursor_pointer()
                .items_center()
                .hover(|h| h.text_color(t.foreground));

            if let Some(click) = handler {
                tab = tab.on_click(click);
            }

            if let Some(icon) = item.icon {
                tab = tab.child(Icon::new(icon).with_px(px(15.)));
            }

            tab = tab.child(item.label.clone());

            if active {
                tab = tab.child(
                    div()
                        .absolute()
                        .left(px(10.))
                        .right(px(10.))
                        .bottom(px(0.))
                        .h(px(2.))
                        .bg(t.list_active_border)
                        .rounded_t(px(2.)),
                );
            }

            if let Some(n) = item.count
                && n > 0
            {
                let count_fg = if active {
                    t.foreground
                } else {
                    t.muted_foreground
                };
                tab = tab.child(
                    div()
                        .text_size(px(typography::FS_XS))
                        .font_family(mono.clone())
                        .text_color(count_fg)
                        .bg(t.secondary)
                        .rounded(px(RADIUS_BADGE))
                        .px(px(6.))
                        .py(px(1.))
                        .child(SharedString::from(n.to_string())),
                );
            }

            tab_divs.push(tab.into_any_element());
        }

        h_flex()
            .id(id)
            .h(px(46.))
            .flex_none()
            .px(px(16.))
            .gap(px(2.))
            .items_end()
            .bg(t.colors.sidebar)
            .border_b_1()
            .border_color(t.border)
            .children(tab_divs)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- TabItem tests --

    #[test]
    fn test_tab_item_new() {
        let ti = TabItem::new("tools", "Tools");
        assert_eq!(ti.id.as_ref(), "tools");
        assert_eq!(ti.label.as_ref(), "Tools");
        assert!(ti.count.is_none());
        assert!(ti.icon.is_none());
    }

    #[test]
    fn test_tab_item_with_count() {
        let ti = TabItem::new("tools", "Tools").count(6);
        assert_eq!(ti.count, Some(6));
    }

    #[test]
    fn test_tab_item_with_count_zero() {
        let ti = TabItem::new("tools", "Tools").count(0);
        assert_eq!(ti.count, Some(0));
    }

    #[test]
    fn test_tab_item_with_icon() {
        let ti = TabItem::new("tools", "Tools").icon(IconName::Tool);
        assert_eq!(ti.icon, Some(IconName::Tool));
    }

    #[test]
    fn test_tab_item_chain() {
        let ti = TabItem::new("tools", "Tools").count(6).icon(IconName::Tool);
        assert_eq!(ti.count, Some(6));
        assert_eq!(ti.icon, Some(IconName::Tool));
    }

    #[test]
    fn test_tab_item_clone() {
        let ti = TabItem::new("tools", "Tools").count(6).icon(IconName::Tool);
        let ti2 = ti.clone();
        assert_eq!(ti2.id, ti.id);
        assert_eq!(ti2.label, ti.label);
        assert_eq!(ti2.count, ti.count);
        assert_eq!(ti2.icon, ti.icon);
    }

    // -- Tabbar tests --

    #[test]
    fn test_tabbar_new() {
        let items = vec![TabItem::new("tools", "Tools")];
        let tb = Tabbar::new("main", items, 0);
        assert_eq!(tb.active_ix, 0);
        assert!(tb.handlers.is_empty());
    }

    #[test]
    fn test_tabbar_multiple_items() {
        let items = vec![
            TabItem::new("tools", "Tools"),
            TabItem::new("res", "Resources"),
        ];
        let tb = Tabbar::new("main", items, 1);
        assert_eq!(tb.active_ix, 1);
    }

    #[test]
    fn test_tabbar_with_handlers() {
        let items = vec![TabItem::new("tools", "Tools")];
        let tb = Tabbar::new("main", items, 0).handlers(vec![None]);
        assert_eq!(tb.handlers.len(), 1);
    }

    #[test]
    fn test_tabbar_handlers_index_aligns_with_items() {
        let items = vec![
            TabItem::new("a", "A"),
            TabItem::new("b", "B"),
            TabItem::new("c", "C"),
        ];
        let tb = Tabbar::new("main", items, 0).handlers(vec![None, None, None]);
        assert_eq!(tb.handlers.len(), 3);
    }

    #[test]
    fn test_tabbar_default_id_name() {
        let items = vec![TabItem::new("x", "X")];
        let tb = Tabbar::new("main-tabs", items, 0);
        assert_eq!(tb.id, ElementId::from("main-tabs"));
    }

    // -- paired helper tests --

    #[test]
    fn test_paired_no_handlers() {
        let items = vec![
            TabItem::new("a", "A"),
            TabItem::new("b", "B"),
            TabItem::new("c", "C"),
        ];
        let result = paired(items, vec![]);
        assert_eq!(result.len(), 3);
        for (_, h) in &result {
            assert!(h.is_none());
        }
    }

    #[test]
    fn test_paired_partial_handlers() {
        let items = vec![
            TabItem::new("a", "A"),
            TabItem::new("b", "B"),
            TabItem::new("c", "C"),
        ];
        let result = paired(items, vec![None, None]);
        assert_eq!(result.len(), 3);
        assert!(result[0].1.is_none());
        assert!(result[1].1.is_none());
        assert!(result[2].1.is_none());
    }

    #[test]
    fn test_paired_all_handlers() {
        let items = vec![TabItem::new("a", "A"), TabItem::new("b", "B")];
        let result = paired(items, vec![None, None]);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_paired_single_item_no_handlers() {
        let items = vec![TabItem::new("x", "X")];
        let result = paired(items, vec![]);
        assert_eq!(result.len(), 1);
    }
}
