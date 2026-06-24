//! Field — labeled input/textarea with optional hint and required marker.
//!
//! 1:1 with `forms/Field.jsx` + `.field`/`.input`/`.textarea` rules in
//! `forms/forms.css`. The canonical typography rule: technical content
//! (commands, args, URLs, paths, keys) is **mono by default**; set
//! `.mono(false)` only for human prose.
//!
//! Anatomy: column (gap 6), optional label (fs-sm 12, weight 600, text-2),
//! required `*` in err colour, control (gpui-component `Input` with canon
//! styles — bg depends on mode), hint (11.5 px, text-3, line-height 1.4).
//! Long mode produces a textarea (min-height 78, line-height 1.5). The
//! caller owns the `Entity<InputState>` — the Field is a stateless wrapper.
//!
//! ```ignore
//! use stand_in_mcp_explorer_ds::forms::Field;
//!
//! Field::new(&my_input_state)
//!     .label("Command")
//!     .required()
//!     .hint("Separated by spaces; use quotes for composite values")
//!     .id("cmd-field");
//! ```

use gpui::{
    App, ClickEvent, ElementId, Entity, FontWeight, InteractiveElement, IntoElement, ParentElement,
    Pixels, RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div, px,
    relative,
};
use gpui_component::input::{Input, InputState};
use gpui_component::{ActiveTheme as _, ThemeMode, h_flex, v_flex};

use crate::core::icon::{Icon, IconName, IconSize};
use crate::theme::colors::JandiExt;
use crate::theme::density::RADIUS_INPUT;
use crate::theme::typography;

// ---------------------------------------------------------------------------
// MaskedToggle — tiny entity tracking the revealed/masked state for a
// secret field, since InputState.masked is a private field with no getter.
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct MaskedToggle {
    visible: bool,
}

impl MaskedToggle {
    fn new(masked: bool) -> Self {
        Self { visible: !masked }
    }
}

// ---------------------------------------------------------------------------
// Field
// ---------------------------------------------------------------------------

/// A labeled form input wrapping gpui-component's `Input`.
///
/// The Field is stateless — the caller owns the `Entity<InputState>` and is
/// responsible for subscribing to `InputEvent` and creating the state with
/// the right mode (single-line / multi-line / auto-grow).
#[derive(IntoElement)]
pub struct Field {
    state: Entity<InputState>,
    label: Option<SharedString>,
    required: bool,
    hint: Option<SharedString>,
    invalid: bool,
    mono: bool,
    long: bool,
    secret: bool,
    width: Option<Pixels>,
    id: ElementId,
}

impl Field {
    /// Create a field wrapping the given input state.
    pub fn new(state: &Entity<InputState>) -> Self {
        Self {
            state: state.clone(),
            label: None,
            required: false,
            hint: None,
            invalid: false,
            mono: true,
            long: false,
            secret: false,
            width: None,
            id: ElementId::from("field"),
        }
    }

    /// Set a label displayed above the input (fs-sm, weight 600, text-2).
    pub fn label(mut self, text: impl Into<SharedString>) -> Self {
        self.label = Some(text.into());
        self
    }

    /// Mark as required — appends a red `*` after the label.
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Set a hint displayed below the input (11.5 px, text-3).
    pub fn hint(mut self, text: impl Into<SharedString>) -> Self {
        self.hint = Some(text.into());
        self
    }

    /// Set the invalid state — border turns `err` (red).
    pub fn invalid(mut self) -> Self {
        self.invalid = true;
        self
    }

    /// Toggle mono font (default `true` — technical content).
    /// Set `false` for human prose.
    pub fn mono(mut self, yes: bool) -> Self {
        self.mono = yes;
        self
    }

    /// Textarea variant (min-height 78, line-height 1.5).
    /// The caller must also configure `InputState` with multi-line / auto-grow
    /// at creation time.
    pub fn long(mut self) -> Self {
        self.long = true;
        self
    }

    /// Secret field: the input is masked (dots) and an Eye toggle button
    /// is appended via `.suffix(...)`. The caller MUST create the
    /// `InputState` with `.masked(true)`. Mutually exclusive with `.long()`
    /// — `secret` ignores `long` (masking requires single-line).
    pub fn secret(mut self) -> Self {
        self.secret = true;
        self
    }

    /// Fix the field column to an explicit width. Without it the column is
    /// `w_full` (fills its parent) — which, as a flex-column item, can collapse
    /// to content width when no sibling anchors the parent's width (e.g. a sole
    /// secret field, whose `.suffix` editor is content-sized). A definite width
    /// gives a stable control, the same remedy `Select::width` uses (028 #16).
    pub fn width(mut self, w: impl Into<Pixels>) -> Self {
        self.width = Some(w.into());
        self
    }

    /// Set the element id.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }
}

impl RenderOnce for Field {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // Gather theme data early (clone what we need) so we can release
        // the immutable borrow of `cx` before `use_keyed_state` needs `&mut cx`.
        let t = cx.theme().clone();
        let mode = t.mode;
        let colors = t.colors; // ThemeColor is Copy
        let ext = cx.global::<JandiExt>().clone();
        let mono_font = t.mono_font_family.clone();
        let sans_font = t.font_family.clone();
        drop(t); // release any possible handle

        // Input background per mode: dark = bg (background), light = surface-2 (secondary).
        let input_bg = match mode {
            ThemeMode::Light => colors.secondary,
            _ => colors.background,
        };

        let font_family = if self.mono { mono_font } else { sans_font };

        let border_color = if self.invalid {
            colors.danger_foreground
        } else {
            ext.border_2
        };

        // --- If secret, grab the MaskedToggle entity BEFORE building the column,
        //     because it needs `&mut cx` and we can't have `cx` immutably borrowed
        //     from the column construction below.
        let toggle: Option<Entity<MaskedToggle>> = if self.secret {
            let toggle_key = format!("field-eye-{}", self.id);
            Some(window.use_keyed_state(
                ElementId::Name(gpui::SharedString::from(toggle_key)),
                cx,
                |_, _| MaskedToggle::new(true),
            ))
        } else {
            None
        };

        // w_full so the field fills its container (the input is already w_full),
        // unless an explicit width is set for a stable control (the input then
        // fills that definite width). A labeled field spans its column (028 #19).
        let mut col = v_flex().id(self.id.clone()).gap(px(6.));
        col = match self.width {
            Some(w) => col.w(w),
            None => col.w_full(),
        };

        // Label
        if let Some(lbl) = &self.label {
            let mut label_row = h_flex().gap(px(6.)).items_center().child(
                div()
                    .text_size(px(typography::FS_SM))
                    .text_color(colors.secondary_foreground)
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(lbl.clone()),
            );
            if self.required {
                label_row = label_row.child(
                    div()
                        .text_size(px(typography::FS_SM))
                        .text_color(colors.danger_foreground)
                        .font_weight(FontWeight::BOLD)
                        .child("*"),
                );
            }
            col = col.child(label_row);
        }

        // Input control — use appearance(false) so we control every visual
        // property; focus ring comes from gpui-component's internal handling
        // (theme ring → oby at 22 %).
        let mut input_el = Input::new(&self.state)
            .appearance(false)
            .w_full()
            .bg(input_bg)
            .border_1()
            .border_color(border_color)
            .text_color(colors.foreground)
            .rounded(px(RADIUS_INPUT))
            .px(px(11.))
            .py(px(9.))
            .text_size(px(typography::FS_MD))
            .font_family(font_family.clone());

        if self.long && !self.secret {
            input_el = input_el.min_h(px(78.)).line_height(relative(1.5));
        }

        // Secret: append an Eye toggle in the suffix slot.
        // The Eye icon changes colour: muted_foreground when masked,
        // foreground when revealed. No EyeOff glyph (closed catalog, prohibition 3).
        // We track the revealed/masked state in a separate tiny entity because
        // InputState.masked is a private field with no getter.
        if let Some(toggle_ent) = toggle {
            let visible = toggle_ent.read(cx).visible;
            let state_clone = self.state.clone();
            let toggle_clone = toggle_ent.clone();

            // Clone colors we need before moving into closure
            let masked_color = colors.muted_foreground;
            let revealed_color = colors.foreground;

            let eye = div()
                .id("field-eye-toggle")
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .on_click(move |_ev: &ClickEvent, window, app| {
                    let next_visible = !toggle_clone.read(app).visible;
                    toggle_clone.update(app, |s, cx| {
                        s.visible = next_visible;
                        cx.notify();
                    });
                    state_clone.update(app, |s, cx| {
                        s.set_masked(!next_visible, window, cx);
                    });
                })
                .child(
                    Icon::new(IconName::Eye)
                        .size(IconSize::Sm)
                        .color(if visible {
                            revealed_color
                        } else {
                            masked_color
                        }),
                );
            input_el = input_el.suffix(eye);
        }

        col = col.child(input_el);

        // Hint
        if let Some(hint_text) = &self.hint {
            col = col.child(
                div()
                    .text_size(px(11.5))
                    .text_color(colors.muted_foreground)
                    .line_height(relative(1.4))
                    .child(hint_text.clone()),
            );
        }

        col
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    // Full render + input interaction tested via the gallery smoke
    // (`smoke-open.ps1 forms overview dark/light`) and the live typing echo
    // (gate funcional). InputState entities require a Window — unit-level
    // property tests belong here once a TestAppContext harness lands (GPUI C2).
}
