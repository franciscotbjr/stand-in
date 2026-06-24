//! CountPill — round numeric pill for inline counts (e.g. env var count).
//!
//! Anatomy: inline-grid, place-items-center, min_w(18), h(18), px(5),
//! rounded(RADIUS_PILL=99), bg = OBY opaque, fg white, fs FS_2XS (10.5),
//! W_SEMIBOLD, mono. Not interactive.

use gpui::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window, px,
};
use gpui_component::ActiveTheme as _;

use crate::theme::density::RADIUS_PILL;
use crate::theme::palette;
use crate::theme::typography;

#[derive(IntoElement)]
pub struct CountPill {
    n: usize,
    id: ElementId,
}

impl CountPill {
    pub fn new(n: usize) -> Self {
        Self {
            n,
            id: ElementId::from("count-pill"),
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }
}

impl RenderOnce for CountPill {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = cx.theme();

        gpui::div()
            .id(self.id)
            .flex()
            .items_center()
            .justify_center()
            .min_w(px(18.))
            .h(px(18.))
            .px(px(5.))
            .rounded(px(RADIUS_PILL))
            .bg(palette::OBY)
            .text_color(gpui::Hsla {
                h: 0.,
                s: 0.,
                l: 1.,
                a: 1.,
            })
            .text_size(px(typography::FS_2XS))
            .font_weight(typography::W_SEMIBOLD)
            .font_family(t.mono_font_family.clone())
            .child(SharedString::from(self.n.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_pill_construction() {
        let pill = CountPill::new(3);
        assert_eq!(pill.n, 3);
    }

    #[test]
    fn test_count_pill_with_id() {
        let pill = CountPill::new(0).id("env-count");
        assert_eq!(pill.n, 0);
    }

    #[test]
    fn test_pill_geometry_constants() {
        assert_eq!(RADIUS_PILL, 99.0);
        assert_eq!(typography::FS_2XS, 10.5);
    }

    #[test]
    fn test_pill_bg_is_oby() {
        let bg = palette::OBY;
        assert_eq!(bg.a, 1.0);
    }
}
