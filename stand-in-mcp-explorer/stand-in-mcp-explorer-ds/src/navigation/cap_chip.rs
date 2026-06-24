//! CapChip — small mono chip summarising capability/count in the topbar.
//!
//! 1:1 with `navigation/CapChip.jsx` + `.cap` in `navigation/navigation.css`.
//! The number is bold + full text colour (from `<b>`), the label is muted.
//! Always mono. Not a button — use `Button` ghost sm for actions alongside
//! the chip.
//!
//! Anatomy: h_flex (inline-flex), gap 6, fs-xs (11), weight 600,
//! padding 5×9, radius RADIUS_CHIP (7), bg surface-2, fg text-2,
//! border 1px border, mono always. Optional count (bold + foreground colour)
//! rendered before the label. Optional icon 12px.
//!
//! The chip lives in the right slot of the Topbar inside `.caps` (gap 7).
//!
//! ```ignore
//! use stand_in_mcp_explorer_ds::navigation::CapChip;
//!
//! CapChip::new("tools").count(6).icon(IconName::Tool)
//! CapChip::new("Portugu\u{ea}s").icon(IconName::Globe)
//! ```

use gpui::{
    App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Window, px,
};
use gpui_component::ActiveTheme as _;

use crate::core::icon::{Icon, IconName, IconSize};
use crate::theme::density::RADIUS_CHIP;
use crate::theme::typography;

// ---------------------------------------------------------------------------
// CapChip
// ---------------------------------------------------------------------------

/// A small mono chip that summarises a capability count or metadata label.
///
/// The chip is **never interactive** — use a `Button` ghost sm for actions
/// placed alongside it.
#[derive(IntoElement)]
pub struct CapChip {
    label: SharedString,
    count: Option<usize>,
    icon: Option<IconName>,
    id: ElementId,
}

impl CapChip {
    /// Create a capability chip with the given label (e.g. "tools",
    /// "resources", "Português").
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            count: None,
            icon: None,
            id: ElementId::from("cap-chip"),
        }
    }

    /// Set a count highlighted in bold + full text colour before the label.
    pub fn count(mut self, n: usize) -> Self {
        self.count = Some(n);
        self
    }

    /// Attach an icon (12px) before the count and label.
    pub fn icon(mut self, name: IconName) -> Self {
        self.icon = Some(name);
        self
    }

    /// Set the element id.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }
}

impl RenderOnce for CapChip {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = cx.theme();

        // gpui's Div defaults to display:block — flex must be explicit or
        // gap/items_center are inert and icon/count/label stack vertically.
        let mut row = gpui::div()
            .id(self.id)
            .flex()
            .bg(t.secondary)
            .border_1()
            .border_color(t.border)
            .rounded(px(RADIUS_CHIP))
            .px(px(9.))
            .py(px(5.))
            .gap(px(6.))
            .text_size(px(typography::FS_XS))
            .font_weight(FontWeight::SEMIBOLD)
            .font_family(t.mono_font_family.clone())
            .text_color(t.secondary_foreground)
            .items_center();

        if let Some(icon_name) = self.icon {
            row = row.child(Icon::new(icon_name).size(IconSize::Xs));
        }

        if let Some(n) = self.count {
            row = row.child(
                gpui::div()
                    .font_weight(FontWeight::BOLD)
                    .text_color(t.foreground)
                    .child(SharedString::from(n.to_string())),
            );
        }

        row.child(self.label)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cap_chip_new_no_count_no_icon() {
        let chip = CapChip::new("tools");
        assert_eq!(chip.label.as_ref(), "tools");
        assert!(chip.count.is_none());
        assert!(chip.icon.is_none());
    }

    #[test]
    fn test_cap_chip_with_count() {
        let chip = CapChip::new("tools").count(6);
        assert_eq!(chip.count, Some(6));
    }

    #[test]
    fn test_cap_chip_with_icon() {
        let chip = CapChip::new("tools").icon(IconName::Tool);
        assert_eq!(chip.icon, Some(IconName::Tool));
    }

    #[test]
    fn test_cap_chip_with_icon_and_count() {
        let chip = CapChip::new("tools").count(6).icon(IconName::Tool);
        assert_eq!(chip.count, Some(6));
        assert_eq!(chip.icon, Some(IconName::Tool));
    }

    #[test]
    fn test_cap_chip_no_count_label_only() {
        let chip = CapChip::new("Portugu\u{ea}s").icon(IconName::Globe);
        assert!(chip.count.is_none());
        assert_eq!(chip.label.as_ref(), "Portugu\u{ea}s");
    }

    #[test]
    fn test_cap_chip_id() {
        let chip = CapChip::new("tools").id("my-chip");
        assert_eq!(chip.id, ElementId::from("my-chip"));
    }

    #[test]
    fn test_cap_chip_default_id() {
        let chip = CapChip::new("tools");
        assert_eq!(chip.id, ElementId::from("cap-chip"));
    }

    #[test]
    fn test_constants_match_canon() {
        assert_eq!(RADIUS_CHIP, 7.0);
        assert_eq!(typography::FS_XS, 11.0);
    }

    #[test]
    fn test_count_formatting() {
        let chip = CapChip::new("tools").count(42);
        assert_eq!(chip.count.unwrap().to_string(), "42");
    }
}
