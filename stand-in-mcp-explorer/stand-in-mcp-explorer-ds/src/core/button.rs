//! Button — the primary action control (primary · ghost · danger × md · sm).
//!
//! 1:1 with `core/Button.jsx` + the `.btn*` rules in `core/core.css`. Fixed
//! semantics (never reinterpret): `Primary` at most once per view; `Danger`
//! only for real destructive actions (disconnect, remove); `Ghost` for
//! everything secondary. No "link" variant — the toggle-link arrives in M6.
//!
//! Anatomy: inline-flex, gap 8, weight 650, nowrap; icon ALWAYS before text
//! (size 15 md / 13 sm). Active shifts 1px down (embedded, never added by
//! the caller). Loading replaces the icon with `Spinner` and disables.
//!
//! Heights are FIXED (40 md / 32 sm) — never stretch vertically. Radii are
//! fixed per role: RADIUS_BTN = 9 (md), RADIUS_INPUT = 8 (sm).

use gpui::{
    App, ClickEvent, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, div, px,
};

/// Click handler callback signature used by interactive components.
pub type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
use gpui_component::{ActiveTheme as _, ThemeColor};

use super::icon::{Icon, IconName, IconSize};
use super::spinner::Spinner;
use crate::theme::colors::JandiExt;
use crate::theme::density::{RADIUS_BTN, RADIUS_INPUT};
use crate::theme::typography;

// ---------------------------------------------------------------------------
// ButtonVariant
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Primary,
    Ghost,
    Danger,
}

impl ButtonVariant {
    fn bg(&self, colors: &ThemeColor, ext: &JandiExt) -> Hsla {
        match self {
            ButtonVariant::Primary => colors.button_primary,
            ButtonVariant::Ghost => colors.secondary,
            ButtonVariant::Danger => ButtonVariant::danger_dim_bg(ext),
        }
    }

    fn fg(&self, colors: &ThemeColor) -> Hsla {
        match self {
            ButtonVariant::Primary => colors.button_primary_foreground,
            ButtonVariant::Ghost => colors.foreground,
            ButtonVariant::Danger => colors.danger,
        }
    }

    fn border(&self, colors: &ThemeColor) -> Option<Hsla> {
        match self {
            ButtonVariant::Primary | ButtonVariant::Danger => None, // transparent border
            ButtonVariant::Ghost => Some(colors.border),
        }
    }

    fn hover_bg(&self, colors: &ThemeColor, ext: &JandiExt) -> Hsla {
        match self {
            ButtonVariant::Primary => colors.button_primary_hover,
            ButtonVariant::Ghost => ext.surface_3,
            // Danger hover: bg = err, text = white (= on_primary)
            ButtonVariant::Danger => colors.danger,
        }
    }

    fn hover_fg(&self, colors: &ThemeColor) -> Hsla {
        match self {
            ButtonVariant::Danger => colors.button_primary_foreground, // white on danger hover
            _ => self.fg(colors),
        }
    }

    fn hover_border(&self, ext: &JandiExt) -> Option<Hsla> {
        match self {
            ButtonVariant::Ghost => Some(ext.border_2),
            _ => None,
        }
    }

    /// Danger uses a dim background (`err-dim`) with the err foreground.
    /// The default ThemeColor bg is `danger_foreground` (which is `err` itself),
    /// so we override in the render method with the dim variant.
    fn danger_dim_bg(ext: &JandiExt) -> Hsla {
        ext.err_dim
    }
}

// ---------------------------------------------------------------------------
// ButtonSize
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    /// 40px height, padding 0 16, fs 13, radius 9 (RADIUS_BTN).
    Md,
    /// 32px height, padding 0 12, fs 12, radius 8 (RADIUS_INPUT).
    Sm,
}

impl ButtonSize {
    pub fn height_px(self) -> f32 {
        match self {
            ButtonSize::Md => 40.0,
            ButtonSize::Sm => 32.0,
        }
    }

    pub fn padding_x(self) -> f32 {
        match self {
            ButtonSize::Md => 16.0,
            ButtonSize::Sm => 12.0,
        }
    }

    pub fn font_size(self) -> f32 {
        match self {
            ButtonSize::Md => typography::FS_MD, // 13
            ButtonSize::Sm => typography::FS_SM, // 12
        }
    }

    pub fn radius(self) -> f32 {
        match self {
            ButtonSize::Md => RADIUS_BTN,   // 9
            ButtonSize::Sm => RADIUS_INPUT, // 8
        }
    }

    pub fn icon_size(self) -> IconSize {
        match self {
            ButtonSize::Md => IconSize::Md, // 15
            ButtonSize::Sm => IconSize::Xs, // 12 — closest to canon's "13–14"
        }
    }
}

// ---------------------------------------------------------------------------
// Button
// ---------------------------------------------------------------------------

/// Standard action button. Use `Primary` at most once per view; `Ghost` for
/// everything secondary; `Danger` only for real destructive actions.
///
/// ```ignore
/// use stand_in_mcp_explorer_ds::core::{Button, ButtonVariant, IconName};
///
/// Button::new("Conectar")
///     .variant(ButtonVariant::Primary)
///     .icon(IconName::Play)
///     .id("connect-btn");
/// ```
#[derive(IntoElement)]
pub struct Button {
    label: SharedString,
    variant: ButtonVariant,
    size: ButtonSize,
    block: bool,
    disabled: bool,
    loading: bool,
    icon: Option<IconName>,
    id: ElementId,
    on_click: Option<ClickHandler>,
}

impl Button {
    /// Create a standard button with the given label.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            variant: ButtonVariant::Primary,
            size: ButtonSize::Md,
            block: false,
            disabled: false,
            loading: false,
            icon: None,
            id: ElementId::from("button"),
            on_click: None,
        }
    }

    /// Set the variant.
    pub fn variant(mut self, v: ButtonVariant) -> Self {
        self.variant = v;
        self
    }

    /// Set the size.
    pub fn size(mut self, s: ButtonSize) -> Self {
        self.size = s;
        self
    }

    /// Shortcut: small size.
    pub fn sm(mut self) -> Self {
        self.size = ButtonSize::Sm;
        self
    }

    /// Full-width button.
    pub fn block(mut self) -> Self {
        self.block = true;
        self
    }

    /// Disable the button (opacity .5, cursor not-allowed, click ignored).
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    /// Show loading state — replaces icon with Spinner and disables.
    pub fn loading(mut self) -> Self {
        self.loading = true;
        self
    }

    /// Add an icon before the label.
    pub fn icon(mut self, name: IconName) -> Self {
        self.icon = Some(name);
        self
    }

    /// Set the element id (required for interactivity).
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    /// Attach a click handler. The caller is responsible for wiring
    /// `cx.listener` / `cx.notify` into the closure if view notification
    /// is needed.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Button {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = cx.theme();
        let colors = &t.colors;
        let ext = cx.global::<JandiExt>();
        let size = self.size;
        let is_disabled = self.disabled || self.loading;

        let raw_bg = self.variant.bg(colors, ext);

        let fg = self.variant.fg(colors);
        let border = self.variant.border(colors);
        let hover_bg = self.variant.hover_bg(colors, ext);
        let hover_fg = self.variant.hover_fg(colors);
        let hover_border = self.variant.hover_border(ext);

        let icon_name = self.icon;
        let label = self.label.clone();

        // gpui's Div defaults to display:block — flex must be explicit or
        // items_center/justify_center are inert (label lands top-left).
        let mut el = div()
            .id(self.id)
            .flex()
            // Never shrink below the label: `overflow_hidden` (below) zeroes the
            // flex `min-width: auto`, so without this a tight flex parent squashes
            // the button and clips the centred text (028 Item #9 — "Modo guiado").
            .flex_shrink_0()
            .h(px(size.height_px()))
            .px(px(size.padding_x()))
            .gap(px(8.))
            .text_size(px(size.font_size()))
            .font_weight(typography::W_SEMIBOLD)
            .rounded(px(size.radius()))
            .items_center()
            .justify_center()
            .whitespace_nowrap()
            .overflow_hidden()
            .bg(raw_bg)
            .text_color(fg)
            .cursor_pointer()
            // Active: 1px translate down (embedded, never added by the caller)
            .active(|a| a.top(px(1.)))
            .hover(|h| {
                let mut h = h.bg(hover_bg).text_color(hover_fg);
                if let Some(hb) = hover_border {
                    h = h.border_color(hb);
                }
                h
            });

        if let Some(bc) = border {
            el = el.border_1().border_color(bc);
        }

        if self.block {
            el = el.w_full();
        }

        if is_disabled {
            el = el.opacity(0.5).cursor_default();
        }

        el = if self.loading {
            el.child(Spinner::new().id("button-spinner").color(fg))
        } else if let Some(icon) = icon_name {
            el.child(Icon::new(icon).size(size.icon_size()))
        } else {
            el
        };

        el = el.child(label);

        if !is_disabled && let Some(click) = self.on_click {
            el = el.on_click(click);
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
    fn test_button_construction_defaults() {
        let btn = Button::new("OK");
        assert_eq!(btn.label.as_ref(), "OK");
        assert_eq!(btn.variant, ButtonVariant::Primary);
        assert_eq!(btn.size, ButtonSize::Md);
        assert!(!btn.block);
        assert!(!btn.disabled);
        assert!(!btn.loading);
        assert!(btn.icon.is_none());
    }

    #[test]
    fn test_button_builder_api() {
        let btn = Button::new("Delete")
            .variant(ButtonVariant::Danger)
            .sm()
            .block()
            .disabled()
            .icon(IconName::X)
            .id("del-btn");
        assert_eq!(btn.variant, ButtonVariant::Danger);
        assert_eq!(btn.size, ButtonSize::Sm);
        assert!(btn.block);
        assert!(btn.disabled);
        assert_eq!(btn.icon, Some(IconName::X));
        assert_eq!(btn.id, ElementId::from("del-btn"));
    }

    #[test]
    fn test_loading_implies_disabled() {
        let btn = Button::new("Wait").loading();
        assert!(btn.loading);
        assert!(!btn.disabled); // struct field
        // During render, is_disabled = disabled || loading → true
    }

    #[test]
    fn test_button_size_geometry() {
        assert_eq!(ButtonSize::Md.height_px(), 40.0);
        assert_eq!(ButtonSize::Sm.height_px(), 32.0);
        assert_eq!(ButtonSize::Md.padding_x(), 16.0);
        assert_eq!(ButtonSize::Sm.padding_x(), 12.0);
        assert_eq!(ButtonSize::Md.font_size(), typography::FS_MD);
        assert_eq!(ButtonSize::Sm.font_size(), typography::FS_SM);
        assert_eq!(ButtonSize::Md.radius(), RADIUS_BTN);
        assert_eq!(ButtonSize::Sm.radius(), RADIUS_INPUT);
    }

    #[test]
    fn test_variant_mapping_is_stable() {
        use crate::theme::colors::jandi_theme;
        use gpui_component::ThemeMode;

        let colors = jandi_theme(ThemeMode::Dark);
        let ext = JandiExt::dark();

        assert_eq!(
            ButtonVariant::Primary.bg(&colors, &ext),
            colors.button_primary
        );
        assert_eq!(
            ButtonVariant::Primary.fg(&colors),
            colors.button_primary_foreground
        );
        assert_eq!(ButtonVariant::Ghost.bg(&colors, &ext), colors.secondary);
        assert_eq!(ButtonVariant::Ghost.fg(&colors), colors.foreground);
        assert_eq!(ButtonVariant::Ghost.border(&colors), Some(colors.border));
        assert_eq!(ButtonVariant::Danger.fg(&colors), colors.danger);
        assert_eq!(ButtonVariant::Primary.border(&colors), None);
        assert_eq!(ButtonVariant::Danger.border(&colors), None);
    }

    #[test]
    fn test_danger_dim_is_ext_not_theme_color() {
        let ext = JandiExt::dark();
        let dim = ButtonVariant::danger_dim_bg(&ext);
        assert_eq!(dim, crate::theme::palette::ERR_DIM);
    }

    #[test]
    fn test_button_icon_size_mapping() {
        assert_eq!(ButtonSize::Md.icon_size(), IconSize::Md);
        assert_eq!(ButtonSize::Sm.icon_size(), IconSize::Xs);
    }

    #[test]
    fn test_button_default_id() {
        let btn = Button::new("Test");
        assert_eq!(btn.id, ElementId::from("button"));
    }
}
