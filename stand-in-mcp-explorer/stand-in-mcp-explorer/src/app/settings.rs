//! App-wide settings — theme, density, primary colour, guided mode.
//!
//! Persisted via `ProjectDirs` to `settings.json`, mirroring the `servers.rs`
//! pattern (M4). Round-trip tested with `tempfile` tempdirs.
//!
//! ## Design
//!
//! `AppSettings` holds serialisable choices (`ThemeChoice`, `DensityChoice`,
//! `PrimaryChoice`) independent of gpui types.  The `apply` function translates
//! them to gpui globals (`Theme`, `GlobalDensity`, `JandiExt`).
//!
//! Primary colour is constrained to the three swatches of the jandi palette
//! (`Jandi`, `Genipina`, `Oby`) — "tokens-only" (prohibition 1).

use std::path::Path;

use directories::ProjectDirs;
use gpui::App;
use gpui_component::{Theme, ThemeMode};
use serde::{Deserialize, Serialize};

use stand_in_mcp_explorer_ds::theme::apply_theme_and_density;
use stand_in_mcp_explorer_ds::theme::density::Density;
use stand_in_mcp_explorer_ds::theme::palette::{BRISA, GENIPINA, JANDI, OBY};

// ---------------------------------------------------------------------------
// Choice enums (serde-able, independent of gpui types)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ThemeChoice {
    Dark,
    Light,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DensityChoice {
    Compact,
    Regular,
    Comfy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PrimaryChoice {
    Jandi,
    Genipina,
    Oby,
}

// ---------------------------------------------------------------------------
// Conversions to gpui types
// ---------------------------------------------------------------------------

impl ThemeChoice {
    pub fn to_theme_mode(self) -> ThemeMode {
        match self {
            ThemeChoice::Dark => ThemeMode::Dark,
            ThemeChoice::Light => ThemeMode::Light,
        }
    }
}

impl DensityChoice {
    pub fn to_density(self) -> Density {
        match self {
            DensityChoice::Compact => Density::Compact,
            DensityChoice::Regular => Density::Regular,
            DensityChoice::Comfy => Density::Comfy,
        }
    }
}

impl PrimaryChoice {
    /// The jandi ramp swatch for this choice.
    pub fn to_hsla(self) -> gpui::Hsla {
        match self {
            PrimaryChoice::Jandi => JANDI,
            PrimaryChoice::Genipina => GENIPINA,
            PrimaryChoice::Oby => OBY,
        }
    }

    /// One ramp step lighter — used as hover/active colour.
    pub fn hover(self) -> gpui::Hsla {
        match self {
            PrimaryChoice::Jandi => OBY,
            PrimaryChoice::Genipina => JANDI,
            PrimaryChoice::Oby => BRISA,
        }
    }

    /// Near-white foreground for each swatch (ON_PRIMARY from the
    /// jandi theme — same for all three since they are medium-to-dark).
    pub fn foreground() -> gpui::Hsla {
        stand_in_mcp_explorer_ds::theme::palette::dark::ON_PRIMARY
    }
}

// ---------------------------------------------------------------------------
// AppSettings
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub theme: ThemeChoice,
    pub density: DensityChoice,
    pub primary: PrimaryChoice,
    /// Note: `guided` is re-exported but lives on `StudioApp`; this field
    /// persists the last state so the toggle survives restarts.  The app's
    /// `guided` flag is **not** owned by settings — it is synchronised
    /// on load and on each change.
    pub guided: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: ThemeChoice::Dark,
            density: DensityChoice::Regular,
            primary: PrimaryChoice::Jandi,
            guided: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Persistence (mirrors servers.rs)
// ---------------------------------------------------------------------------

fn project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("", "", "mcp-explorer")
}

pub fn config_dir() -> Option<std::path::PathBuf> {
    project_dirs().map(|p| p.config_dir().to_path_buf())
}

fn config_path() -> Option<std::path::PathBuf> {
    config_dir().map(|d| d.join("settings.json"))
}

pub fn load() -> AppSettings {
    match config_path() {
        Some(p) => load_from(&p),
        None => AppSettings::default(),
    }
}

pub fn save(settings: &AppSettings) -> std::io::Result<()> {
    let path = config_path()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no config directory"))?;
    save_to(settings, &path)
}

pub fn load_from(path: &Path) -> AppSettings {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return AppSettings::default(),
    };
    serde_json::from_str::<AppSettings>(&content).unwrap_or_default()
}

pub fn save_to(settings: &AppSettings, path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(settings)?;
    std::fs::write(path, json)
}

// ---------------------------------------------------------------------------
// Application (modifies gpui globals — needs &mut App)
// ---------------------------------------------------------------------------

/// Apply settings into gpui globals.  Must be called on the gpui thread with
/// `&mut App` (e.g. from `main.rs` or from a handler that has app access).
pub fn apply(settings: &AppSettings, cx: &mut App) {
    let mode = settings.theme.to_theme_mode();
    let density = settings.density.to_density();
    apply_theme_and_density(mode, density, cx);

    // Override primary colour if not the default Jandi.
    let primary = settings.primary.to_hsla();
    if primary != JANDI {
        let theme = Theme::global_mut(cx);
        theme.colors.primary = primary;
        theme.colors.primary_hover = settings.primary.hover();
        theme.colors.primary_active = theme.colors.primary_hover;
        theme.colors.button_primary = primary;
        theme.colors.button_primary_hover = theme.colors.primary_hover;
        theme.colors.button_primary_active = theme.colors.primary_hover;
    }
}

/// Apply theme, density, AND primary.  Call after ANY setting change so the
/// full state is consistent.
pub fn apply_full(settings: &AppSettings, cx: &mut App) {
    apply(settings, cx);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- PrimaryChoice conversions ---

    #[test]
    fn primary_choice_to_hsla() {
        assert_eq!(PrimaryChoice::Jandi.to_hsla(), JANDI);
        assert_eq!(PrimaryChoice::Genipina.to_hsla(), GENIPINA);
        assert_eq!(PrimaryChoice::Oby.to_hsla(), OBY);
    }

    #[test]
    fn primary_choice_hover() {
        assert_eq!(PrimaryChoice::Jandi.hover(), OBY);
        assert_eq!(PrimaryChoice::Genipina.hover(), JANDI);
        assert_eq!(PrimaryChoice::Oby.hover(), BRISA);
    }

    #[test]
    fn primary_choice_foreground_consistent() {
        let fg = PrimaryChoice::foreground();
        // ON_PRIMARY should be near-white (high luminance).
        assert!(fg.l > 0.8);
    }

    // --- Choice → gpui conversions ---

    #[test]
    fn theme_choice_to_theme_mode() {
        assert_eq!(ThemeChoice::Dark.to_theme_mode(), ThemeMode::Dark);
        assert_eq!(ThemeChoice::Light.to_theme_mode(), ThemeMode::Light);
    }

    #[test]
    fn density_choice_to_density() {
        assert_eq!(DensityChoice::Compact.to_density(), Density::Compact);
        assert_eq!(DensityChoice::Regular.to_density(), Density::Regular);
        assert_eq!(DensityChoice::Comfy.to_density(), Density::Comfy);
    }

    // --- Defaults ---

    #[test]
    fn default_settings() {
        let s = AppSettings::default();
        assert_eq!(s.theme, ThemeChoice::Dark);
        assert_eq!(s.density, DensityChoice::Regular);
        assert_eq!(s.primary, PrimaryChoice::Jandi);
        assert!(!s.guided);
    }

    // --- Persistence round-trip ---

    #[test]
    fn load_from_missing_path_returns_default() {
        let settings = load_from(Path::new("/nonexistent/path/settings.json"));
        assert_eq!(settings, AppSettings::default());
    }

    #[test]
    fn load_from_corrupt_file_returns_default() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("settings.json");
        std::fs::write(&path, b"not json {{{").expect("write");
        let settings = load_from(&path);
        assert_eq!(settings, AppSettings::default());
    }

    #[test]
    fn round_trip_save_and_load() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("settings.json");
        let original = AppSettings {
            theme: ThemeChoice::Light,
            density: DensityChoice::Comfy,
            primary: PrimaryChoice::Oby,
            guided: true,
        };
        save_to(&original, &path).expect("save");
        let loaded = load_from(&path);
        assert_eq!(loaded, original);
    }

    #[test]
    fn round_trip_default() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("settings.json");
        let original = AppSettings::default();
        save_to(&original, &path).expect("save");
        let loaded = load_from(&path);
        assert_eq!(loaded, original);
    }

    #[test]
    fn round_trip_genipina_compact() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("settings.json");
        let original = AppSettings {
            theme: ThemeChoice::Dark,
            density: DensityChoice::Compact,
            primary: PrimaryChoice::Genipina,
            guided: false,
        };
        save_to(&original, &path).expect("save");
        let loaded = load_from(&path);
        assert_eq!(loaded, original);
    }

    #[test]
    fn save_to_creates_parent_dir() {
        let dir = tempfile::tempdir().expect("tempdir");
        let nested = dir.path().join("sub").join("deep").join("settings.json");
        save_to(&AppSettings::default(), &nested).expect("save");
        assert!(nested.exists());
        let loaded = load_from(&nested);
        assert_eq!(loaded, AppSettings::default());
    }

    // --- serde camelCase round-trip ---

    #[test]
    fn serde_json_camel_case() {
        let json = serde_json::json!({
            "theme": "light",
            "density": "compact",
            "primary": "oby",
            "guided": true
        });
        let settings: AppSettings = serde_json::from_value(json).expect("deserialize");
        assert_eq!(settings.theme, ThemeChoice::Light);
        assert_eq!(settings.density, DensityChoice::Compact);
        assert_eq!(settings.primary, PrimaryChoice::Oby);
        assert!(settings.guided);
    }
}
