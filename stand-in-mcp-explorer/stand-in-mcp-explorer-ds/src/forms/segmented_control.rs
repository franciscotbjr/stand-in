//! SegmentedControl — mutually exclusive segment picker for 2–4 short options.
//!
//! 1:1 with `forms/SegmentedControl.jsx` + `.seg` rules in `forms/forms.css`.
//! One-word labels preferred (uppercase siglas for technical domains).
//! More than 4 options or long labels → use a different pattern (list, select).
//!
//! Anatomy: h_flex container (surface-2 bg, border 1px, radius RADIUS_BTN=9,
//! padding 3, gap 3) with 2–4 `flex_1` segments (fs-sm 12, weight 600, text-2,
//! padding 7×4, radius RADIUS_BADGE=6). Hover → text. Active → primary bg +
//! on-primary text.
//!
//! ## Delta vs CSS canon
//!
//! The canon `.seg button[data-on]` has `box-shadow: 0 1px 2px rgba(0,0,0,.2)`.
//! The pinned 024 rule (shadow-only-on-overlays) wins: **the active segment has
//! no shadow**. The primary background + on-primary text already distinguish the
//! active segment visually. This is a conscious delta — the elevation rule
//! overrides the pixel value (same pattern as Select/D9).
//!
//! The caller owns the selected index. Use `on_click` handlers (one per option)
//! created via `cx.listener` to implement selection change.

use gpui::{
    App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div, px,
};
use gpui_component::{ActiveTheme as _, h_flex};

use crate::core::button::ClickHandler;
use crate::theme::density::{RADIUS_BADGE, RADIUS_BTN};
use crate::theme::typography;

// ---------------------------------------------------------------------------
// SegmentedControl
// ---------------------------------------------------------------------------

/// A segmented picker for 2–4 mutually exclusive options.
///
/// The active segment gets the primary background and on-primary text.
/// Handlers are a `Vec<ClickHandler>` — one per option, created by the caller
/// via `cx.listener` so entity state updates + `cx.notify` work correctly.
///
/// ```ignore
/// use stand_in_mcp_explorer_ds::forms::SegmentedControl;
///
/// let options = vec![
///     ("stdio".into(), "STDIO".into()),
///     ("http".into(), "HTTP".into()),
/// ];
/// SegmentedControl::new("seg-transport", options, selected_ix)
///     .handlers(handlers);
/// ```
#[derive(IntoElement)]
pub struct SegmentedControl {
    id: ElementId,
    options: Vec<(gpui::SharedString, gpui::SharedString)>,
    selected_ix: usize,
    handlers: Vec<Option<ClickHandler>>,
}

impl SegmentedControl {
    /// Create a SegmentedControl.
    ///
    /// `options`: 2–4 `(value, label)` pairs. Labels should be one word each.
    /// `selected_ix`: index (0-based) of the currently active option.
    pub fn new(
        id: impl Into<ElementId>,
        options: Vec<(impl Into<gpui::SharedString>, impl Into<gpui::SharedString>)>,
        selected_ix: usize,
    ) -> Self {
        let opts: Vec<_> = options
            .into_iter()
            .map(|(v, l)| (v.into(), l.into()))
            .collect();
        let n = opts.len();
        let mut handlers = Vec::with_capacity(n);
        for _ in 0..n {
            handlers.push(None);
        }
        Self {
            id: id.into(),
            options: opts,
            selected_ix,
            handlers,
        }
    }

    /// Attach click handlers — one per option, in order.
    pub fn handlers(mut self, handlers: Vec<ClickHandler>) -> Self {
        let n = self.options.len().min(handlers.len());
        self.handlers = handlers.into_iter().take(n).map(Some).collect();
        // Pad to match option count if fewer handlers than options.
        while self.handlers.len() < self.options.len() {
            self.handlers.push(None);
        }
        self
    }
}

impl RenderOnce for SegmentedControl {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = cx.theme();
        let colors = &t.colors;
        let font = t.font_family.clone();

        // Container: surface-2, border 1px, radius RADIUS_BTN (9), padding 3, gap 3.
        let mut el = h_flex()
            .id(self.id)
            .bg(colors.secondary)
            .border_1()
            .border_color(colors.border)
            .rounded(px(RADIUS_BTN))
            .px(px(3.))
            .py(px(3.))
            .gap(px(3.));

        for (i, (label, handler)) in self
            .options
            .into_iter()
            .map(|(_, l)| l)
            .zip(self.handlers)
            .enumerate()
        {
            let is_active = i == self.selected_ix;

            // gpui's Div defaults to display:block — flex must be explicit or
            // items_center/justify_center are inert (label lands top-left).
            let mut seg = div()
                .id(("seg-btn", i))
                .flex()
                .flex_1()
                .text_size(px(typography::FS_SM))
                .font_weight(FontWeight(600.0))
                .px(px(4.))
                .py(px(7.))
                .rounded(px(RADIUS_BADGE))
                .items_center()
                .justify_center()
                .cursor_pointer()
                .font_family(font.clone())
                .child(label);

            if is_active {
                // Active: primary bg + on-primary text, NO shadow (delta vs canon CSS).
                seg = seg.bg(colors.primary).text_color(colors.primary_foreground);
            } else {
                seg = seg
                    .text_color(colors.secondary_foreground)
                    .hover(|h| h.text_color(colors.foreground));
            }

            if let Some(click) = handler {
                seg = seg.on_click(click);
            }

            el = el.child(seg);
        }

        el
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segmented_control_construction() {
        let sc = SegmentedControl::new("seg1", vec![("a", "A"), ("b", "B"), ("c", "C")], 0);
        assert_eq!(sc.options.len(), 3);
        assert_eq!(sc.selected_ix, 0);
        assert_eq!(sc.handlers.len(), 3);
        assert!(sc.handlers.iter().all(|h| h.is_none()));
    }

    #[test]
    fn test_segmented_control_id() {
        let sc = SegmentedControl::new("transport", vec![("stdio", "STDIO"), ("http", "HTTP")], 1);
        assert_eq!(sc.id, ElementId::from("transport"));
        assert_eq!(sc.selected_ix, 1);
    }

    #[test]
    fn test_two_to_four_options_allowed() {
        for n in [2, 3, 4] {
            let opts: Vec<(String, String)> =
                (0..n).map(|i| (format!("v{i}"), format!("L{i}"))).collect();
            let sc = SegmentedControl::new("test", opts, 0);
            assert_eq!(sc.options.len(), n);
            assert_eq!(sc.handlers.len(), n);
        }
    }

    #[test]
    fn test_radius_constants_match_canon() {
        assert_eq!(RADIUS_BTN, 9.0);
        assert_eq!(RADIUS_BADGE, 6.0);
    }

    #[test]
    fn test_label_is_one_word() {
        let sc = SegmentedControl::new("seg", vec![("stdio", "STDIO")], 0);
        assert_eq!(sc.options[0].1.as_ref(), "STDIO");
    }
}
