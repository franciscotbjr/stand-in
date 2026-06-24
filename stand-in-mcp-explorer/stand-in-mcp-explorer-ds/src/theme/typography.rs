//! Typography — font families, size scale, weights, and tracking.
//!
//! Two families only: Hanken Grotesk (sans) and JetBrains Mono (mono).
//! The mono/sans routing rule (prohibition 6): identifiers, paths, JSON,
//! timestamps, counters, and badges are ALWAYS mono. Prose is always sans.
//! Values from `stand-in-client-prototipo/tokens/typography.css`.

use gpui::{FontWeight, SharedString};

// ---------------------------------------------------------------------------
// Font families
// ---------------------------------------------------------------------------

pub const SANS: &str = "Hanken Grotesk";
pub const MONO: &str = "JetBrains Mono";

pub fn sans_family() -> SharedString {
    SANS.into()
}

pub fn mono_family() -> SharedString {
    MONO.into()
}

// ---------------------------------------------------------------------------
// Sizes (px) — the canonical scale; base (`fs`) shifts with density
// ---------------------------------------------------------------------------

pub const FS_2XS: f32 = 10.5; // badges
pub const FS_XS: f32 = 11.0; // metadata, section labels (uppercase)
pub const FS_SM: f32 = 12.0; // field labels, secondary descriptions
pub const FS_MD: f32 = 13.0; // inputs, buttons, list items
pub const FS_LG: f32 = 14.0; // body default (= base at regular density)
pub const FS_XL: f32 = 15.0; // brand name
pub const FS_TITLE: f32 = 20.0; // detail titles, empty states

// ---------------------------------------------------------------------------
// Weights
// ---------------------------------------------------------------------------

pub const W_REGULAR: FontWeight = FontWeight::NORMAL;
pub const W_MEDIUM: FontWeight = FontWeight::MEDIUM;
pub const W_SEMIBOLD: FontWeight = FontWeight::SEMIBOLD;
pub const W_BOLD: FontWeight = FontWeight::BOLD;

// ---------------------------------------------------------------------------
// Tracking (letter-spacing approximation via px offsets or skipped in GPUI)
// ---------------------------------------------------------------------------

pub const TRACK_TIGHT: f32 = -0.01; // titles
pub const TRACK_WIDE: f32 = 0.06; // panel-head uppercase
pub const TRACK_WIDER: f32 = 0.08; // section-label uppercase
