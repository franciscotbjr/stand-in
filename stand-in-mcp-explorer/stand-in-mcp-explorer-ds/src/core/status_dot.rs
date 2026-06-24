//! StatusDot — the fixed connection-state vocabulary (on / off / busy / err).
//!
//! 9px dot with a 4px translucent halo ring, 1:1 with `core/StatusDot.jsx` and
//! the `.dot` rules in `core/core.css`. Fixed semantics (never reinterpret):
//! `on` = green/connected · `off` = grey/inactive (no halo) · `busy` =
//! amber/working (the ONLY state that pulses) · `err` = red/error.
//!
//! The dot is 9px in every density (the canon forbids resizing it) and must
//! always be accompanied by a textual label at the call site — it is never the
//! only indicator.
//!
//! The halo is a composed ring (an absolutely-positioned 17px circle painted
//! behind the dot), NOT a drop shadow — it overflows the 9px layout footprint
//! exactly like the canon's `box-shadow: 0 0 0 4px` (which never affects
//! layout).

use std::time::Duration;

use gpui::prelude::FluentBuilder as _;
use gpui::{
    Animation, AnimationExt as _, App, ElementId, Hsla, IntoElement, ParentElement, RenderOnce,
    Styled, Window, div, px,
};
use gpui_component::{ActiveTheme as _, ThemeColor};

use crate::theme::colors::JandiExt;

/// Dot diameter in px — fixed in every density (canon rule: never resize).
pub const DOT_SIZE: f32 = 9.0;
/// Halo ring width in px (the translucent `<state>-dim` ring).
pub const HALO_WIDTH: f32 = 4.0;
/// Pulse cycle for the `busy` state (canon: `jandi-pulse 1s infinite`).
pub const PULSE_DURATION: Duration = Duration::from_secs(1);

/// Connection state — the fixed vocabulary. Never reinterpret the colours.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DotState {
    /// Connected — green (`ok`) with halo.
    On,
    /// Inactive — grey (`text-3`), no halo.
    Off,
    /// Working — amber (`warn`) with halo; the only state that pulses.
    Busy,
    /// Error — red (`err`) with halo.
    Err,
}

impl DotState {
    /// The dot fill colour for this state, from the installed theme.
    pub fn fill(&self, colors: &ThemeColor) -> Hsla {
        match self {
            DotState::On => colors.success,
            DotState::Off => colors.muted_foreground,
            DotState::Busy => colors.warning,
            DotState::Err => colors.danger,
        }
    }

    /// The halo ring colour (`<state>-dim`), or `None` for `off` (no halo).
    pub fn halo(&self, ext: &JandiExt) -> Option<Hsla> {
        match self {
            DotState::On => Some(ext.ok_dim),
            DotState::Off => None,
            DotState::Busy => Some(ext.warn_dim),
            DotState::Err => Some(ext.err_dim),
        }
    }

    /// Whether this state animates. The pulse belongs to `busy` ONLY (canon).
    pub const fn pulses(&self) -> bool {
        matches!(self, DotState::Busy)
    }
}

/// Triangle-wave pulse opacity: 1.0 at the cycle edges, 0.35 at the midpoint —
/// the GPUI realization of `@keyframes jandi-pulse { 50% { opacity: .35 } }`.
pub(crate) fn pulse_opacity(delta: f32) -> f32 {
    1.0 - 0.65 * (1.0 - (2.0 * delta - 1.0).abs())
}

/// The 9px connection-state dot. See the module docs for the fixed vocabulary.
#[derive(IntoElement)]
pub struct StatusDot {
    state: DotState,
    id: ElementId,
}

impl StatusDot {
    /// Create a dot for the given state.
    pub fn new(state: DotState) -> Self {
        Self {
            state,
            id: ElementId::from("status-dot-pulse"),
        }
    }

    /// Override the animation element id — needed only when more than one
    /// `busy` dot shares the same parent (element ids must be unique among
    /// siblings).
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }
}

impl RenderOnce for StatusDot {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let fill = self.state.fill(&cx.theme().colors);
        let halo = self.state.halo(cx.global::<JandiExt>());

        let dot = div()
            .relative()
            .size(px(DOT_SIZE))
            .flex_none()
            .when_some(halo, |this, h| {
                // Painted before the dot child => sits behind it; overflows the
                // 9px footprint without affecting layout (like the canon's
                // box-shadow halo).
                this.child(
                    div()
                        .absolute()
                        .top(px(-HALO_WIDTH))
                        .left(px(-HALO_WIDTH))
                        .size(px(DOT_SIZE + 2.0 * HALO_WIDTH))
                        .rounded_full()
                        .bg(h),
                )
            })
            .child(div().size_full().rounded_full().bg(fill));

        if self.state.pulses() {
            dot.with_animation(
                self.id,
                Animation::new(PULSE_DURATION).repeat(),
                |this, delta| this.opacity(pulse_opacity(delta)),
            )
            .into_any_element()
        } else {
            dot.into_any_element()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::colors::jandi_theme;
    use crate::theme::palette;
    use gpui_component::ThemeMode;

    #[test]
    fn test_fill_maps_state_to_semantic_colors() {
        let colors = jandi_theme(ThemeMode::Dark);
        assert_eq!(DotState::On.fill(&colors), palette::OK);
        assert_eq!(DotState::Off.fill(&colors), palette::dark::TEXT_3);
        assert_eq!(DotState::Busy.fill(&colors), palette::WARN);
        assert_eq!(DotState::Err.fill(&colors), palette::ERR);
    }

    #[test]
    fn test_halo_present_except_off() {
        let ext = JandiExt::dark();
        assert_eq!(DotState::On.halo(&ext), Some(palette::OK_DIM));
        assert_eq!(DotState::Off.halo(&ext), None);
        assert_eq!(DotState::Busy.halo(&ext), Some(palette::WARN_DIM));
        assert_eq!(DotState::Err.halo(&ext), Some(palette::ERR_DIM));
    }

    #[test]
    fn test_only_busy_pulses() {
        assert!(DotState::Busy.pulses());
        assert!(!DotState::On.pulses());
        assert!(!DotState::Off.pulses());
        assert!(!DotState::Err.pulses());
    }

    #[test]
    fn test_pulse_opacity_triangle() {
        assert!((pulse_opacity(0.0) - 1.0).abs() < 1e-6);
        assert!((pulse_opacity(0.5) - 0.35).abs() < 1e-6);
        assert!((pulse_opacity(1.0) - 1.0).abs() < 1e-6);
        assert!((pulse_opacity(0.25) - 0.675).abs() < 1e-6);
    }

    #[test]
    fn test_dot_geometry_constants() {
        assert_eq!(DOT_SIZE, 9.0);
        assert_eq!(HALO_WIDTH, 4.0);
        assert_eq!(PULSE_DURATION, Duration::from_secs(1));
    }
}
