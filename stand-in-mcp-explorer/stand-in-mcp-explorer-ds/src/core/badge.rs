//! Badge — small mono badge encoding the technical nature of an item.
//!
//! 1:1 with `core/Badge.jsx` + `.badge*` rules in `core/core.css`. Five fixed
//! semantic kinds — never reinterpret, never create new ones:
//! `Read` (ok, safe), `Write` (warn, attention), `Mime` (content types),
//! `Muted` (metadata), `Role` (categories/roles).
//!
//! Anatomy: inline-flex, gap 5, fs-2xs (10.5), weight 600, padding 2×7,
//! radius RADIUS_BADGE (6), mono SEMPRE, tracking 0.01em, nowrap.
//! Optional icon 12px (gap 5 already accounted for). Not interactive.

use gpui::{
    App, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window, px,
};
use gpui_component::{ActiveTheme as _, ThemeColor};

use super::icon::{Icon, IconName, IconSize};
use crate::theme::colors::JandiExt;
use crate::theme::density::RADIUS_BADGE;
use crate::theme::typography;

// ---------------------------------------------------------------------------
// BadgeKind
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeKind {
    /// Read operations, safe states — ok_dim bg + ok fg.
    Read,
    /// Write operations, attention — warn_dim bg + warn fg.
    Write,
    /// Content types — surface_3 bg + text_2 fg.
    Mime,
    /// Metadata without charge — surface_2 bg + text_3 fg.
    Muted,
    /// Roles and categories — oby-18% bg + OBY fg.
    Role,
}

impl BadgeKind {
    pub fn bg(&self, colors: &ThemeColor, ext: &JandiExt) -> Hsla {
        match self {
            BadgeKind::Read => ext.ok_dim,
            BadgeKind::Write => ext.warn_dim,
            BadgeKind::Mime => ext.surface_3,
            BadgeKind::Muted => colors.secondary, // surface-2
            BadgeKind::Role => Hsla {
                h: crate::theme::palette::OBY.h,
                s: crate::theme::palette::OBY.s,
                l: crate::theme::palette::OBY.l,
                a: 0.18,
            },
        }
    }

    pub fn fg(&self, colors: &ThemeColor) -> Hsla {
        match self {
            BadgeKind::Read => colors.success_foreground,   // ok
            BadgeKind::Write => colors.warning_foreground,  // warn
            BadgeKind::Mime => colors.secondary_foreground, // text-2
            BadgeKind::Muted => colors.muted_foreground,    // text-3
            BadgeKind::Role => {
                // OBY opaque — the raw palette const (not derived from ThemeColor.link
                // which IS OBY but might change independently)
                crate::theme::palette::OBY
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Badge
// ---------------------------------------------------------------------------

/// Small mono badge encoding the technical nature of an item.
///
/// Text should always be lowercase and short (1–2 words). The badge is
/// mono by default and never interactive. Icon is optional, always 12px.
///
/// ```ignore
/// use stand_in_mcp_explorer_ds::core::{Badge, BadgeKind};
///
/// Badge::new("leitura", BadgeKind::Read)
/// Badge::new("stdio", BadgeKind::Muted)
/// Badge::new("assistant", BadgeKind::Role).icon(IconName::Chat)
/// ```
#[derive(IntoElement)]
pub struct Badge {
    label: SharedString,
    kind: BadgeKind,
    icon: Option<IconName>,
    id: ElementId,
}

impl Badge {
    pub fn new(label: impl Into<SharedString>, kind: BadgeKind) -> Self {
        Self {
            label: label.into(),
            kind,
            icon: None,
            id: ElementId::from("badge"),
        }
    }

    pub fn icon(mut self, name: IconName) -> Self {
        self.icon = Some(name);
        self
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }
}

impl RenderOnce for Badge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = cx.theme();
        let colors = &t.colors;
        let ext = cx.global::<JandiExt>();

        let bg = self.kind.bg(colors, ext);
        let fg = self.kind.fg(colors);

        // gpui's Div defaults to display:block — flex must be explicit or
        // gap/items_center are inert and icon/label stack vertically.
        let mut el = gpui::div()
            .id(self.id)
            .flex()
            .bg(bg)
            .text_color(fg)
            .rounded(px(RADIUS_BADGE))
            .px(px(7.))
            .py(px(2.))
            .gap(px(5.))
            .text_size(px(typography::FS_2XS))
            .font_weight(typography::W_SEMIBOLD)
            .font_family(t.mono_font_family.clone())
            .whitespace_nowrap()
            .items_center();

        if let Some(icon_name) = self.icon {
            el = el.child(Icon::new(icon_name).size(IconSize::Xs));
        }

        el = el.child(self.label);

        el
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::colors::{JandiExt, jandi_theme};
    use gpui_component::ThemeMode;

    #[test]
    fn test_badge_construction_defaults() {
        let badge = Badge::new("leitura", BadgeKind::Read);
        assert_eq!(badge.label.as_ref(), "leitura");
        assert_eq!(badge.kind, BadgeKind::Read);
        assert!(badge.icon.is_none());
    }

    #[test]
    fn test_badge_with_icon() {
        let badge = Badge::new("assistant", BadgeKind::Role).icon(IconName::Chat);
        assert_eq!(badge.icon, Some(IconName::Chat));
    }

    #[test]
    fn test_read_colors_dark() {
        let colors = jandi_theme(ThemeMode::Dark);
        let ext = JandiExt::dark();
        assert_eq!(BadgeKind::Read.bg(&colors, &ext), ext.ok_dim);
        assert_eq!(BadgeKind::Read.fg(&colors), colors.success_foreground);
    }

    #[test]
    fn test_write_colors_dark() {
        let colors = jandi_theme(ThemeMode::Dark);
        let ext = JandiExt::dark();
        assert_eq!(BadgeKind::Write.bg(&colors, &ext), ext.warn_dim);
        assert_eq!(BadgeKind::Write.fg(&colors), colors.warning_foreground);
    }

    #[test]
    fn test_mime_colors_dark() {
        let colors = jandi_theme(ThemeMode::Dark);
        let ext = JandiExt::dark();
        assert_eq!(BadgeKind::Mime.bg(&colors, &ext), ext.surface_3);
        assert_eq!(BadgeKind::Mime.fg(&colors), colors.secondary_foreground);
    }

    #[test]
    fn test_muted_colors_dark() {
        let colors = jandi_theme(ThemeMode::Dark);
        let ext = JandiExt::dark();
        assert_eq!(BadgeKind::Muted.bg(&colors, &ext), colors.secondary);
        assert_eq!(BadgeKind::Muted.fg(&colors), colors.muted_foreground);
    }

    #[test]
    fn test_role_bg_is_oby_18pct() {
        let colors = jandi_theme(ThemeMode::Dark);
        let ext = JandiExt::dark();
        let role_bg = BadgeKind::Role.bg(&colors, &ext);
        assert_eq!(role_bg.h, crate::theme::palette::OBY.h);
        assert_eq!(role_bg.s, crate::theme::palette::OBY.s);
        assert_eq!(role_bg.l, crate::theme::palette::OBY.l);
        assert_eq!(role_bg.a, 0.18);
    }

    #[test]
    fn test_role_fg_is_oby_opaque() {
        let colors = jandi_theme(ThemeMode::Dark);
        assert_eq!(BadgeKind::Role.fg(&colors), crate::theme::palette::OBY);
    }

    #[test]
    fn test_read_not_write() {
        let colors = jandi_theme(ThemeMode::Dark);
        let ext = JandiExt::dark();
        assert_ne!(
            BadgeKind::Read.bg(&colors, &ext),
            BadgeKind::Write.bg(&colors, &ext)
        );
        assert_ne!(BadgeKind::Read.fg(&colors), BadgeKind::Write.fg(&colors));
    }

    #[test]
    fn test_all_five_kinds_distinct_bg() {
        let colors = jandi_theme(ThemeMode::Dark);
        let ext = JandiExt::dark();
        let kinds = &[
            BadgeKind::Read,
            BadgeKind::Write,
            BadgeKind::Mime,
            BadgeKind::Muted,
            BadgeKind::Role,
        ];
        for i in 0..kinds.len() {
            for j in i + 1..kinds.len() {
                assert_ne!(
                    kinds[i].bg(&colors, &ext),
                    kinds[j].bg(&colors, &ext),
                    "kind {:?} and {:?} have same bg",
                    kinds[i],
                    kinds[j],
                );
            }
        }
    }

    #[test]
    fn test_badge_geometry_constants() {
        assert_eq!(RADIUS_BADGE, 6.0);
        assert_eq!(typography::FS_2XS, 10.5);
    }
}
