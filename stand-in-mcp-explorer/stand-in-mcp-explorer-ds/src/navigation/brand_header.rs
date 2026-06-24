//! BrandHeader — the app brand identity: a 34×34 gradient brand-mark + name + sub.
//!
//! Extracted from `SidebarShell` so the brand can live either in the sidebar OR
//! in the app header row (the `.app` grid header). The 34×34 mark carries the
//! second legitimate gradient (prohibition 5) — owned **here**, never duplicated.
//!
//! Content only: `h_flex` gap 11, items_center (mark + name/sub). The caller
//! wraps it with the height/padding/borders for its context (sidebar brand zone
//! vs header cell).
//!
//! ```ignore
//! use stand_in_mcp_explorer_ds::navigation::BrandHeader;
//!
//! BrandHeader::new()
//!     .mark(Icon::new(IconName::Leaf).with_px(px(18.)))
//!     .name("MCP Explorer")
//!     .subtitle("MCP \u{b7} local-first");
//! ```

use gpui::{
    AnyElement, App, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Window, div, linear_color_stop, linear_gradient, px,
};
use gpui_component::{ActiveTheme as _, h_flex};

use crate::theme::density::RADIUS_BTN;
use crate::theme::palette;
use crate::theme::typography;

// ---------------------------------------------------------------------------
// BrandHeader
// ---------------------------------------------------------------------------

/// The brand identity block: 34×34 gradient mark (+ icon slot) + name + sub.
///
/// Renders **content only** (an `h_flex`); the caller owns the surrounding
/// height/padding/borders so the same brand works in the sidebar and in the
/// app header row.
#[derive(IntoElement)]
pub struct BrandHeader {
    mark: Option<AnyElement>,
    name: SharedString,
    sub: Option<SharedString>,
}

impl BrandHeader {
    /// Create an empty brand header. Call `.mark/.name/.sub`.
    pub fn new() -> Self {
        Self {
            mark: None,
            name: "".into(),
            sub: None,
        }
    }

    /// Element rendered inside the 34×34 brand mark (typically an Icon 18px).
    pub fn mark(mut self, el: impl IntoElement) -> Self {
        self.mark = Some(el.into_any_element());
        self
    }

    /// Brand name (fs-xl 15, weight 700, nowrap/ellipsis).
    pub fn name(mut self, name: impl Into<SharedString>) -> Self {
        self.name = name.into();
        self
    }

    /// Subtitle below the name (fs-xs, text-3). Optional.
    pub fn subtitle(mut self, sub: impl Into<SharedString>) -> Self {
        self.sub = Some(sub.into());
        self
    }
}

impl Default for BrandHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for BrandHeader {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = cx.theme();

        // --- Brand mark (34×34, gradient + inset ring) ---
        // Canon: linear-gradient(150deg, oby, genipina 60%, yandi). Pinned gpui
        // supports 2 stops; approximate with oby→yandi (the lost genipina mid-stop
        // is a minor fidelity gap). The brand colour is still the gradient of the
        // two legitimate system colours (prohibition 5 — owned here).
        let brand_mark_div = div()
            .id("brand-mark")
            .w(px(34.))
            .h(px(34.))
            .flex_none()
            .rounded(px(RADIUS_BTN))
            .bg(linear_gradient(
                150.0,
                linear_color_stop(palette::OBY, 0.0),
                linear_color_stop(palette::YANDI, 1.0),
            ))
            .border_1()
            .border_color(palette::BRAND_RING)
            .flex()
            .items_center()
            .justify_center()
            .text_color(t.colors.primary_foreground);

        let brand_mark = match self.mark {
            Some(el) => brand_mark_div.child(el),
            None => brand_mark_div,
        };

        // --- Brand text column ---
        let mut brand_text_col = div().id("brand-text").min_w(px(0.)).child(
            div()
                .id("brand-name")
                .text_size(px(typography::FS_XL))
                .font_weight(FontWeight::BOLD)
                .font_family(typography::sans_family())
                .line_height(px(1.0))
                .text_ellipsis()
                .child(self.name.clone()),
        );

        if let Some(sub) = &self.sub {
            brand_text_col = brand_text_col.child(
                div()
                    .id("brand-sub")
                    .text_size(px(typography::FS_XS))
                    .text_color(t.muted_foreground)
                    .mt(px(3.))
                    .child(sub.clone()),
            );
        }

        h_flex()
            .id("brand")
            .gap(px(11.))
            .items_center()
            .child(brand_mark)
            .child(brand_text_col)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brand_header_defaults() {
        let b = BrandHeader::new();
        assert!(b.mark.is_none());
        assert!(b.name.is_empty());
        assert!(b.sub.is_none());
    }

    #[test]
    fn test_brand_header_setters() {
        let b = BrandHeader::new()
            .name("MCP Explorer")
            .subtitle("MCP \u{b7} local-first");
        assert_eq!(b.name.as_ref(), "MCP Explorer");
        assert_eq!(b.sub.as_deref(), Some("MCP \u{b7} local-first"));
    }

    #[test]
    fn test_brand_header_default_trait() {
        let b = BrandHeader::default();
        assert!(b.name.is_empty());
    }

    #[test]
    fn test_constants_match_canon() {
        assert_eq!(typography::FS_XL, 15.0);
        assert_eq!(typography::FS_XS, 11.0);
        assert_eq!(RADIUS_BTN, 9.0);
    }
}
