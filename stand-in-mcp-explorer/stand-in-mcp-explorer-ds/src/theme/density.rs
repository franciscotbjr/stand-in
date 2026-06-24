//! Density system — three levels (compact / regular / comfy) governing
//! padding, row height, gap, font size, and panel radius.
//!
//! Fixed per-role radii do **not** scale with density.
//! Values transcribed from `stand-in-client-prototipo/tokens/spacing.css`.

/// Density level. Determines 5 variables: pad, row-h, gap, fs, radius.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Density {
    Compact,
    #[default]
    Regular,
    Comfy,
}

impl Density {
    pub fn pad(&self) -> f32 {
        match self {
            Density::Compact => 10.0,
            Density::Regular => 14.0,
            Density::Comfy => 18.0,
        }
    }

    pub fn row_h(&self) -> f32 {
        match self {
            Density::Compact => 32.0,
            Density::Regular => 38.0,
            Density::Comfy => 46.0,
        }
    }

    pub fn gap(&self) -> f32 {
        match self {
            Density::Compact => 8.0,
            Density::Regular => 12.0,
            Density::Comfy => 16.0,
        }
    }

    pub fn fs(&self) -> f32 {
        match self {
            Density::Compact => 13.0,
            Density::Regular => 14.0,
            Density::Comfy => 15.0,
        }
    }

    pub fn radius(&self) -> f32 {
        match self {
            Density::Compact => 8.0,
            Density::Regular => 10.0,
            Density::Comfy => 12.0,
        }
    }

    /// Sidebar width shrinks at compact density (304 → 280).
    pub fn sidebar_w(&self) -> f32 {
        match self {
            Density::Compact => 280.0,
            Density::Regular => 304.0,
            Density::Comfy => 304.0,
        }
    }
}

/// Fixed per-role radii (px). Do **not** scale with density.
pub const RADIUS_INPUT: f32 = 8.0;
pub const RADIUS_CHIP: f32 = 7.0;
pub const RADIUS_BADGE: f32 = 6.0;
pub const RADIUS_BTN: f32 = 9.0;
pub const RADIUS_CARD: f32 = 10.0;
pub const RADIUS_PILL: f32 = 99.0;

/// Focus ring: 3px, oby at 22% alpha.
pub use super::palette::OBY as FOCUS_RING_COLOR;
pub const FOCUS_RING_WIDTH: f32 = 3.0;

// ---------------------------------------------------------------------------
// GlobalDensity — a gpui Global so components read density via
// `cx.global::<GlobalDensity>()`.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlobalDensity(pub Density);

impl gpui::Global for GlobalDensity {}

impl GlobalDensity {
    pub fn new(d: Density) -> Self {
        Self(d)
    }
}

impl std::ops::Deref for GlobalDensity {
    type Target = Density;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_density_values_canonical() {
        assert_eq!(Density::Compact.pad(), 10.0);
        assert_eq!(Density::Regular.pad(), 14.0);
        assert_eq!(Density::Comfy.pad(), 18.0);

        assert_eq!(Density::Compact.row_h(), 32.0);
        assert_eq!(Density::Regular.row_h(), 38.0);
        assert_eq!(Density::Comfy.row_h(), 46.0);

        assert_eq!(Density::Compact.gap(), 8.0);
        assert_eq!(Density::Regular.gap(), 12.0);
        assert_eq!(Density::Comfy.gap(), 16.0);

        assert_eq!(Density::Compact.fs(), 13.0);
        assert_eq!(Density::Regular.fs(), 14.0);
        assert_eq!(Density::Comfy.fs(), 15.0);

        assert_eq!(Density::Compact.radius(), 8.0);
        assert_eq!(Density::Regular.radius(), 10.0);
        assert_eq!(Density::Comfy.radius(), 12.0);
    }

    #[test]
    fn test_fixed_radii() {
        assert_eq!(RADIUS_BADGE, 6.0);
        assert_eq!(RADIUS_CHIP, 7.0);
        assert_eq!(RADIUS_INPUT, 8.0);
        assert_eq!(RADIUS_BTN, 9.0);
        assert_eq!(RADIUS_CARD, 10.0);
        assert_eq!(RADIUS_PILL, 99.0);
    }

    #[test]
    fn test_default_is_regular() {
        assert_eq!(Density::default(), Density::Regular);
    }

    #[test]
    fn test_sidebar_w_compact() {
        assert_eq!(Density::Compact.sidebar_w(), 280.0);
        assert_eq!(Density::Regular.sidebar_w(), 304.0);
    }
}
