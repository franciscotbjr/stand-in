//! Builds a `gpui_component::ThemeColor` per mode from the jandi palette.
//!
//! Mapping table: role → `ThemeColor` field. Source of truth for field names
//! is the pinned `gpui-component` source at `70d2c44b`. Honest gaps (surface-3,
//! warn/err dim, code-bg, JSON tokens, shadow) live in `JandiExt`.

use gpui::Hsla;
use gpui_component::{ThemeColor, ThemeMode};

use super::palette;

// ---------------------------------------------------------------------------
// JandiExt — fields the standard ThemeColor does not carry
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct JandiExt {
    pub surface: Hsla,
    pub surface_3: Hsla,
    pub border_2: Hsla,
    pub ok_dim: Hsla,
    pub warn_dim: Hsla,
    pub err_dim: Hsla,
    pub code_bg: Hsla,
    pub tok_key: Hsla,
    pub tok_str: Hsla,
    pub tok_num: Hsla,
    pub tok_bool: Hsla,
    pub shadow_overlay: Hsla,
}

impl gpui::Global for JandiExt {}

impl JandiExt {
    pub fn dark() -> Self {
        Self {
            surface: palette::dark::SURFACE,
            surface_3: palette::dark::SURFACE_3,
            border_2: palette::dark::BORDER_2,
            ok_dim: palette::OK_DIM,
            warn_dim: palette::WARN_DIM,
            err_dim: palette::ERR_DIM,
            code_bg: palette::dark::CODE_BG,
            tok_key: palette::TOK_KEY,
            tok_str: palette::TOK_STR,
            tok_num: palette::TOK_NUM,
            tok_bool: palette::TOK_BOOL,
            shadow_overlay: palette::dark::SHADOW,
        }
    }

    pub fn light() -> Self {
        Self {
            surface: palette::light::SURFACE,
            surface_3: palette::light::SURFACE_3,
            border_2: palette::light::BORDER_2,
            ok_dim: palette::OK_DIM,
            warn_dim: palette::WARN_DIM,
            err_dim: palette::ERR_DIM,
            code_bg: palette::light::CODE_BG,
            tok_key: palette::TOK_KEY,
            tok_str: palette::TOK_STR,
            tok_num: palette::TOK_NUM,
            tok_bool: palette::TOK_BOOL,
            shadow_overlay: palette::light::SHADOW,
        }
    }
}

// ---------------------------------------------------------------------------
// jandi_theme — produce a populated ThemeColor for the given mode
// ---------------------------------------------------------------------------

pub fn jandi_theme(mode: ThemeMode) -> ThemeColor {
    match mode {
        ThemeMode::Dark => jandi_dark(),
        ThemeMode::Light => jandi_light(),
    }
}

fn jandi_dark() -> ThemeColor {
    ThemeColor {
        background: palette::dark::BG,
        foreground: palette::dark::TEXT,
        muted_foreground: palette::dark::TEXT_3,
        secondary: palette::dark::SURFACE_2,
        secondary_foreground: palette::dark::TEXT_2,
        muted: palette::dark::SURFACE_2,
        border: palette::dark::BORDER,
        primary: palette::dark::PRIMARY,
        primary_foreground: palette::dark::ON_PRIMARY,
        primary_hover: palette::dark::PRIMARY_H,
        primary_active: palette::dark::PRIMARY_H,
        button_primary: palette::dark::PRIMARY,
        button_primary_hover: palette::dark::PRIMARY_H,
        button_primary_active: palette::dark::PRIMARY_H,
        button_primary_foreground: palette::dark::ON_PRIMARY,
        success: palette::OK,
        warning: palette::WARN,
        danger: palette::ERR,
        sidebar: palette::dark::SURFACE,
        sidebar_foreground: palette::dark::TEXT,
        sidebar_border: palette::dark::BORDER,
        tab_bar: palette::dark::SURFACE,
        tab: palette::dark::SURFACE,
        tab_active: palette::dark::SURFACE_2,
        tab_active_foreground: palette::dark::TEXT,
        tab_foreground: palette::dark::TEXT_3,
        list: palette::dark::BG,
        list_active: palette::dark::SURFACE_2,
        list_active_border: palette::OBY,
        list_hover: palette::dark::BORDER_2,
        list_even: palette::dark::SURFACE,
        list_head: palette::dark::SURFACE,
        accent: palette::dark::SURFACE_2,
        accent_foreground: palette::dark::TEXT,
        ring: Hsla {
            h: palette::OBY.h,
            s: palette::OBY.s,
            l: palette::OBY.l,
            a: 0.22,
        },
        scrollbar: palette::dark::BORDER,
        scrollbar_thumb: palette::dark::BORDER_2,
        scrollbar_thumb_hover: palette::OBY,
        input: palette::dark::BORDER_2,
        caret: palette::dark::TEXT,
        link: palette::OBY,
        link_hover: palette::BRISA,
        link_active: palette::JANDI,
        selection: Hsla {
            h: palette::OBY.h,
            s: palette::OBY.s,
            l: palette::OBY.l,
            a: 0.22,
        },
        popover: palette::dark::SURFACE_3,
        popover_foreground: palette::dark::TEXT,
        info: Hsla {
            h: palette::OBY.h,
            s: palette::OBY.s,
            l: palette::OBY.l,
            a: 0.16,
        },
        info_foreground: palette::OBY,
        info_hover: Hsla {
            h: palette::OBY.h,
            s: palette::OBY.s,
            l: palette::OBY.l,
            a: 0.30,
        },
        info_active: Hsla {
            h: palette::OBY.h,
            s: palette::OBY.s,
            l: palette::OBY.l,
            a: 0.22,
        },
        success_foreground: palette::OK,
        success_hover: palette::OK_DIM,
        success_active: Hsla {
            h: palette::OK.h,
            s: palette::OK.s,
            l: palette::OK.l,
            a: 0.22,
        },
        warning_foreground: palette::WARN,
        warning_hover: palette::WARN_DIM,
        warning_active: Hsla {
            h: palette::WARN.h,
            s: palette::WARN.s,
            l: palette::WARN.l,
            a: 0.22,
        },
        danger_foreground: palette::ERR,
        danger_hover: palette::ERR_DIM,
        danger_active: Hsla {
            h: palette::ERR.h,
            s: palette::ERR.s,
            l: palette::ERR.l,
            a: 0.22,
        },
        overlay: Hsla {
            h: palette::GUERRA.h,
            s: palette::GUERRA.s,
            l: palette::GUERRA.l,
            a: 0.45,
        },
        title_bar: palette::dark::BG,
        title_bar_border: palette::dark::BORDER,
        table: palette::dark::BG,
        table_active: palette::dark::SURFACE_2,
        table_active_border: palette::OBY,
        table_even: palette::dark::SURFACE,
        table_hover: palette::dark::BORDER_2,
        table_head: palette::dark::SURFACE,
        table_head_foreground: palette::dark::TEXT,
        table_foot: palette::dark::SURFACE,
        table_foot_foreground: palette::dark::TEXT_3,
        table_row_border: palette::dark::BORDER,
        switch: palette::dark::SURFACE_2,
        switch_thumb: palette::dark::PRIMARY,
        slider_bar: palette::dark::SURFACE_2,
        slider_thumb: palette::dark::PRIMARY,
        progress_bar: palette::OBY,
        skeleton: palette::dark::SURFACE_2,
        tiles: palette::dark::SURFACE,
        ..*ThemeColor::dark().as_ref()
    }
}

fn jandi_light() -> ThemeColor {
    ThemeColor {
        background: palette::light::BG,
        foreground: palette::light::TEXT,
        muted_foreground: palette::light::TEXT_3,
        secondary: palette::light::SURFACE_2,
        secondary_foreground: palette::light::TEXT_2,
        muted: palette::light::SURFACE_2,
        border: palette::light::BORDER,
        primary: palette::light::PRIMARY,
        primary_foreground: palette::light::ON_PRIMARY,
        primary_hover: palette::light::PRIMARY_H,
        primary_active: palette::light::PRIMARY_H,
        button_primary: palette::light::PRIMARY,
        button_primary_hover: palette::light::PRIMARY_H,
        button_primary_active: palette::light::PRIMARY_H,
        button_primary_foreground: palette::light::ON_PRIMARY,
        success: palette::OK,
        warning: palette::WARN,
        danger: palette::ERR,
        sidebar: palette::light::SURFACE,
        sidebar_foreground: palette::light::TEXT,
        sidebar_border: palette::light::BORDER,
        tab_bar: palette::light::SURFACE,
        tab: palette::light::SURFACE,
        tab_active: palette::light::SURFACE_2,
        tab_active_foreground: palette::light::TEXT,
        tab_foreground: palette::light::TEXT_3,
        list: palette::light::BG,
        list_active: palette::light::SURFACE_2,
        list_active_border: palette::OBY,
        list_hover: palette::light::BORDER_2,
        list_even: palette::light::SURFACE,
        list_head: palette::light::SURFACE,
        accent: palette::light::SURFACE_2,
        accent_foreground: palette::light::TEXT,
        ring: Hsla {
            h: palette::OBY.h,
            s: palette::OBY.s,
            l: palette::OBY.l,
            a: 0.22,
        },
        scrollbar: palette::light::BORDER,
        scrollbar_thumb: palette::light::BORDER_2,
        scrollbar_thumb_hover: palette::OBY,
        input: palette::light::BORDER_2,
        caret: palette::light::TEXT,
        link: palette::OBY,
        link_hover: palette::JANDI,
        link_active: palette::GENIPINA,
        selection: Hsla {
            h: palette::OBY.h,
            s: palette::OBY.s,
            l: palette::OBY.l,
            a: 0.22,
        },
        popover: palette::light::SURFACE_3,
        popover_foreground: palette::light::TEXT,
        info: Hsla {
            h: palette::OBY.h,
            s: palette::OBY.s,
            l: palette::OBY.l,
            a: 0.12,
        },
        info_foreground: palette::OBY,
        info_hover: Hsla {
            h: palette::OBY.h,
            s: palette::OBY.s,
            l: palette::OBY.l,
            a: 0.24,
        },
        info_active: Hsla {
            h: palette::OBY.h,
            s: palette::OBY.s,
            l: palette::OBY.l,
            a: 0.16,
        },
        success_foreground: palette::OK,
        success_hover: palette::OK_DIM,
        success_active: Hsla {
            h: palette::OK.h,
            s: palette::OK.s,
            l: palette::OK.l,
            a: 0.22,
        },
        warning_foreground: palette::WARN,
        warning_hover: palette::WARN_DIM,
        warning_active: Hsla {
            h: palette::WARN.h,
            s: palette::WARN.s,
            l: palette::WARN.l,
            a: 0.22,
        },
        danger_foreground: palette::ERR,
        danger_hover: palette::ERR_DIM,
        danger_active: Hsla {
            h: palette::ERR.h,
            s: palette::ERR.s,
            l: palette::ERR.l,
            a: 0.22,
        },
        overlay: Hsla {
            h: palette::YANDI.h,
            s: palette::YANDI.s,
            l: palette::YANDI.l,
            a: 0.35,
        },
        title_bar: palette::light::BG,
        title_bar_border: palette::light::BORDER,
        table: palette::light::BG,
        table_active: palette::light::SURFACE_2,
        table_active_border: palette::OBY,
        table_even: palette::light::SURFACE,
        table_hover: palette::light::BORDER_2,
        table_head: palette::light::SURFACE,
        table_head_foreground: palette::light::TEXT,
        table_foot: palette::light::SURFACE,
        table_foot_foreground: palette::light::TEXT_3,
        table_row_border: palette::light::BORDER,
        switch: palette::light::SURFACE_2,
        switch_thumb: palette::light::PRIMARY,
        slider_bar: palette::light::SURFACE_2,
        slider_thumb: palette::light::PRIMARY,
        progress_bar: palette::OBY,
        skeleton: palette::light::SURFACE_2,
        tiles: palette::light::SURFACE,
        ..*ThemeColor::light().as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jandi_dark_background_is_guerra() {
        let t = jandi_dark();
        assert_eq!(t.background, palette::GUERRA);
        assert_eq!(t.foreground, palette::dark::TEXT);
    }

    #[test]
    fn test_jandi_light_background_is_edf1ee() {
        let t = jandi_light();
        assert_eq!(t.background, palette::light::BG);
        assert_eq!(t.foreground, palette::YANDI);
    }

    #[test]
    fn test_semantic_colors_present() {
        let t = jandi_dark();
        assert_eq!(t.success, palette::OK);
        assert_eq!(t.warning, palette::WARN);
        assert_eq!(t.danger, palette::ERR);
    }

    #[test]
    fn test_warn_not_err_in_theme() {
        let t = jandi_dark();
        assert_ne!(t.warning, t.danger);
    }

    #[test]
    fn test_jandi_ext_dark_light_differ() {
        let d = JandiExt::dark();
        let l = JandiExt::light();
        assert_ne!(d.code_bg, l.code_bg);
        assert_ne!(d.surface_3, l.surface_3);
        assert_ne!(d.surface, l.surface);
    }
}
