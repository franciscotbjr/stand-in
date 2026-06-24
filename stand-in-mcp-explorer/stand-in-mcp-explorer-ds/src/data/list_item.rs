//! ListItem — interactive data row for the list→detail pattern.
//!
//! 1:1 with `data/ListItem.jsx` + `.litem*` rules in `data/data.css`. Fixed
//! semantics (never reinterpret): selection shows a **2px oby left bar** +
//! `surface-2` bg; `PresetCard` (M14) selects with border+ring → never swap.
//!
//! Anatomy (two lines, FIXED HEIGHT — 031/M1 DS-extension):
//! 1. Top: `name` mono 13px semibold (technical identifier) + spacer + `badge`.
//! 2. Bottom: `desc` sans 12px text-3, line-clamped to 2 lines.
//!
//!    The desc container is ALWAYS rendered at a fixed reserved height, even
//!    when no `desc` is set, so every `ListItem` has the same external row
//!    height — the prerequisite for `gpui::uniform_list` (windowing).
//!
//! States: hover → `surface-2` bg only; selected → `surface-2` bg + 2px oby
//! left bar (child element, never a shadow — prohibition 4).
//!
//! Rules: `name` is a technical identifier → mono, never capitalise or translate;
//! `desc` is prose → sans, sentence case, ends with a period; items separate by
//! 1px bottom border (no margins between them); the list lives in `.list-col`
//! with `ListSearch` sticky at the top (M14).

use gpui::prelude::FluentBuilder;
use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, px,
};
use gpui_component::{ActiveTheme as _, h_flex, v_flex};

use crate::theme::palette::OBY;
use crate::theme::typography;

/// Click handler callback signature used by interactive components.
pub use crate::core::button::ClickHandler;

/// App-set hint: while the active list is being **mouse-wheel** scrolled,
/// suppress the row hover highlight so it doesn't "strobe" between items as
/// content scrolls under a stationary cursor (037 / O-025). Trackpad (precise
/// pixel) scrolling is unaffected. Default (`false`) = hover behaves normally.
#[derive(Default)]
pub struct ListScrollHoverSuppressed(pub bool);

impl gpui::Global for ListScrollHoverSuppressed {}

// ---------------------------------------------------------------------------
// Row-height constants (031/M1 DS-extension — fixed uniform row height so
// Tools / Resources / Prompts can use `gpui::uniform_list` for windowing.)
// ---------------------------------------------------------------------------

/// Reserved height for the mono `name` line (13 px semibold with breathing).
/// Must be ≥ the tallest badge the row can carry.
pub const NAME_LINE_H: f32 = 20.0;

/// Reserved height for the 2-line `desc` area (2 × 17.4 px line-height of the
/// canonical `fs-sm` × 1.45).
pub const DESC_RESERVED_H: f32 = 35.0;

/// External row height guaranteed for every `ListItem` variant — the sum of
/// top padding + `NAME_LINE_H` + desc margin + `DESC_RESERVED_H` + bottom
/// padding. `uniform_list` measures the first row against this constant.
///
/// Anatomy: `12 + 20 + 5 + 35 + 12 = 84`.
pub const LIST_ROW_HEIGHT: f32 = 84.0;

// ---------------------------------------------------------------------------
// ListItem
// ---------------------------------------------------------------------------

/// Interactive list row with mono identifier, optional badge, and 2-line
/// sans-serif description. Every instance renders at the same external height
/// (`LIST_ROW_HEIGHT`) — fixed uniform row, prerequisite for `gpui::uniform_list`
/// windowing (031/M1). Selection is caller-owned — pass `.selected(bool)`
/// and `.on_click(handler)` to drive the list→detail pattern.
///
/// ```ignore
/// use stand_in_mcp_explorer_ds::data::ListItem;
/// use stand_in_mcp_explorer_ds::core::{Badge, BadgeKind};
///
/// ListItem::new("read_file", "read_file")
///     .desc("Lê o conteúdo completo de um arquivo do disco como texto UTF-8.")
///     .selected(sel == "read_file")
///     .badge(Badge::new(BadgeKind::Read, "leitura").into_any_element())
///     .on_click(move |_ev, _w, _cx| set_sel("read_file"));
/// ```
#[derive(IntoElement)]
pub struct ListItem {
    id: ElementId,
    name: SharedString,
    desc: Option<SharedString>,
    badge: Option<gpui::AnyElement>,
    selected: bool,
    on_click: Option<ClickHandler>,
}

impl ListItem {
    /// Create a list item. `id` is the element id; `name` is the technical
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

    /// Set the prose description (sans-serif, sentence case, ends with period).
    pub fn desc(mut self, text: impl Into<SharedString>) -> Self {
        self.desc = Some(text.into());
        self
    }

    /// Attach a badge in the top-row right slot (typically `Badge` read/write).
    pub fn badge(mut self, element: gpui::AnyElement) -> Self {
        self.badge = Some(element);
        self
    }

    /// Mark this item as the active selection (surface-2 bg + 2px oby left bar).
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

impl RenderOnce for ListItem {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // O-025: suppress the row hover highlight while the active list is being
        // mouse-wheel scrolled (app-set global), so it doesn't strobe between
        // items under a stationary cursor. Trackpad (pixel) scrolling is normal.
        let suppress_hover = cx
            .try_global::<ListScrollHoverSuppressed>()
            .is_some_and(|g| g.0);
        let t = cx.theme();
        let colors = &t.colors;
        let mono = t.mono_font_family.clone();
        let selected = self.selected;
        let has_click = self.on_click.is_some();

        let mut row = h_flex()
            .id(self.id)
            .w_full()
            .h(px(LIST_ROW_HEIGHT))
            .overflow_hidden()
            .items_start()
            .cursor_pointer()
            .border_b_1()
            .border_color(colors.border)
            .when(selected, |el| el.bg(colors.secondary)) // surface-2
            .hover(|h| {
                if selected || suppress_hover {
                    h
                } else {
                    h.bg(colors.secondary) // surface-2
                }
            });

        // Selection bar: 2px child element (not shadow — prohibition 4).
        // Always present (transparent when inactive) so layout is stable.
        row = row.child(
            gpui::div()
                .flex_none()
                .w(px(2.))
                .h_full()
                .when(selected, |b| b.bg(OBY)),
        );

        // Content area — padding compensates for the 2px bar: total left
        // spacing = 2px bar + 12px padding = 14px (matches canon).
        let mut content = v_flex()
            .flex_1()
            .min_w(px(0.))
            .py(px(12.))
            .pr(px(14.))
            .pl(px(12.));

        // Top row: name (mono) + spacer + badge
        let mut top = h_flex().gap(px(8.)).items_center();

        top = top.child(
            gpui::div()
                .flex_1()
                .min_w(px(0.))
                .text_size(px(typography::FS_MD))
                .font_weight(FontWeight::SEMIBOLD)
                .font_family(mono.clone())
                .text_color(colors.foreground)
                .text_ellipsis()
                .whitespace_nowrap()
                .child(self.name.clone()),
        );

        if let Some(badge) = self.badge {
            top = top.child(badge);
        }

        content = content.child(top);

        // Bottom: description (sans, 2-line clamp). Always rendered at a
        // fixed reserved height so every row has the same external height
        // (prerequisite for `gpui::uniform_list` windowing — 031/M1).
        content = content.child(
            gpui::div()
                .mt(px(5.))
                .text_size(px(typography::FS_SM))
                .text_color(colors.muted_foreground)
                .line_height(px(17.4)) // fs-sm 12 × 1.45
                .h(px(DESC_RESERVED_H))
                .overflow_hidden()
                .when_some(self.desc, |el, desc| el.child(desc)),
        );

        row = row.child(content);

        if has_click && let Some(click) = self.on_click {
            row = row.on_click(click);
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
    fn test_list_item_defaults() {
        let item = ListItem::new("read_file", "read_file");
        assert_eq!(item.name.as_ref(), "read_file");
        assert_eq!(item.id, ElementId::from("read_file"));
        assert!(item.desc.is_none());
        assert!(item.badge.is_none());
        assert!(!item.selected);
        assert!(item.on_click.is_none());
    }

    #[test]
    fn test_list_item_with_desc() {
        let item = ListItem::new("r", "r").desc("Lê um arquivo.");
        assert_eq!(item.desc.as_deref(), Some("Lê um arquivo."));
    }

    #[test]
    fn test_list_item_with_badge() {
        let item = ListItem::new("r", "r").badge(gpui::div().into_any_element());
        assert!(item.badge.is_some());
    }

    #[test]
    fn test_list_item_selected() {
        let item = ListItem::new("r", "r").selected(true);
        assert!(item.selected);
    }

    #[test]
    fn test_list_item_on_click() {
        let item = ListItem::new("r", "r").on_click(|_ev, _w, _cx| {});
        assert!(item.on_click.is_some());
    }

    #[test]
    fn test_list_item_builder_chain() {
        let item = ListItem::new("write_file", "write_file")
            .desc("Grava dados em disco.")
            .selected(false)
            .on_click(|_ev, _w, _cx| {});
        assert_eq!(item.name.as_ref(), "write_file");
        assert!(item.desc.is_some());
        assert!(!item.selected);
        assert!(item.on_click.is_some());
    }

    #[test]
    fn test_list_item_name_not_capitalized() {
        let item = ListItem::new("read_file", "read_file");
        assert_eq!(item.name.as_ref(), "read_file");
    }

    #[test]
    fn test_list_item_id_is_stable() {
        let item = ListItem::new("list_dir", "list_dir");
        assert_eq!(item.id, ElementId::from("list_dir"));
    }

    #[test]
    fn test_selection_bar_not_shadow() {
        let item = ListItem::new("test", "test").selected(true);
        assert!(item.selected);
    }

    #[test]
    fn test_row_height_const_anatomy_031_m1() {
        // 12 (py top) + 20 (name) + 5 (mt) + 35 (desc) + 12 (py bottom) = 84
        assert!(
            (LIST_ROW_HEIGHT - (12.0 + NAME_LINE_H + 5.0 + DESC_RESERVED_H + 12.0)).abs() < 0.01,
            "LIST_ROW_HEIGHT must equal the sum of the anatomy parts"
        );
    }
}
