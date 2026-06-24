//! ToggleLink — discreet link with `link` color, underlined on hover.
//!
//! Traces to the `.toggle-link` class in `core/core.css` and the "Copy / link"
//! card. No background, no border. Color = `link` (OBY), hover = `link_hover`
//! (BRISA) + underline. Weight 600, fs-sm (12).
//!
//! ```ignore
//! use stand_in_mcp_explorer_ds::core::ToggleLink;
//!
//! ToggleLink::new("add-var", "+ adicionar variável")
//!     .on_click(|_ev, _window, cx| { /* … */ });
//! ```

use gpui::{
    App, ClickEvent, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, px,
};
use gpui_component::{ActiveTheme as _, ThemeColor};

use crate::theme::typography;

use super::button::ClickHandler;

// ---------------------------------------------------------------------------
// ToggleLink
// ---------------------------------------------------------------------------

/// Discreet link — no background/border, color `link`, underline on hover.
#[derive(IntoElement)]
pub struct ToggleLink {
    label: SharedString,
    id: ElementId,
    on_click: Option<ClickHandler>,
}

impl ToggleLink {
    /// Create a ToggleLink with the given element id and label.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            id: id.into(),
            on_click: None,
        }
    }

    /// Attach a click handler.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for ToggleLink {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = cx.theme();
        let colors: &ThemeColor = &t.colors;

        let mut el = gpui::div()
            .id(self.id)
            .text_color(colors.link)
            .text_size(px(typography::FS_SM))
            .font_weight(typography::W_SEMIBOLD)
            .cursor_pointer()
            .hover(|h| h.text_color(colors.link_hover).underline())
            .child(self.label);

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
    fn test_toggle_link_construction() {
        let link = ToggleLink::new("add-var", "+ adicionar variável");
        assert_eq!(link.label.as_ref(), "+ adicionar variável");
        assert_eq!(link.id, ElementId::from("add-var"));
    }

    #[test]
    fn test_toggle_link_with_on_click() {
        let link = ToggleLink::new("toggle-demo", "Toggle").on_click(|_ev, _w, _cx| {
            // no-op for test
        });
        assert!(link.on_click.is_some());
    }

    #[test]
    fn test_toggle_link_default_id() {
        let link = ToggleLink::new("some-link", "Click me");
        assert_eq!(link.id, ElementId::from("some-link"));
    }

    #[test]
    fn test_link_font_size_is_sm() {
        assert_eq!(typography::FS_SM, 12.0);
    }
}
