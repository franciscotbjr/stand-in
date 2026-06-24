//! CopyButton — small copy-to-clipboard chip with a 1.3s check-mark
//! confirmation. Pair it with every code block, JSON view, or technical value
//! worth copying.
//!
//! Anatomy: chip `surface-2` + 1px border (`border`) + text `text-2`, radius
//! RADIUS_CHIP (7), padding 4×9, fs-xs (11) weight 600, inline-flex gap 5.
//! Icon `copy` 12px. Hover: text `text`, border `border_2`.
//!
//! Behaviour: click → `cx.write_to_clipboard(…)` → state `copied=true` via
//! `use_keyed_state` → icon `check` + label → `copied_label` →
//! timer 1.3s (`background_executor().timer`) reverts. During copied, new
//! clicks are ignored.
//!
//! Labels are parameterised (caller provides both; defaults "Copy" / "Copied"
//! as fallback). The canonical position is the `right` slot of a panel-head.

use std::time::{Duration, Instant};

use gpui::{
    App, AppContext, ClickEvent, ClipboardItem, ElementId, Entity, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, px,
};
use gpui_component::ActiveTheme as _;

use super::icon::{Icon, IconName, IconSize};
use crate::theme::colors::JandiExt;
use crate::theme::density::RADIUS_CHIP;
use crate::theme::typography;

use super::button::ClickHandler;

/// Duration of the copied confirmation (ms). Testable as a constant.
pub const COPY_DURATION_MS: u64 = 1300;

/// Internal state persisted per CopyButton instance via `use_keyed_state`.
/// Uses an expiry-based approach — the is_copied() check auto-expires based
/// on real time, so the visual state clears on the next render even if the
/// parent delays re-rendering.
#[derive(Debug)]
struct CopyEntity {
    copied_until: Option<Instant>,
}

impl CopyEntity {
    fn is_copied(&self) -> bool {
        self.copied_until
            .map(|t| Instant::now() < t)
            .unwrap_or(false)
    }

    fn set_copied(&mut self) {
        self.copied_until = Some(Instant::now() + Duration::from_millis(COPY_DURATION_MS));
    }
}

// ---------------------------------------------------------------------------
// CopyButton
// ---------------------------------------------------------------------------

/// Small copy-to-clipboard chip. Renders as a labelled chip; clicking copies
/// the value and shows a 1.3s check-mark confirmation.
///
/// ```ignore
/// use stand_in_mcp_explorer_ds::core::CopyButton;
///
/// CopyButton::new("copy-json", json_payload)
///     .label("Copiar")
///     .copied_label("Copiado")
///     .id("copy-json-btn");
/// ```
#[derive(IntoElement)]
pub struct CopyButton {
    value: SharedString,
    label: SharedString,
    copied_label: SharedString,
    id: ElementId,
    on_copied: Option<ClickHandler>,
}

impl CopyButton {
    /// Create a CopyButton with the given element id and value to copy.
    pub fn new(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: SharedString::from("Copy"),
            copied_label: SharedString::from("Copied"),
            id: id.into(),
            on_copied: None,
        }
    }

    /// Label shown in the idle state.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    /// Label shown during the 1.3s confirmation.
    pub fn copied_label(mut self, label: impl Into<SharedString>) -> Self {
        self.copied_label = label.into();
        self
    }

    /// Override the element id (defaults to the one passed to `new`).
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    /// Optional callback fired after a successful copy (idle → copied).
    pub fn on_copied(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_copied = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for CopyButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = cx.theme().clone();
        let colors = &t.colors;
        let ext = cx.global::<JandiExt>().clone();

        // Persistent copied flag via window.use_keyed_state (entity per element id).
        let state: Entity<CopyEntity> =
            window.use_keyed_state(self.id.clone(), cx, |_window, _cx| CopyEntity {
                copied_until: None,
            });
        let copied = state.read(cx).is_copied();

        let label_text = if copied {
            self.copied_label.clone()
        } else {
            self.label.clone()
        };

        let value = self.value.clone();
        let on_copied = self.on_copied;

        // gpui's Div defaults to display:block — flex must be explicit or
        // gap/items_center are inert and icon/label stack vertically.
        let mut el = gpui::div()
            .id(self.id)
            .flex()
            .bg(colors.secondary)
            .text_color(colors.secondary_foreground)
            .border_1()
            .border_color(colors.border)
            .rounded(px(RADIUS_CHIP))
            .px(px(9.))
            .py(px(4.))
            .gap(px(5.))
            .text_size(px(typography::FS_XS))
            .font_weight(typography::W_SEMIBOLD)
            .whitespace_nowrap()
            .items_center()
            .cursor_pointer()
            .hover(|h| {
                h.text_color(colors.foreground)
                    .border_color(ext.border_2)
                    .bg(colors.secondary) // bg stays the same on hover
            });

        if !copied {
            el = el.child(Icon::new(IconName::Copy).size(IconSize::Xs));
        } else {
            el = el.child(Icon::new(IconName::Check).size(IconSize::Xs));
        }
        el = el.child(label_text);

        let state_entity = state.clone();

        if !copied {
            el = el.on_click(move |_ev: &ClickEvent, win: &mut Window, app: &mut App| {
                if state_entity.read(app).is_copied() {
                    return;
                }

                app.update_entity(&state_entity, |s, cx| {
                    s.set_copied();
                    cx.notify();
                });

                app.write_to_clipboard(ClipboardItem::new_string(value.to_string()));

                let st = state_entity.clone();
                app.spawn(async move |cx| {
                    cx.background_executor()
                        .timer(Duration::from_millis(COPY_DURATION_MS))
                        .await;
                    st.update(cx, |s, cx| {
                        s.copied_until = None;
                        cx.notify();
                    });
                })
                .detach();

                if let Some(ref callback) = on_copied {
                    callback(_ev, win, app);
                }
            });
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
    fn test_copy_button_construction_defaults() {
        let cb = CopyButton::new("cb1", "hello");
        assert_eq!(cb.value.as_ref(), "hello");
        assert_eq!(cb.label.as_ref(), "Copy");
        assert_eq!(cb.copied_label.as_ref(), "Copied");
        assert_eq!(cb.id, ElementId::from("cb1"));
    }

    #[test]
    fn test_copy_button_builder_api() {
        let cb = CopyButton::new("cb2", "{\"key\": 42}")
            .label("Copiar")
            .copied_label("Copiado")
            .id("copy-json");
        assert_eq!(cb.label.as_ref(), "Copiar");
        assert_eq!(cb.copied_label.as_ref(), "Copiado");
        assert_eq!(cb.id, ElementId::from("copy-json"));
        assert_eq!(cb.value.as_ref(), "{\"key\": 42}");
    }

    #[test]
    fn test_copy_duration_is_1300ms() {
        assert_eq!(COPY_DURATION_MS, 1300);
    }

    #[test]
    fn test_copy_duration_is_1300ms_std() {
        let dur = Duration::from_millis(COPY_DURATION_MS);
        assert_eq!(dur, Duration::from_millis(1300));
    }

    #[test]
    fn test_chip_geometry_constants() {
        assert_eq!(RADIUS_CHIP, 7.0);
        assert_eq!(typography::FS_XS, 11.0);
    }
}
