//! Theme tokens, colour mapping, density, and typography.
//!
//! Public API:
//! - `apply_theme(mode, cx)` — installs the jandi theme (dark or light).
//! - `apply_theme_and_density(mode, density, cx)` — installs theme + density.
//! - `JandiExt` — extension fields (surface-3, dim states, code-bg, tokens,
//!   shadow) accessible via `cx.global::<JandiExt>()`.
//! - `GlobalDensity` — current density, accessible via
//!   `cx.global::<GlobalDensity>()`.

pub mod colors;
pub mod density;
pub mod palette;
pub mod typography;

use gpui::App;
use gpui_component::{Theme, ThemeMode};

use colors::{JandiExt, jandi_theme};
use density::{Density, GlobalDensity};

/// Install the jandi theme (colours + fonts + radii) for the given mode.
///
/// Must be called **after** `gpui_component::init(cx)`.
pub fn apply_theme(mode: ThemeMode, cx: &mut App) {
    let theme = Theme::global_mut(cx);
    theme.mode = mode;
    theme.colors = jandi_theme(mode);
    theme.font_family = typography::sans_family();
    theme.mono_font_family = typography::mono_family();
    theme.radius = gpui::px(Density::default().radius());
    theme.radius_lg = gpui::px(density::RADIUS_CARD);

    cx.set_global(JandiExt::from_mode(mode));
}

/// Install theme and set the active density.
pub fn apply_theme_and_density(mode: ThemeMode, density: Density, cx: &mut App) {
    apply_theme(mode, cx);
    cx.set_global(GlobalDensity::new(density));
}

/// Primary colour shortcut (the `--jandi` ramp step, same for both modes).
pub fn primary_color() -> gpui::Hsla {
    palette::JANDI
}

impl JandiExt {
    fn from_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Dark => JandiExt::dark(),
            ThemeMode::Light => JandiExt::light(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primary_same_both_modes() {
        let d = jandi_theme(ThemeMode::Dark);
        let l = jandi_theme(ThemeMode::Light);
        assert_eq!(d.primary, l.primary);
        assert_eq!(d.primary, palette::JANDI);
    }
}
