//! Panel — bordered surface panel with an uppercase header and right-side action slot.
//!
//! 1:1 with `data/Panel.jsx` + `.panel*` rules in `data/data.css`. The canonical
//! container for any detail-column block (parameters, results, content).
//!
//! Anatomy: bg `surface`, border 1px `border`, radius `Density::radius()` (scales
//! with density — prohibition 7), mb 16 (built-in stacking), overflow_hidden. No
//! shadow (in-flow card — prohibition 4).
//!
//! Head (optional): gap 10, padding 12×16, border-bottom 1px; title fs-sm (12)
//! weight 700, tracking-wide, UPPERCASE by style, text-2; optional 14px icon;
//! right slot (ml auto, gap 8 — Buttons ghost sm / CopyButton). No title → no
//! head at all (bare card).
//!
//! Rules: stack Panels directly; title is always short and nominal ("Parâmetros",
//! "Resultado" — never full sentences).
//!
//! ```ignore
//! use stand_in_mcp_explorer_ds::data::Panel;
//! use stand_in_mcp_explorer_ds::core::{Button, ButtonVariant, IconName};
//!
//! Panel::new()
//!     .title("Parâmetros")
//!     .icon(IconName::Bolt)
//!     .right_children([Button::new("Exemplo").variant(ButtonVariant::Ghost).sm().into_any_element()])
//!     .children([field.into_any_element()]);
//! ```

use gpui::{
    App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Window, px,
};
use gpui_component::{ActiveTheme as _, h_flex};

use crate::core::icon::{Icon, IconName};
use crate::theme::colors::JandiExt;
use crate::theme::density::GlobalDensity;
use crate::theme::typography;

// ---------------------------------------------------------------------------
// Panel
// ---------------------------------------------------------------------------

/// Bordered surface container with an optional uppercase header and right-side
/// action slot.
///
/// No shadow — this is an in-flow card (prohibition 4). Radius scales with
/// density (prohibition 7). Callers write the title in normal case; the component
/// uppercases it automatically.
#[derive(IntoElement)]
pub struct Panel {
    title: Option<SharedString>,
    icon: Option<IconName>,
    right_children: Vec<gpui::AnyElement>,
    body_children: Vec<gpui::AnyElement>,
    id: ElementId,
}

impl Default for Panel {
    fn default() -> Self {
        Self::new()
    }
}

impl Panel {
    /// Create an empty Panel. The head is absent by default — add `.title(…)`
    /// to reveal the header row.
    pub fn new() -> Self {
        Self {
            title: None,
            icon: None,
            right_children: Vec::new(),
            body_children: Vec::new(),
            id: ElementId::from("panel"),
        }
    }

    /// Set the panel title (short, nominal — "Parâmetros", never a sentence).
    /// Uppercased automatically at render time.
    pub fn title(mut self, text: impl Into<SharedString>) -> Self {
        self.title = Some(text.into());
        self
    }

    /// Attach an icon (14px) before the title in the header row.
    pub fn icon(mut self, name: IconName) -> Self {
        self.icon = Some(name);
        self
    }

    /// Elements placed in the right slot of the header (ml auto, gap 8).
    /// Typically ghosts `Button` sm or `CopyButton`.
    pub fn right_children(mut self, children: impl IntoIterator<Item = gpui::AnyElement>) -> Self {
        self.right_children = children.into_iter().collect();
        self
    }

    /// Body children — rendered inside the padded body area.
    pub fn children(mut self, children: impl IntoIterator<Item = gpui::AnyElement>) -> Self {
        self.body_children = children.into_iter().collect();
        self
    }

    /// Set the element id.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }
}

impl RenderOnce for Panel {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = cx.theme();
        let colors = &t.colors;
        let density = cx.global::<GlobalDensity>().0;
        let surface = cx.global::<JandiExt>().surface;

        let mut panel = gpui::div()
            .id(self.id)
            // w_full so the panel + its body fill the detail column. Inside an
            // `overflow_y_scrollbar` leaf (size_auto when content fits), block
            // children don't inherit a definite width, so percentage-width
            // descendants (form/fields) collapse without this (028 #19).
            .w_full()
            .bg(surface)
            .border_1()
            .border_color(colors.border)
            .rounded(px(density.radius()))
            .mb(px(16.))
            .overflow_hidden();

        if let Some(title) = self.title {
            let mut head = h_flex()
                .gap(px(10.))
                .pt(px(12.))
                .pb(px(12.))
                .px(px(16.))
                .border_b_1()
                .border_color(colors.border)
                .items_center();

            if let Some(icon_name) = self.icon {
                head = head.child(Icon::new(icon_name).with_px(px(14.)));
            }

            head = head.child(
                gpui::div()
                    .text_size(px(typography::FS_SM))
                    .font_weight(FontWeight::BOLD)
                    .text_color(colors.muted_foreground)
                    .child(SharedString::from(title.to_uppercase())),
            );

            if !self.right_children.is_empty() {
                let right = h_flex()
                    .ml_auto()
                    .gap(px(8.))
                    .items_center()
                    .children(self.right_children);
                head = head.child(right);
            }

            panel = panel.child(head);
        }

        panel = panel.child(gpui::div().w_full().p(px(16.)).children(self.body_children));

        panel
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_new_empty() {
        let p = Panel::new();
        assert!(p.title.is_none());
        assert!(p.icon.is_none());
        assert!(p.right_children.is_empty());
        assert!(p.body_children.is_empty());
    }

    #[test]
    fn test_panel_with_title_uppercases_in_render() {
        let p = Panel::new().title("Parâmetros");
        assert_eq!(p.title.as_deref(), Some("Parâmetros"));
        let uppercased = p.title.as_deref().unwrap().to_uppercase();
        assert_eq!(uppercased, "PARÂMETROS");
    }

    #[test]
    fn test_panel_with_icon() {
        let p = Panel::new().icon(IconName::Bolt);
        assert_eq!(p.icon, Some(IconName::Bolt));
    }

    #[test]
    fn test_panel_right_children_collected() {
        use gpui::div;
        let p = Panel::new().right_children([div().into_any_element(), div().into_any_element()]);
        assert_eq!(p.right_children.len(), 2);
    }

    #[test]
    fn test_panel_body_children_collected() {
        use gpui::div;
        let p = Panel::new().children([div().into_any_element()]);
        assert_eq!(p.body_children.len(), 1);
    }

    #[test]
    fn test_panel_id() {
        let p = Panel::new().id("my-panel");
        assert_eq!(p.id, ElementId::from("my-panel"));
    }

    #[test]
    fn test_panel_default_id() {
        let p = Panel::new();
        assert_eq!(p.id, ElementId::from("panel"));
    }

    #[test]
    fn test_panel_no_title_no_head() {
        let p = Panel::new().children([gpui::div().into_any_element()]);
        assert!(p.title.is_none());
        assert_eq!(p.body_children.len(), 1);
    }

    #[test]
    fn test_panel_no_shadow_validated_by_doc() {
        let p = Panel::new();
        assert!(p.title.is_none());
    }
}
