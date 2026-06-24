//! Icon — the closed catalog of 22 stroke glyphs, 1:1 with `core/Icon.jsx`.
//!
//! Every glyph is stroke 2, 24×24 viewBox, round caps/joins. Color comes from the
//! theme via the element's text color (the SVG uses `currentColor`). Canonical
//! sizes: 12 (badges/chips), 14 (panel heads/IconButton), 15 (tabs/buttons, default),
//! 28 (EmptyState).

use gpui::{
    App, Hsla, IntoElement, Pixels, RenderOnce, SharedString, Styled, Transformation, Window, px,
    svg,
};
use std::fmt;

// ---------------------------------------------------------------------------
// IconName — closed catalog of 22 glyphs (DS-icons / prohibition 3 by construction)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IconName {
    Plug,
    Tool,
    Doc,
    Chat,
    Bell,
    History,
    Play,
    Plus,
    X,
    Search,
    Copy,
    Check,
    Lock,
    Bolt,
    Refresh,
    Chevron,
    Sub,
    Info,
    Leaf,
    Eye,
    File,
    Globe,
}

impl IconName {
    /// The full catalog in canonical order (matches `ICON_NAMES` in `Icon.jsx`).
    pub const ALL: [IconName; 22] = [
        IconName::Plug,
        IconName::Tool,
        IconName::Doc,
        IconName::Chat,
        IconName::Bell,
        IconName::History,
        IconName::Play,
        IconName::Plus,
        IconName::X,
        IconName::Search,
        IconName::Copy,
        IconName::Check,
        IconName::Lock,
        IconName::Bolt,
        IconName::Refresh,
        IconName::Chevron,
        IconName::Sub,
        IconName::Info,
        IconName::Leaf,
        IconName::Eye,
        IconName::File,
        IconName::Globe,
    ];

    /// Human-readable name matching the canonical key in `Icon.jsx`.
    pub const fn as_str(self) -> &'static str {
        match self {
            IconName::Plug => "plug",
            IconName::Tool => "tool",
            IconName::Doc => "doc",
            IconName::Chat => "chat",
            IconName::Bell => "bell",
            IconName::History => "history",
            IconName::Play => "play",
            IconName::Plus => "plus",
            IconName::X => "x",
            IconName::Search => "search",
            IconName::Copy => "copy",
            IconName::Check => "check",
            IconName::Lock => "lock",
            IconName::Bolt => "bolt",
            IconName::Refresh => "refresh",
            IconName::Chevron => "chevron",
            IconName::Sub => "sub",
            IconName::Info => "info",
            IconName::Leaf => "leaf",
            IconName::Eye => "eye",
            IconName::File => "file",
            IconName::Globe => "globe",
        }
    }
}

impl From<IconName> for SharedString {
    fn from(n: IconName) -> Self {
        n.as_str().into()
    }
}

// ---------------------------------------------------------------------------
// IconSize — canonical sizes
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IconSize {
    /// 12 px — badges, chips.
    Xs,
    /// 14 px — panel heads, IconButton.
    Sm,
    /// 15 px — tabs, buttons (default).
    #[default]
    Md,
    /// 28 px — EmptyState glyph.
    Lg,
}

impl IconSize {
    pub const fn pixels(self) -> f32 {
        match self {
            IconSize::Xs => 12.0,
            IconSize::Sm => 14.0,
            IconSize::Md => 15.0,
            IconSize::Lg => 28.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Icon — the renderable element
// ---------------------------------------------------------------------------

/// A renderable icon from the closed 22-glyph catalog.
///
/// Icons are served from the DS asset source via `include_bytes!`‑embedded
/// SVG files at `icons/<name>.svg`. The `svg().path()` call uses the asset
/// **key** (not inline markup), resolved at render time through `DsAssets`.
///
/// **Colour resolution (the pinned gpui-component idiom):** GPUI's `svg()`
/// only paints when the svg element's OWN `style.text.color` is set
/// (`gpui/src/elements/svg.rs` — `path.zip(style.text.color)`); the parent's
/// `text_color` does **not** cascade into the svg element. So the Icon
/// resolves the colour at render time: an explicit `.color()` override, else
/// `window.text_style().color` — the contextual text colour pushed by the
/// ancestor `div().text_color(…)` (exactly what the pinned gpui-component
/// `Icon` does, `crates/ui/src/icon.rs:144`). Without this resolution every
/// icon renders invisible (caught by the human visual gate, 025 post-DONE).
#[derive(IntoElement)]
pub struct Icon {
    name: IconName,
    size_px: Pixels,
    color: Option<Hsla>,
    rotation: Option<Transformation>,
}

impl Icon {
    /// Create an icon for the given glyph name at the default size (15 px).
    /// Colour is inherited from the parent element's text colour.
    pub fn new(name: IconName) -> Self {
        Self {
            name,
            size_px: px(IconSize::default().pixels()),
            color: None,
            rotation: None,
        }
    }

    /// Set a canonical size.
    pub fn size(mut self, size: IconSize) -> Self {
        self.size_px = px(size.pixels());
        self
    }

    /// Set a custom pixel size (freeform; canonical sizes prefer `IconSize`).
    pub fn with_px(mut self, px_val: impl Into<Pixels>) -> Self {
        self.size_px = px_val.into();
        self
    }

    /// Explicitly set the colour. Prefer inheriting via parent `text_color()`
    /// for theme-consistency; use this only when the parent cannot set colour
    /// (e.g. a call-site that needs a one-off colour override).
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    /// Apply a transformation (e.g. rotation) to the icon.
    /// Use `Transformation::rotate(gpui::percentage(0.25))` for the canonical
    /// 90-degree chevron-down rotation (idiom from Spinner M4).
    pub fn rotate(mut self, t: Transformation) -> Self {
        self.rotation = Some(t);
        self
    }

    /// The glyph name.
    pub fn name(&self) -> IconName {
        self.name
    }

    /// The asset key used to resolve the SVG bytes at render time.
    fn asset_key(&self) -> SharedString {
        SharedString::from(format!("icons/{}.svg", self.name.as_str()))
    }
}

impl fmt::Debug for Icon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Icon")
            .field("name", &self.name)
            .field("size_px", &self.size_px)
            .finish()
    }
}

impl RenderOnce for Icon {
    fn render(self, window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // The svg element does NOT inherit the parent's text colour — resolve
        // it explicitly (explicit override, else the current text style), the
        // pinned gpui-component Icon idiom. See the struct docs.
        let color = self.color.unwrap_or_else(|| window.text_style().color);
        let mut el = svg()
            .flex_none()
            .size(self.size_px)
            .text_color(color)
            .path(self.asset_key());
        if let Some(t) = self.rotation {
            el = el.with_transformation(t);
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
    fn test_icon_catalog_has_22_variants() {
        assert_eq!(IconName::ALL.len(), 22);
    }

    #[test]
    fn test_icon_names_match_canon() {
        let expected = [
            "plug", "tool", "doc", "chat", "bell", "history", "play", "plus", "x", "search",
            "copy", "check", "lock", "bolt", "refresh", "chevron", "sub", "info", "leaf", "eye",
            "file", "globe",
        ];
        let actual: Vec<&str> = IconName::ALL.iter().map(|n| n.as_str()).collect();
        assert_eq!(actual.as_slice(), expected.as_slice());
    }

    #[test]
    fn test_asset_key_format() {
        for name in IconName::ALL {
            let icon = Icon::new(name);
            let key = icon.asset_key();
            assert_eq!(
                key.as_ref(),
                format!("icons/{}.svg", name.as_str()),
                "wrong asset key for {:?}",
                name,
            );
        }
    }

    #[test]
    fn test_canonical_sizes() {
        assert_eq!(IconSize::Xs.pixels(), 12.0);
        assert_eq!(IconSize::Sm.pixels(), 14.0);
        assert_eq!(IconSize::Md.pixels(), 15.0);
        assert_eq!(IconSize::Lg.pixels(), 28.0);
    }

    #[test]
    fn test_icon_construction_default_size_is_md() {
        let icon = Icon::new(IconName::Play);
        assert_eq!(icon.size_px, px(15.));
    }

    #[test]
    fn test_icon_with_size() {
        let icon = Icon::new(IconName::Play).size(IconSize::Lg);
        assert_eq!(icon.size_px, px(28.));
    }

    #[test]
    fn test_icon_with_px() {
        let icon = Icon::new(IconName::Play).with_px(20.);
        assert_eq!(icon.size_px, px(20.));
    }

    #[test]
    fn test_icon_with_rotation() {
        use gpui::percentage;
        let icon = Icon::new(IconName::Chevron).rotate(Transformation::rotate(percentage(0.25)));
        assert!(icon.rotation.is_some());
    }
}
