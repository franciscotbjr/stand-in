//! # stand-in-mcp-explorer-ds — Design System for the MCP Explorer
//!
//! A themed, token-driven component library for the native desktop MCP
//! Explorer, built on **GPUI** and **`gpui-component`**. This crate is the
//! **canonical implementation** of the Design System described in
//! `stand-in-client-prototipo/` — 23 components from the prototype's catalog
//! plus one formal extension (Select, M9). The crate delivers the **visual
//! layer only**; the Explorer app consuming it is rebuilt separately (026).
//!
//! ## Module taxonomy (mirrors the canon)
//!
//! | Folder | Canon source | Contents |
//! |--------|-------------|----------|
//! | `theme/`   | `tokens/*.css` | Palette (jandi ramp → `ThemeColor`), density, typography, theme application |
//! | `core/`    | `components/core/` | Icon (22 glyphs), Button, IconButton, Badge, CopyButton, ToggleLink, StatusDot, Spinner |
//! | `forms/`   | `components/forms/` | Field, KeyValueRow, SegmentedControl, Select (extension 025) |
//! | `navigation/` | `components/navigation/` | SectionLabel, CapChip, Topbar, Tabbar, SidebarShell |
//! | `data/`    | `components/data/` | Panel, ListItem, ListSearch, PresetCard, LogRow, EmptyState, HintBar, JsonView |
//!
//! **24 components total** — every component traces 1:1 to a canon entry
//! (`components/<cat>/<Name>.{jsx,prompt.md}`). No component is orphaned; all
//! have a gallery section.
//!
//! ## Usage (from the 026 Explorer app)
//!
//! ```ignore
//! // Bootstrap at app startup:
//! stand_in_mcp_explorer_ds::init(&mut cx);
//! stand_in_mcp_explorer_ds::theme::apply_theme(
//!     stand_in_mcp_explorer_ds::theme::ThemeMode::Dark,
//!     cx
//! );
//!
//! // Compose components with the prelude:
//! use stand_in_mcp_explorer_ds::prelude::*;
//! use stand_in_mcp_explorer_ds::core::button::Button;
//! use stand_in_mcp_explorer_ds::navigation::topbar::Topbar;
//! ```
//!
//! ## The eight prohibitions
//!
//! Component code adheres to the Design System's binding rules:
//!
//! 1. **Never invent colours** — tokens via `cx.theme()` + `JandiExt` only.
//! 2. **No font but Hanken/JetBrains** — embedded via `DsAssets`.
//! 3. **Only the 22 Icon catalog** — no icon library, no emoji.
//! 4. **No shadow on in-flow cards** — separation via 1px border + surface step.
//! 5. **No decorative gradients** — only brand-mark and empty-glyph.
//! 6. **Mono/sans routing** — identifiers/values mono, prose sans.
//! 7. **Density-driven values** — `pad`, `row-h`, `gap`, `fs`, `radius`.
//! 8. **Reuse before create** — every component traces to a canon entry.
//!
//! ## Gallery & Storybook
//!
//! The companion crate `stand-in-mcp-explorer-gallery` is a native Storybook
//! that renders all 10 sections (Foundations, Icon, Indicators, Actions,
//! Badges, Forms, Forms Advanced, Select, Navigation, Data) in dark/light
//! mode with density switching:
//!
//! ```bash
//! cargo run -p stand-in-mcp-explorer-gallery
//! ```
//!
//! ## Pinned revisions
//!
//! `gpui` 0.2.2 @ `3f5705b9` / `gpui-component` 0.5.2 @ `70d2c44b`. Code must
//! target these exact revisions — never a web example or a later `main`.

pub mod assets;
pub mod core;
pub mod data;
pub mod forms;
pub mod navigation;
pub mod prelude;
pub mod theme;

use gpui::App;

/// Bootstrap `gpui-component` so its themed widgets and overlay infrastructure
/// (`Root`, modals, tooltips) work. Called once at app startup before building
/// any component.
///
/// Does **not** install the jandi theme — call `theme::apply_theme(mode, cx)`
/// separately so the caller chooses the initial mode and density.
pub fn init(cx: &mut App) {
    gpui_component::init(cx);
    // Default density — caller may override via apply_theme_and_density.
    cx.set_global(theme::density::GlobalDensity::new(
        theme::density::Density::Regular,
    ));
}
