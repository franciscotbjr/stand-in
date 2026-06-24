//! KeyValueRow — removable key-value input pair (grid "k x" / "v x", 1fr auto).
//!
//! 1:1 with `forms/KeyValueRow.jsx` + `.kv-row` rules in `forms/forms.css`.
//! For editable pair lists like environment variables. New rows are added via
//! a quiet `ToggleLink` "+ add" below the list, never a large button.
//!
//! Anatomy (revised 025, design-owner decision: stacked wins for readable
//! field width — was side-by-side 1fr 1fr auto): `w_full` h_flex (gap 6,
//! items center) holding a `flex_1 min_w(0)` column that stacks the key input
//! ABOVE the value input (gap 6, both `w_full`; key placeholder "CHAVE"
//! uppercase, value placeholder "valor" lowercase, both 12px mono, padding
//! 7×9), plus the IconButton X `flex_none` centred at the right of both rows.
//! The row fills its container — the fields never collapse to min-content.
//!
//! The caller owns both `Entity<InputState>` — the component is stateless.

use gpui::{
    App, ElementId, Entity, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled,
    Window, px,
};
use gpui_component::input::{Input, InputState};
use gpui_component::{ActiveTheme as _, ThemeMode, h_flex, v_flex};

use crate::core::button::ClickHandler;
use crate::core::icon::IconName;
use crate::core::icon_button::IconButton;
use crate::theme::colors::JandiExt;
use crate::theme::density::RADIUS_INPUT;
use crate::theme::typography;

// ---------------------------------------------------------------------------
// KeyValueRow
// ---------------------------------------------------------------------------

/// A key-value pair row with key input, value input, and a remove button.
///
/// ```ignore
/// use stand_in_mcp_explorer_ds::forms::KeyValueRow;
///
/// KeyValueRow::new(&key_state, &value_state)
///     .on_remove(cx.listener(move |this, _ev, _window, cx| { ... }))
///     .id(("kv-row", 0));
/// ```
#[derive(IntoElement)]
pub struct KeyValueRow {
    key_state: Entity<InputState>,
    value_state: Entity<InputState>,
    id: ElementId,
    on_remove: Option<ClickHandler>,
}

impl KeyValueRow {
    /// Create a KeyValueRow wrapping the given key and value input states.
    /// Both states are owned by the caller.
    pub fn new(key_state: &Entity<InputState>, value_state: &Entity<InputState>) -> Self {
        Self {
            key_state: key_state.clone(),
            value_state: value_state.clone(),
            id: ElementId::from("kv-row"),
            on_remove: None,
        }
    }

    /// Set the element id (use a stable identifier per row, not a volatile
    /// vector index).
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    /// Attach a remove handler fired by the IconButton X.
    pub fn on_remove(mut self, handler: ClickHandler) -> Self {
        self.on_remove = Some(handler);
        self
    }
}

impl RenderOnce for KeyValueRow {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = cx.theme();
        let mode = t.mode;
        let colors = &t.colors;
        let ext = cx.global::<JandiExt>();
        let mono = t.mono_font_family.clone();

        // Input bg: dark = bg, light = surface-2 (same as Field).
        let input_bg = match mode {
            ThemeMode::Light => t.secondary,
            _ => t.background,
        };

        let border_color = ext.border_2;

        h_flex()
            .id(self.id)
            .w_full()
            .gap(px(6.))
            .items_center()
            // Stacked fields column (canon grid areas "k" over "v", 1fr):
            // key ABOVE value, both full width — never min-content collapse.
            .child(
                v_flex()
                    .flex_1()
                    .min_w(px(0.))
                    .gap(px(6.))
                    .child(
                        Input::new(&self.key_state)
                            .appearance(false)
                            .w_full()
                            .bg(input_bg)
                            .border_1()
                            .border_color(border_color)
                            .text_color(colors.foreground)
                            .rounded(px(RADIUS_INPUT))
                            .px(px(9.))
                            .py(px(7.))
                            .text_size(px(typography::FS_SM))
                            .font_family(mono.clone()),
                    )
                    .child(
                        Input::new(&self.value_state)
                            .appearance(false)
                            .w_full()
                            .bg(input_bg)
                            .border_1()
                            .border_color(border_color)
                            .text_color(colors.foreground)
                            .rounded(px(RADIUS_INPUT))
                            .px(px(9.))
                            .py(px(7.))
                            .text_size(px(typography::FS_SM))
                            .font_family(mono),
                    ),
            )
            // Remove button — flex_none, centred beside both rows (area "x").
            .child({
                let mut btn = IconButton::new(IconName::X, "Remover");
                if let Some(handler) = self.on_remove {
                    btn = btn.on_click_boxed(handler);
                }
                btn
            })
    }
}

// Tests: `Entity<InputState>` requires a Window, so rendering/property tests
// live in the gallery smoke (`smoke-open.ps1 forms-advanced dark/light`).
// Unit-level property tests belong here once a TestAppContext harness lands
// (GPUI C2). No empty test stubs — the M17 sweep removed the placeholder.
