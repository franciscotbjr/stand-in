//! Shared helpers for gallery sections (single home — no per-section copies).

use gpui::{FontWeight, IntoElement, ParentElement, SharedString, Styled, div};
use gpui_component::{h_flex, v_flex};
use stand_in_mcp_explorer_ds::prelude;
use stand_in_mcp_explorer_ds::theme::typography;

/// `f32` → `Pixels` shorthand used across section renderers.
pub fn px(v: f32) -> gpui::Pixels {
    prelude::px(v)
}

/// Inner body column for section scroll containers.
///
/// CSS/taffy rule: shrinkable children of a height-limited flex column are
/// **squashed to min-content before overflow kicks in** (the BUG-10/§5b class —
/// content "partially hidden by the view height"). Wrapping the section content
/// in a `flex_none` column keeps every child at its natural height so the
/// scrollbar covers the real total.
pub fn section_body() -> gpui::Div {
    v_flex().flex_none().w_full().pb(px(16.))
}

/// Mono section heading used by every gallery section.
pub fn section_label(
    label: &str,
    t: &gpui_component::Theme,
    mono: &SharedString,
) -> impl IntoElement + use<> {
    h_flex().px(px(typography::FS_LG)).py_2().child(
        div()
            .text_size(px(typography::FS_XS))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(t.foreground)
            .font_family(mono.clone())
            .child(SharedString::from(label)),
    )
}
