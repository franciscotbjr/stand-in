//! Spinner — 15px circular progress indicator that inherits `currentColor`.
//!
//! 1:1 with `core/Spinner.jsx` + the `.spin` rules in `core/core.css`: a 2px
//! ring — translucent track + a solid top arc — rotating 360° every 0.6s,
//! linear, infinite. Fixed at 15px in every density (canon rule: never scale).
//!
//! Drop it inside buttons (replacing the icon) or next to timings while an
//! operation runs, always with the action label and an ellipsis
//! ("Conectando…") — never a bare "Loading" (canon rule; call-site concern).
//!
//! Anatomy notes (recorded per the M4 spec):
//! - The rotating arc is an embedded SVG (`spinner/arc.svg`) — component
//!   anatomy, NOT an icon glyph; the 22-glyph catalog is untouched (the canon
//!   itself draws the spinner with CSS borders, not an icon).
//! - The canon's track is `rgba(255,255,255,.25)`; here it is the resolved
//!   spinner colour at 25% alpha — identical in dark mode and still visible in
//!   light mode (the theme-aware realization of the same intent).

use std::time::Duration;

use gpui::{
    Animation, AnimationExt as _, App, ElementId, Hsla, IntoElement, ParentElement, RenderOnce,
    Styled, Transformation, Window, div, percentage, px, svg,
};

/// Spinner diameter in px — fixed in every density (canon rule: never scale).
pub const SPINNER_SIZE: f32 = 15.0;
/// Track alpha (the translucent full ring under the arc).
pub const TRACK_OPACITY: f32 = 0.25;
/// One full rotation (canon: `jandi-spin .6s linear infinite`).
pub const ROTATION_DURATION: Duration = Duration::from_millis(600);

/// The 15px rotating spinner. Inherits the contextual text colour unless an
/// explicit `.color()` is set (the pinned gpui-component Icon pattern).
#[derive(IntoElement)]
pub struct Spinner {
    color: Option<Hsla>,
    id: ElementId,
}

impl Spinner {
    /// Create a spinner that inherits the contextual text colour.
    pub fn new() -> Self {
        Self {
            color: None,
            id: ElementId::from("spinner-rotation"),
        }
    }

    /// Explicitly set the spinner colour (the track derives from it at 25%
    /// alpha).
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    /// Override the animation element id — needed only when more than one
    /// spinner shares the same parent (element ids must be unique among
    /// siblings).
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for Spinner {
    fn render(self, window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // Resolve the colour the way the pinned gpui-component Icon does
        // (crates/ui/src/icon.rs:144): explicit override, else the current
        // text style colour.
        let color = self.color.unwrap_or_else(|| window.text_style().color);

        div()
            .relative()
            .size(px(SPINNER_SIZE))
            .flex_none()
            // Track: the full translucent ring (static).
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .size(px(SPINNER_SIZE))
                    .rounded_full()
                    .border_2()
                    .border_color(color.opacity(TRACK_OPACITY)),
            )
            // Arc: the solid top quarter, rotating 360°/0.6s linear infinite.
            .child(
                svg()
                    .absolute()
                    .top_0()
                    .left_0()
                    .size(px(SPINNER_SIZE))
                    .text_color(color)
                    .path("spinner/arc.svg")
                    .with_animation(
                        self.id,
                        Animation::new(ROTATION_DURATION).repeat(),
                        |this, delta| {
                            this.with_transformation(Transformation::rotate(percentage(delta)))
                        },
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_geometry_constants() {
        assert_eq!(SPINNER_SIZE, 15.0);
        assert_eq!(TRACK_OPACITY, 0.25);
        assert_eq!(ROTATION_DURATION, Duration::from_millis(600));
    }

    #[test]
    fn test_spinner_color_override() {
        let s = Spinner::new();
        assert!(s.color.is_none());
        let c = Hsla {
            h: 0.5,
            s: 0.5,
            l: 0.5,
            a: 1.0,
        };
        let s = s.color(c);
        assert_eq!(s.color, Some(c));
    }
}
