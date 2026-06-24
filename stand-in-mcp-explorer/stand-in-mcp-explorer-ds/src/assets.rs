//! Combined asset source for the MCP Explorer Design System.
//!
//! `DsAssets` serves the 22 icon SVGs from `assets/icons/`, plus component
//! anatomy SVGs that are NOT icon glyphs (the Spinner's rotating arc — the
//! canon draws the spinner with CSS borders, so the catalog stays at 22), and
//! delegates all other asset paths (fonts, etc.) to
//! `gpui_component_assets::Assets`. This single `AssetSource` is what the
//! gallery Storybook installs at app startup.

use crate::core::IconName;
use gpui::{AssetSource, Result, SharedString};
use std::borrow::Cow;

// ---------------------------------------------------------------------------
// DsAssets — combined source (icon SVGs + gpui-component fonts)
// ---------------------------------------------------------------------------

/// Serves the 22 closed-catalog icon SVGs embedded at compile time,
/// delegating all other paths to `gpui_component_assets::Assets`.
pub struct DsAssets;

impl AssetSource for DsAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        // Component-anatomy SVGs (not icon glyphs — the 22-glyph catalog is closed).
        if path == "spinner/arc.svg" {
            return Ok(Some(Cow::Borrowed(include_bytes!(
                "assets/spinner-arc.svg"
            ))));
        }
        if let Some(name) = path
            .strip_prefix("icons/")
            .and_then(|p| p.strip_suffix(".svg"))
            && let Some(bytes) = icon_bytes(name)
        {
            return Ok(Some(Cow::Borrowed(bytes)));
        }
        // Icon name not in our catalog — delegate to gpui_component_assets
        gpui_component_assets::Assets.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        if path == "icons" || path == "icons/" {
            let ours: Vec<SharedString> = IconName::ALL
                .iter()
                .map(|n| SharedString::from(format!("icons/{}.svg", n.as_str())))
                .collect();
            let mut delegate = gpui_component_assets::Assets
                .list("icons")
                .unwrap_or_default();
            for name in ours {
                if !delegate.contains(&name) {
                    delegate.push(name);
                }
            }
            return Ok(delegate);
        }
        gpui_component_assets::Assets.list(path)
    }
}

// ---------------------------------------------------------------------------
// Embedded SVG bytes — one include per glyph
// ---------------------------------------------------------------------------

fn icon_bytes(name: &str) -> Option<&'static [u8]> {
    let b: &[u8] = match name {
        "plug" => include_bytes!("assets/icons/plug.svg"),
        "tool" => include_bytes!("assets/icons/tool.svg"),
        "doc" => include_bytes!("assets/icons/doc.svg"),
        "chat" => include_bytes!("assets/icons/chat.svg"),
        "bell" => include_bytes!("assets/icons/bell.svg"),
        "history" => include_bytes!("assets/icons/history.svg"),
        "play" => include_bytes!("assets/icons/play.svg"),
        "plus" => include_bytes!("assets/icons/plus.svg"),
        "x" => include_bytes!("assets/icons/x.svg"),
        "search" => include_bytes!("assets/icons/search.svg"),
        "copy" => include_bytes!("assets/icons/copy.svg"),
        "check" => include_bytes!("assets/icons/check.svg"),
        "lock" => include_bytes!("assets/icons/lock.svg"),
        "bolt" => include_bytes!("assets/icons/bolt.svg"),
        "refresh" => include_bytes!("assets/icons/refresh.svg"),
        "chevron" => include_bytes!("assets/icons/chevron.svg"),
        "sub" => include_bytes!("assets/icons/sub.svg"),
        "info" => include_bytes!("assets/icons/info.svg"),
        "leaf" => include_bytes!("assets/icons/leaf.svg"),
        "eye" => include_bytes!("assets/icons/eye.svg"),
        "file" => include_bytes!("assets/icons/file.svg"),
        "globe" => include_bytes!("assets/icons/globe.svg"),
        _ => return None,
    };
    Some(b)
}

// ---------------------------------------------------------------------------
// Tests — prove that the 22 assets resolve (not a façade)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_22_icons_load() {
        let assets = DsAssets;
        for name in IconName::ALL {
            let path = format!("icons/{}.svg", name.as_str());
            let data = assets
                .load(&path)
                .expect("load returned Err")
                .unwrap_or_else(|| panic!("asset {} not found", path));
            assert!(!data.is_empty(), "asset {} is empty", path);
        }
    }

    #[test]
    fn test_play_contains_play_path() {
        let assets = DsAssets;
        let data = assets
            .load("icons/play.svg")
            .expect("load err")
            .expect("not found");
        let svg = std::str::from_utf8(&data).expect("invalid utf8");
        assert!(
            svg.contains("<path d=\"M7 5l12 7-12 7V5Z\"/>"),
            "play SVG missing expected path: {svg}",
        );
    }

    #[test]
    fn test_search_two_elements() {
        let assets = DsAssets;
        let data = assets
            .load("icons/search.svg")
            .expect("load err")
            .expect("not found");
        let svg = std::str::from_utf8(&data).expect("invalid utf8");
        assert!(
            svg.contains("<circle cx=\"11\" cy=\"11\" r=\"7\"/>"),
            "search SVG missing circle"
        );
        assert!(
            svg.contains("<path d=\"m21 21-4.3-4.3\"/>"),
            "search SVG missing path"
        );
    }

    #[test]
    fn test_sub_has_filled_circle() {
        let assets = DsAssets;
        let data = assets
            .load("icons/sub.svg")
            .expect("load err")
            .expect("not found");
        let svg = std::str::from_utf8(&data).expect("invalid utf8");
        assert!(
            svg.contains("fill=\"currentColor\""),
            "sub SVG missing fill=currentColor"
        );
        assert!(
            svg.contains("stroke=\"none\""),
            "sub SVG missing stroke=none"
        );
    }

    #[test]
    fn test_doc_and_file_same_bytes() {
        let assets = DsAssets;
        let doc = assets
            .load("icons/doc.svg")
            .expect("load err")
            .expect("not found");
        let file = assets
            .load("icons/file.svg")
            .expect("load err")
            .expect("not found");
        assert_eq!(doc, file, "doc and file must share the same geometry");
    }

    #[test]
    fn test_unknown_icon_not_in_catalog() {
        assert!(icon_bytes("nonexistent").is_none());
    }

    #[test]
    fn test_spinner_arc_asset_loads() {
        let assets = DsAssets;
        let data = assets
            .load("spinner/arc.svg")
            .expect("load err")
            .expect("not found");
        assert!(!data.is_empty(), "spinner arc asset is empty");
        let svg = std::str::from_utf8(&data).expect("invalid utf8");
        assert!(svg.contains("<path"), "spinner arc SVG has no path: {svg}");
    }
}
