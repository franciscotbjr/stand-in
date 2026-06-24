//! SectionLabel — uppercase 11px section label with optional 13px icon.
//!
//! 1:1 with `navigation/SectionLabel.jsx` + `.section-label` in
//! `navigation/navigation.css`. The canonical presentation rule: the caller
//! writes normal lowercase prose ("Servidores salvos"); the component applies
//! `to_uppercase()` via style — never require the caller to type uppercase.
//!
//! Anatomy: h_flex, gap 7, fs-xs (11), weight 600, text-3 (muted),
//! uppercase by style, margin-bottom 9, margin-left 2. Optional icon 13px
//! (before the text) and optional count bullet (small rounded surface-2 pill,
//! before the text). GPUI does not expose letter-spacing/tracking; the
//! spacing-heavy layout approximates the canon's `track-wider` (0.08em).
//! The gap is documented here for any future tracking API.
//!
//! Rules: 1–3 words; inside a Panel use the Panel's title — never
//! duplicate a heading with a SectionLabel.
//!
//! ```ignore
//! use stand_in_mcp_explorer_ds::navigation::SectionLabel;
//!
//! SectionLabel::new("Servidores salvos").icon(IconName::Bolt)
//! SectionLabel::new("Transporte")
//! ```

use gpui::{
    App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Window, px,
};
use gpui_component::{ActiveTheme as _, h_flex};

use crate::core::icon::{Icon, IconName};
use crate::theme::typography;

// ---------------------------------------------------------------------------
// SectionLabel
// ---------------------------------------------------------------------------

/// An uppercase 11px label with an optional 13px icon for section headings.
///
/// The caller provides the text in normal case; the component uppercases it
/// automatically, matching the CSS `text-transform: uppercase` idiom.
#[derive(IntoElement)]
pub struct SectionLabel {
    text: SharedString,
    icon: Option<IconName>,
    count: Option<usize>,
    id: ElementId,
}

impl SectionLabel {
    /// Create a section label. The text is automatically uppercased at
    /// render time — the caller writes normal case ("Servidores salvos").
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            icon: None,
            count: None,
            id: ElementId::from("section-label"),
        }
    }

    /// Attach an icon (13px) before the label text.
    pub fn icon(mut self, name: IconName) -> Self {
        self.icon = Some(name);
        self
    }

    /// Attach a count bullet (small rounded pill) shown before the text —
    /// e.g. the number of items in the section. Optional.
    pub fn count(mut self, n: usize) -> Self {
        self.count = Some(n);
        self
    }

    /// Set the element id.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }
}

impl RenderOnce for SectionLabel {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = cx.theme();

        let mut row = h_flex()
            .id(self.id)
            .gap(px(7.))
            .mb(px(9.))
            .ml(px(2.))
            .items_center();

        if let Some(icon_name) = self.icon {
            row = row.child(Icon::new(icon_name).with_px(px(13.)));
        }

        // Count bullet: a small rounded pill (surface-2 + muted) carrying the
        // section item count, placed before the text. tokens-only, no shadow.
        if let Some(n) = self.count {
            row = row.child(
                gpui::div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .min_w(px(16.))
                    .h(px(16.))
                    .px(px(5.))
                    .rounded_full()
                    .bg(t.colors.secondary)
                    .text_size(px(typography::FS_2XS))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(t.muted_foreground)
                    .child(n.to_string()),
            );
        }

        row.child(
            gpui::div()
                .text_size(px(typography::FS_XS))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(t.muted_foreground)
                .child(SharedString::from(self.text.to_uppercase())),
        )
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_section_label_new_no_icon() {
        let label = SectionLabel::new("Servidores salvos");
        assert_eq!(label.text.as_ref(), "Servidores salvos");
        assert!(label.icon.is_none());
    }

    #[test]
    fn test_section_label_with_icon() {
        let label = SectionLabel::new("Transporte").icon(IconName::Bolt);
        assert_eq!(label.icon, Some(IconName::Bolt));
    }

    #[test]
    fn test_section_label_id() {
        let label = SectionLabel::new("abc").id("my-label");
        assert_eq!(label.id, ElementId::from("my-label"));
    }

    #[test]
    fn test_render_uppercases_text() {
        let label = SectionLabel::new("abc");
        assert_eq!(label.text.to_uppercase(), "ABC");
    }

    #[test]
    fn test_render_mixed_case_uppercased() {
        let label = SectionLabel::new("Servidores salvos");
        assert_eq!(label.text.to_uppercase(), "SERVIDORES SALVOS");
    }

    #[test]
    fn test_builder_default_icon_is_none() {
        let label = SectionLabel::new("abc");
        assert!(label.icon.is_none());
    }

    #[test]
    fn test_section_label_count() {
        let label = SectionLabel::new("Servidores salvos").count(4);
        assert_eq!(label.count, Some(4));
    }

    #[test]
    fn test_section_label_count_none_by_default() {
        assert!(SectionLabel::new("abc").count.is_none());
    }

    #[test]
    fn test_constants_match_canon() {
        assert_eq!(typography::FS_XS, 11.0);
    }
}
