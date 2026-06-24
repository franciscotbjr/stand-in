//! IconButton — 32×32 square icon-only button for compact secondary actions.
//!
//! 1:1 with `core/IconButton.jsx` + the `.icon-btn` rules in `core/core.css`.
//! Fixed semantics (never reinterpret): **never** for the primary action of a
//! view — that is `Button::Primary`'s job. Hover: only changes icon colour and
//! border; no sudden background shift. The `label` is REQUIRED (accessibility;
//! stored for future tooltip / aria-label use). Icon size = 14px (Sm).
//!
//! Geometry: 32×32 fixed, bg surface-2, border 1px border, icon text-2 by
//! default, radius RADIUS_CHIP = 7.

use gpui::{
    App, ClickEvent, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, div, px,
};
use gpui_component::ActiveTheme as _;

use super::button::ClickHandler;
use super::icon::{Icon, IconName, IconSize};
use crate::theme::colors::JandiExt;
use crate::theme::density::RADIUS_CHIP;

/// The fixed icon size inside the 32×32 IconButton.
const ICON_SIZE: IconSize = IconSize::Sm; // 14px

// ---------------------------------------------------------------------------
// IconButton
// ---------------------------------------------------------------------------

/// 32×32 square icon-only button. Always pass a label for accessibility.
///
/// ```ignore
/// use stand_in_mcp_explorer_ds::core::{IconButton, IconName};
///
/// IconButton::new(IconName::X, "Remover")
///     .id("remove-row-btn");
/// ```
#[derive(IntoElement)]
pub struct IconButton {
    icon_name: IconName,
    /// Accessibility label (required; stored for future tooltip/aria-label support).
    #[allow(dead_code)]
    label: SharedString,
    id: ElementId,
    on_click: Option<ClickHandler>,
}

impl IconButton {
    /// Create an IconButton. The `label` is mandatory (accessibility).
    pub fn new(icon: IconName, label: impl Into<SharedString>) -> Self {
        Self {
            icon_name: icon,
            label: label.into(),
            id: ElementId::from("icon-button"),
            on_click: None,
        }
    }

    /// Set the element id (required for interactivity).
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    /// Attach a click handler.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    /// Attach a pre-boxed click handler (used by composed components that
    /// forward handlers, e.g. KeyValueRow).
    pub fn on_click_boxed(mut self, handler: ClickHandler) -> Self {
        self.on_click = Some(handler);
        self
    }
}

impl RenderOnce for IconButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = cx.theme();
        let colors = &t.colors;
        let ext = cx.global::<JandiExt>();
        let size_px = 32.0_f32;

        // gpui's Div defaults to display:block — flex must be explicit or
        // items_center/justify_center are inert (icon lands top-left).
        let mut el = div()
            .id(self.id)
            .flex()
            .size(px(size_px)) // 32×32 fixed
            .rounded(px(RADIUS_CHIP)) // 7
            .flex_none()
            .items_center()
            .justify_center()
            .bg(colors.secondary) // surface-2
            .border_1()
            .border_color(colors.border)
            .text_color(colors.secondary_foreground) // text-2
            .cursor_pointer()
            // Hover: icon steps to text, border steps to border-2.
            // No sudden bg shift (canon rule).
            .hover(|h| h.text_color(colors.foreground).border_color(ext.border_2))
            .child(Icon::new(self.icon_name).size(ICON_SIZE));

        if let Some(click) = self.on_click {
            el = el.on_click(click);
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
    fn test_icon_button_construction() {
        let btn = IconButton::new(IconName::X, "Remover");
        assert_eq!(btn.icon_name, IconName::X);
        assert_eq!(btn.label.as_ref(), "Remover");
        assert_eq!(btn.id, ElementId::from("icon-button"));
    }

    #[test]
    fn test_icon_button_builder() {
        let btn = IconButton::new(IconName::Refresh, "Reconectar").id("refresh-btn");
        assert_eq!(btn.id, ElementId::from("refresh-btn"));
    }

    #[test]
    fn test_icon_size_is_sm_14px() {
        assert_eq!(ICON_SIZE, IconSize::Sm);
        assert_eq!(ICON_SIZE.pixels(), 14.0);
    }
}
