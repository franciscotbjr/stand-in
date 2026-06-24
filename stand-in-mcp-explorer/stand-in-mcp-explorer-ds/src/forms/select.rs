//! Select — compact single-choice dropdown (32px trigger + overlay option list).
//!
//! Extension 025 (D9/D10, canon entry `components/forms/Select.prompt.md`).
//! O-003: seletor de idioma. Popup overlay carries the DS's only legitimate shadow
//! (the elevation rule: shadow-only-on-overlays, 024).
//!
//! Investigation of gpui-component (pinned `70d2c44b`): the `Select` widget
//! (select.rs, 813 LoC) is a heavily-featured generic dropdown built on
//! `SearchableListState` / `SearchableListDelegate` / `List` — searchable,
//! keyboard-navigable, multi-section. Its rendering is deeply coupled to its
//! own design system: `input_style()` for bg/fg, `cx.theme().input` for border,
//! `shadow_md()` / `rounded(cx.theme().radius)` for the popup, its own
//! `Icon::new(IconName::ChevronDown)` icon, and `List`-based item delegates.
//! Achieving the exact jandi canon (surface-3 popup, border_2, RADIUS_CARD(10),
//! shadow_overlay token, our Icon catalog, fs-md items with our check icon)
//! through the pin would require fighting the theme across ~10 dimension hooks.
//! The `Popover` (popover.rs, 531 LoC) is similarly generic. **Verdict: build.**
//! A manual Select on top of `gpui::deferred(anchored())` with our tokens is
//! simpler and more faithful to the canon. (The pin's `Combobox` was also
//! examined — same coupling to its own theme; same verdict.)
//!
//! Anatomy:
//! - Trigger (closed): 32px, input-like (bg by mode: dark=bg, light=surface-2),
//!   border_2 1px, RADIUS_INPUT(8), fs-md 13, label of active option + chevron
//!   (IconName::Chevron from the 22-glyph catalog). Focus = oby ring.
//! - Popup (open, OVERLAY): surface-3 bg (JandiExt), border_2 1px,
//!   RADIUS_CARD(10), shadow_md() (gpui built-in — the shadow_overlay token
//!   is an Hsla value that gpui's auto-generated shadow API does not accept;
//!   shadow_md() is the same method the pin's own Select/Combobox use),
//!   width ≥ trigger.
//!   Items: fs-md, padding 7×9, hover surface_3, selected=check 12px + text,
//!   unselected=text_2. Sans by default (human labels); .mono(false).
//!
//! API: caller-owned selection. The selected index is passed to the builder;
//! `.on_change(handler)` notifies the caller when the user picks a new option.
//!
//! ```ignore
//! use stand_in_mcp_explorer_ds::forms::Select;
//!
//! Select::new("lang", vec![("pt", "Português"), ("en", "English")], 0)
//!     .on_change(move |ix, _value, _window, cx| {
//!         this.selected_lang = ix;
//!         cx.notify();
//!     });
//! ```

use gpui::prelude::FluentBuilder as _;
use gpui::{
    App, Bounds, ClickEvent, ElementId, Entity, FontWeight, InteractiveElement, IntoElement,
    MouseDownEvent, ParentElement, Pixels, RenderOnce, SharedString, StatefulInteractiveElement,
    Styled, Transformation, Window, anchored, deferred, div, percentage, px,
};
use gpui_component::{ActiveTheme as _, ElementExt as _, ThemeMode, v_flex};

use crate::core::icon::{Icon, IconName, IconSize};
use crate::theme::colors::JandiExt;
use crate::theme::density::{RADIUS_CARD, RADIUS_INPUT};
use crate::theme::typography;

use std::sync::Arc;

// ---------------------------------------------------------------------------
// SelectEntity — internal popup-state per instance
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct SelectEntity {
    open: bool,
    trigger_bounds: Bounds<Pixels>,
}

impl SelectEntity {
    fn new() -> Self {
        Self {
            open: false,
            trigger_bounds: Bounds::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Select — builder
// ---------------------------------------------------------------------------

/// Change handler: (new_index, value_string, window, cx).
pub type SelectHandler = Arc<dyn Fn(usize, SharedString, &mut Window, &mut App) + 'static>;

/// Compact single-choice dropdown (32px trigger + overlay option list).
///
/// The caller owns the selected index. Pass options via the builder and react
/// to selections via `.on_change(handler)`. 2–10 options of short labels.
/// More than 10, or with search → use a different pattern.
///
/// Labels are sans by default (human-readable); `.mono(true)` for technical
/// values (e.g. environment names).
#[derive(IntoElement)]
pub struct Select {
    id: ElementId,
    options: Vec<(SharedString, SharedString)>,
    selected_index: usize,
    placeholder: Option<SharedString>,
    mono: bool,
    width: Option<Pixels>,
    on_change: Option<SelectHandler>,
}

impl Select {
    pub fn new(
        id: impl Into<ElementId>,
        options: Vec<(impl Into<SharedString>, impl Into<SharedString>)>,
        selected_index: usize,
    ) -> Self {
        Self {
            id: id.into(),
            options: options
                .into_iter()
                .map(|(v, l)| (v.into(), l.into()))
                .collect(),
            selected_index,
            placeholder: None,
            mono: false,
            width: None,
            on_change: None,
        }
    }

    /// Placeholder text shown when no option is selected.
    /// If unset and `selected_index >= options.len()`, the trigger shows nothing.
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// When `true` (default `false`), trigger label uses the mono font.
    /// Human-readable labels (language names, etc.) should stay `false`.
    pub fn mono(mut self, yes: bool) -> Self {
        self.mono = yes;
        self
    }

    /// Fix the control width. Without it the Select is `w_full` (fills its
    /// parent) — which, inside a content-sized container (e.g. the topbar caps
    /// cluster, `flex_none`), makes the width follow the active label and
    /// "float" per option. Set a fixed width there for a stable control
    /// (028 QA Item #16). The popup follows the trigger width.
    pub fn width(mut self, w: impl Into<Pixels>) -> Self {
        self.width = Some(w.into());
        self
    }

    /// Callback fired when the user selects an option.
    /// Receives `(new_index, value, window, cx)`.
    pub fn on_change(
        mut self,
        handler: impl Fn(usize, SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for Select {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = cx.theme().clone();
        let mode = t.mode;
        let colors = &t.colors;
        let ext = cx.global::<JandiExt>().clone();
        let width = self.width;

        // Input background per mode: dark = bg, light = surface-2.
        let input_bg = match mode {
            ThemeMode::Light => colors.secondary,
            _ => colors.background,
        };

        let font_family = if self.mono {
            t.mono_font_family.clone()
        } else {
            t.font_family.clone()
        };

        // Persistent popup state per element id.
        let state: Entity<SelectEntity> =
            window.use_keyed_state(self.id.clone(), cx, |_window, _cx| SelectEntity::new());

        let is_open = state.read(cx).open;
        let trigger_bounds = state.read(cx).trigger_bounds;

        let active_label: SharedString = self
            .options
            .get(self.selected_index)
            .map(|(_, l)| l.clone())
            .or_else(|| self.placeholder.clone())
            .unwrap_or_default();

        // ---------- Trigger ----------
        let trigger = div()
            .id("select-trigger")
            .flex()
            .items_center()
            .justify_between()
            .w_full()
            .h(px(32.0))
            .bg(input_bg)
            .border_1()
            .border_color(ext.border_2)
            .rounded(px(RADIUS_INPUT))
            .px(px(11.0))
            .gap(px(8.0))
            .text_size(px(typography::FS_MD))
            .text_color(colors.foreground)
            .font_family(font_family.clone())
            .font_weight(FontWeight::NORMAL)
            .cursor_pointer()
            .on_click({
                let s = state.clone();
                move |_ev: &ClickEvent, _window: &mut Window, app: &mut App| {
                    s.update(app, |se, cx| {
                        se.open = !se.open;
                        cx.notify();
                    });
                }
            })
            .on_prepaint({
                let s = state.clone();
                move |bounds, _, app| {
                    s.update(app, |se, _| se.trigger_bounds = bounds);
                }
            })
            .child(
                div()
                    .id("select-label")
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .truncate()
                    .child(active_label),
            )
            .child(
                Icon::new(IconName::Chevron)
                    .size(IconSize::Xs)
                    .rotate(Transformation::rotate(percentage(0.25))),
            );

        // ---------- Popup (open only) ----------
        if is_open {
            let options = self.options;
            let selected_index = self.selected_index;
            let on_change = self.on_change;
            let font_family_popup = font_family.clone();

            let popup = deferred(
                anchored()
                    .snap_to_window_with_margin(px(8.0))
                    .child(
                        div()
                            .occlude()
                            .w(trigger_bounds.size.width + px(2.0))
                            .child(
                                v_flex()
                                    .occlude()
                                    .mt(px(6.0))
                                    .bg(colors.secondary)
                                    .border_1()
                                    .border_color(ext.border_2)
                                    .rounded(px(RADIUS_CARD))
                                    .shadow_md()
                                    .p(px(4.0))
                                    .children(options.iter().enumerate().map(
                                        |(i, (_val, label))| {
                                            let is_selected = i == selected_index;
                                            let value: SharedString = options[i].0.clone();
                                            let label = label.clone();

                                            let row = div()
                                                .id(("select-item", i))
                                                .flex()
                                                .items_center()
                                                .gap(px(8.0))
                                                .px(px(9.0))
                                                .py(px(7.0))
                                                .rounded(px(RADIUS_INPUT))
                                                .text_size(px(typography::FS_MD))
                                                .text_color(if is_selected {
                                                    colors.foreground
                                                } else {
                                                    colors.secondary_foreground
                                                })
                                                .font_family(font_family_popup.clone())
                                                .cursor_pointer()
                                                .hover(|h| h.bg(ext.surface_3))
                                                .child(if is_selected {
                                                    Icon::new(IconName::Check)
                                                        .size(IconSize::Xs)
                                                        .into_any_element()
                                                } else {
                                                    div().w(px(12.0)).into_any_element()
                                                })
                                                .child(label);

                                            if on_change.is_some() {
                                                let s = state.clone();
                                                let v = value.clone();
                                                let oc = on_change.clone();
                                                row.on_click(move |_ev: &ClickEvent, win: &mut Window, app: &mut App| {
                                                    s.update(app, |se, cx| {
                                                        se.open = false;
                                                        cx.notify();
                                                    });
                                                    if let Some(ref handler) = oc {
                                                        handler(i, v.clone(), win, app);
                                                    }
                                                })
                                                .into_any_element()
                                            } else {
                                                row.into_any_element()
                                            }
                                        },
                                    )),
                            )
                            .on_mouse_down_out({
                                let s = state.clone();
                                move |_ev: &MouseDownEvent, _window: &mut Window, app: &mut App| {
                                    s.update(app, |se, cx| {
                                        se.open = false;
                                        cx.notify();
                                    });
                                }
                            }),
                    ),
            )
            .with_priority(1);

            div()
                .id(self.id.clone())
                .relative()
                .map(|d| match width {
                    Some(w) => d.w(w),
                    None => d.w_full(),
                })
                .child(trigger)
                .child(popup)
        } else {
            div()
                .id(self.id.clone())
                .relative()
                .map(|d| match width {
                    Some(w) => d.w(w),
                    None => d.w_full(),
                })
                .child(trigger)
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_construction() {
        let sel = Select::new("lang", vec![("pt", "Português"), ("en", "English")], 0);
        assert_eq!(sel.id, ElementId::from("lang"));
        assert_eq!(sel.options.len(), 2);
        assert_eq!(sel.selected_index, 0);
        assert_eq!(sel.options[0].1.as_ref(), "Português");
        assert!(!sel.mono);
    }

    #[test]
    fn test_select_placeholder() {
        let sel = Select::new("sel", vec![("a", "A")], 0).placeholder("Pick...");
        assert_eq!(sel.placeholder.unwrap().as_ref(), "Pick...");
    }

    #[test]
    fn test_select_mono() {
        let sel = Select::new("sel", vec![("a", "A")], 0).mono(true);
        assert!(sel.mono);
    }

    #[test]
    fn test_select_default_mono_false() {
        let sel = Select::new("sel", vec![("a", "A")], 0);
        assert!(!sel.mono);
    }

    #[test]
    fn test_select_default_width_none() {
        let sel = Select::new("sel", vec![("a", "A")], 0);
        assert!(sel.width.is_none());
    }

    #[test]
    fn test_select_fixed_width() {
        let sel = Select::new("sel", vec![("a", "A")], 0).width(px(128.0));
        assert_eq!(sel.width, Some(px(128.0)));
    }

    #[test]
    fn test_select_allows_two_to_ten_options() {
        for n in [2, 5, 10] {
            let opts: Vec<(String, String)> =
                (0..n).map(|i| (format!("v{i}"), format!("L{i}"))).collect();
            let sel = Select::new("sel", opts, 0);
            assert_eq!(sel.options.len(), n);
        }
    }

    #[test]
    fn test_select_no_handler_still_constructs() {
        let sel = Select::new("sel", vec![("a", "A"), ("b", "B")], 1);
        assert_eq!(sel.selected_index, 1);
        assert!(sel.on_change.is_none());
    }
}
