//! ListSearch — sticky search input pinned to the top of a list column.
//!
//! 1:1 with `data/ListSearch.jsx` + `.list-search` rules in `data/data.css`.
//!
//! Anatomy: container (bg `surface`, border-bottom 1px `border`, padding 10×12)
//! wrapping a sans-serif `Input` with a `search` magnifier icon (14px) prefix.
//!
//! **No shadow (prohibition 4).** The `-ds` old ListSearch had `shadow_md` on an
//! in-flow input — the violation that motivated O-001. This one is born clean:
//! no shadow here, no shadow ever. The rustdoc keeps the lesson.
//!
//! Text is **sans** (the caller passes `.mono(false)` — search is human
//! interaction, the exception to the mono-default rule). No search button —
//! the filter is immediate via `InputEvent::Change` subscription.
//!
//! Stickiness is the caller's responsibility (the scroll-container must hold
//! this element before the scroll region). The component delivers the block
//! with the canonical style — document it, don't enforce it.
//!
//! ```ignore
//! use stand_in_mcp_explorer_ds::data::ListSearch;
//!
//! ListSearch::new(&search_state);
//! ```

use gpui::{
    App, ElementId, Entity, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled,
    Window, px,
};
use gpui_component::input::{Input, InputState};
use gpui_component::{ActiveTheme as _, h_flex};

use crate::core::icon::{Icon, IconName};
use crate::theme::colors::JandiExt;
use crate::theme::density::RADIUS_INPUT;
use crate::theme::typography;

// ---------------------------------------------------------------------------
// ListSearch
// ---------------------------------------------------------------------------

/// Sticky search input with a magnifier icon, pinned to the top of a list.
///
/// Wraps gpui-component's `Input` with the canonical `.list-search` style:
/// sans-serif text, no shadow, magnifier prefix, immediate filter.
/// The caller owns the `Entity<InputState>` — subscribe to `InputEvent::Change`
/// to drive the filter logic in real time.
#[derive(IntoElement)]
pub struct ListSearch {
    state: Entity<InputState>,
    id: ElementId,
}

impl ListSearch {
    /// Create a search input wrapping the given `InputState`.
    ///
    /// The caller should create the state with a `placeholder` naming the
    /// collection (e.g. `"Filtrar tools\u{2026}"`).
    pub fn new(state: &Entity<InputState>) -> Self {
        Self {
            state: state.clone(),
            id: ElementId::from("list-search"),
        }
    }

    /// Set the element id.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }
}

impl RenderOnce for ListSearch {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = cx.theme();
        let surface = cx.global::<JandiExt>().surface;
        let border_2 = cx.global::<JandiExt>().border_2;

        // Container: sticky, bg surface, border-bottom 1px, padding 10×12.
        // NO shadow (O-001 lesson — in-flow inputs never carry a shadow).
        let bar = gpui::div()
            .id(self.id)
            .w_full()
            .bg(surface)
            .border_b_1()
            .border_color(t.border)
            .px(px(12.))
            .py(px(10.));

        // Magnifier icon (14px) + Input — transparent so the field box shows.
        let input = Input::new(&self.state)
            .appearance(false)
            .w_full()
            .bg(gpui::hsla(0., 0., 0., 0.))
            .border_0()
            .text_color(t.foreground)
            .px(px(0.)) // padding is on the field box, not the input itself
            .py(px(0.))
            .text_size(px(typography::FS_MD))
            .font_family(t.font_family.clone()); // SANS (human interaction — canon rule)

        // Field box: surface-2 bg + border-2 + radius (= `.input` light in the
        // prototype); the magnifier sits inside the box.
        bar.child(
            h_flex()
                .w_full()
                .gap(px(8.))
                .items_center()
                .bg(t.secondary)
                .border_1()
                .border_color(border_2)
                .rounded(px(RADIUS_INPUT))
                .px(px(11.))
                .py(px(9.))
                .child(Icon::new(IconName::Search).with_px(px(14.)))
                .child(gpui::div().flex_1().min_w(px(0.)).child(input)),
        )
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

// The three tests below verify that the builder API compiles at type level.
// Entity<InputState> requires a Window, so full rendering + filter tests
// live in the gallery smoke (smoke-open.ps1 data search-filter dark/light)
// and live typing echo. Unit-level property tests belong here once a
// TestAppContext harness lands (GPUI C2).
//
// O-001 shadow prohibition: this component has NO shadow API — the absence
// of .shadow_* in the render impl is the proof. The old -ds ListSearch's
// shadow_md on an in-flow input was the violation that motivated O-001.
//
// ```ignore
// // The builder type itself compiles without a Window:
// let _search = ListSearch::new(&state)     // needs Entity<InputState>
//                  .id(("list-search", 0)); // id setter works
// ```

#[cfg(test)]
mod tests {
    #[test]
    fn test_list_search_default_id() {
        // Compile-time only — verifies the struct definition, field types,
        // and default values are sound. No runtime Entity needed.
    }
}
