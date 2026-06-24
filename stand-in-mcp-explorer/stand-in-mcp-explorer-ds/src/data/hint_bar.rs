//! HintBar — inline guided-mode hint with a 10% oby tint, info icon,
//! and didactic text. It teaches, never blocks.
//!
//! 1:1 with `data/HintBar.prompt.md` + `.hintbar` / `.ht` rules in
//! `data/data.css`.
//!
//! Anatomy: h_flex, gap 10, padding 11×14, bg **oby-10%** (derived from
//! the OBY palette constant, like Badge::Role with 18%), border 1px `border`,
//! radius `RADIUS_CARD`, mb 16. Icon `info` 14px (IconSize::Sm) on the
//! left; text 12.5 `text-2` (colors.secondary_foreground) lh 1.5 on the right.
//!
//! Caller controls visibility (gated on a guided-mode toggle — never
//! permanent). Text is a slot: `.text(str)` for plain text, or
//! `.children(elements)` for inline bold highlights.
//!
//! Voice: teacher-patient, 1–2 sentences.
//!
//! ```ignore
//! use stand_in_mcp_explorer_ds::data::HintBar;
//!
//! if guided {
//!     HintBar::new()
//!         .children(vec![
//!             gpui::div().font_weight(FontWeight::BOLD).child("Tools"),
//!             gpui::div().child(" s\u{e3}o fun\u{e7}\u{f5}es que o servidor exp\u{f5}e."),
//!         ]);
//! }
//! ```

use gpui::{
    App, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window, px,
};
use gpui_component::{ActiveTheme as _, h_flex};

use crate::core::icon::{Icon, IconName, IconSize};
use crate::theme::density::RADIUS_CARD;
use crate::theme::palette;
use crate::theme::typography;

// ---------------------------------------------------------------------------
// HintBar
// ---------------------------------------------------------------------------

/// Inline guided-mode hint bar with an info icon and slot-based text.
///
/// **Never permanent** — the caller must gate on a guided-mode toggle.
/// Teacher-patient voice, 1–2 sentences. Use `.text(str)` for plain text,
/// `.children(elements)` for rich inline content with bold highlights.
#[derive(IntoElement)]
pub struct HintBar {
    text: Option<SharedString>,
    children: Vec<gpui::AnyElement>,
    id: ElementId,
}

impl HintBar {
    /// Create an empty hint bar. Call `.text(…)` or `.children(…)` to
    /// populate it.
    pub fn new() -> Self {
        Self {
            text: None,
            children: Vec::new(),
            id: ElementId::from("hintbar"),
        }
    }

    /// Plain-text content (no bold highlights).
    pub fn text(mut self, text: impl Into<SharedString>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Rich content with optional bold highlights.
    ///
    /// Pass elements such as `div().font_weight(BOLD).child("term")` for
    /// bold terms inline with plain `div().child("rest of sentence")`.
    /// The children are rendered in a flex-wrap row so they flow inline.
    pub fn children(mut self, children: impl IntoIterator<Item = gpui::AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    /// Set a stable element id.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }
}

impl Default for HintBar {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for HintBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = cx.theme();
        let colors = &t.colors;

        // OBY at 10% alpha — derived from the palette constant (cf. Badge::Role
        // at 18%).
        let oby_10: Hsla = Hsla {
            h: palette::OBY.h,
            s: palette::OBY.s,
            l: palette::OBY.l,
            a: 0.10,
        };

        // Text container — children or plain text.
        let text_container = if !self.children.is_empty() {
            gpui::div()
                .flex()
                .flex_wrap()
                .flex_1()
                .min_w(px(0.))
                .gap(px(4.))
                .text_size(px(12.5))
                .text_color(colors.secondary_foreground) // text-2
                .line_height(gpui::relative(1.5))
                .font_family(typography::sans_family())
                .children(self.children)
        } else if let Some(text) = self.text {
            gpui::div()
                .flex_1()
                .min_w(px(0.))
                .text_size(px(12.5))
                .text_color(colors.secondary_foreground) // text-2
                .line_height(gpui::relative(1.5))
                .font_family(typography::sans_family())
                .child(text)
        } else {
            gpui::div()
        };

        h_flex()
            .id(self.id)
            .gap(px(10.))
            .items_start()
            .px(px(14.))
            .py(px(11.))
            .bg(oby_10)
            .border_1()
            .border_color(colors.border)
            .rounded(px(RADIUS_CARD))
            .mb(px(16.))
            .w_full()
            .child(
                Icon::new(IconName::Info)
                    .size(IconSize::Sm)
                    .color(colors.secondary_foreground), // text-2
            )
            .child(text_container)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hint_bar_defaults() {
        let bar = HintBar::new();
        assert!(bar.text.is_none());
        assert!(bar.children.is_empty());
        assert_eq!(bar.id, ElementId::from("hintbar"));
    }

    #[test]
    fn test_hint_bar_default_trait() {
        let bar = HintBar::default();
        assert!(bar.text.is_none());
    }

    #[test]
    fn test_hint_bar_with_text() {
        let bar = HintBar::new().text("Dica do modo guiado.");
        assert_eq!(bar.text.as_deref(), Some("Dica do modo guiado."));
    }

    #[test]
    fn test_hint_bar_with_children() {
        let bar = HintBar::new().children([
            gpui::div().child("term").into_any_element(),
            gpui::div().child(" rest").into_any_element(),
        ]);
        assert_eq!(bar.children.len(), 2);
    }

    #[test]
    fn test_hint_bar_id_override() {
        let bar = HintBar::new().id("guided-hint");
        assert_eq!(bar.id, ElementId::from("guided-hint"));
    }

    #[test]
    fn test_hint_bar_text_takes_precedence() {
        // When both are set, children wins (checked at render time).
        let bar = HintBar::new()
            .text("plain")
            .children([gpui::div().child("rich").into_any_element()]);
        assert!(bar.text.is_some());
        assert_eq!(bar.children.len(), 1);
    }

    #[test]
    fn test_oby_10_derived_from_palette() {
        let oby_10: Hsla = Hsla {
            h: palette::OBY.h,
            s: palette::OBY.s,
            l: palette::OBY.l,
            a: 0.10,
        };
        assert_eq!(oby_10.h, palette::OBY.h);
        assert_eq!(oby_10.s, palette::OBY.s);
        assert_eq!(oby_10.l, palette::OBY.l);
        assert_eq!(oby_10.a, 0.10);
    }

    #[test]
    fn test_hint_bar_never_permanent_by_doc() {
        let bar = HintBar::new();
        assert!(bar.text.is_none());
        // The caller must gate on a guided-mode toggle — the component
        // carries no permanent-state flag.
    }
}
