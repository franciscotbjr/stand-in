//! Raw jandi palette — the **only** file in the DS with color literals.
//!
//! Two sources, one canon:
//! - The **8-step named ramp** (`SUCO`…`GUERRA`) is derived directly from the
//!   published [`jandi-colors`](https://crates.io/crates/jandi-colors) crate —
//!   the same hexes the prototype's `tokens/colors.css` encodes. The crate ships
//!   8-bit `Rgb`; [`rgb8_to_hsla`] converts each to gpui `Hsla` in `const`
//!   context, so the ramp is no longer a hand-transcribed literal.
//! - The **theme tokens** that do not exist in the public palette (semantic
//!   states, surfaces, JSON syntax tokens, `BRAND_RING`) remain transcribed from
//!   `tokens/colors.css`, with OKLCH pre-converted to `Hsla` and verified.

use gpui::Hsla;
use jandi_colors::Rgb;

/// Convert an 8-bit `jandi_colors::Rgb` to a gpui `Hsla` at alpha `a`.
///
/// `const` so the ramp constants below stay compile-time values. Pure
/// arithmetic (no `powf`; `f32::max`/`min` aren't `const`, so the extremes are
/// picked by hand). The hue/sat/lightness are stored as gpui's 0..1 fractions.
/// This is exactly the hex→HSL the prototype encodes — same colors, computed.
pub const fn rgb8_to_hsla(rgb: Rgb, a: f32) -> Hsla {
    let r = rgb.r as f32 / 255.0;
    let g = rgb.g as f32 / 255.0;
    let b = rgb.b as f32 / 255.0;

    let max = if r >= g && r >= b {
        r
    } else if g >= b {
        g
    } else {
        b
    };
    let min = if r <= g && r <= b {
        r
    } else if g <= b {
        g
    } else {
        b
    };

    let l = (max + min) / 2.0;
    let delta = max - min;

    let (h, s) = if delta == 0.0 {
        (0.0, 0.0)
    } else {
        let s = if l > 0.5 {
            delta / (2.0 - max - min)
        } else {
            delta / (max + min)
        };
        let h_sector = if max == r {
            let hh = (g - b) / delta;
            if g < b { hh + 6.0 } else { hh }
        } else if max == g {
            (b - r) / delta + 2.0
        } else {
            (r - g) / delta + 4.0
        };
        (h_sector / 6.0, s)
    };

    Hsla { h, s, l, a }
}

// ---------------------------------------------------------------------------
// 8-step named ramp (suco = lightest, guerra = darkest), from `jandi-colors`.
// Hexes in comments are the canon values the crate carries.
// ---------------------------------------------------------------------------

pub const SUCO: Hsla = rgb8_to_hsla(jandi_colors::SUCO_VERDE.rgb, 1.0); // #C8D5CC
pub const BRISA: Hsla = rgb8_to_hsla(jandi_colors::BRISA.rgb, 1.0); // #8FA7B3
pub const OBY: Hsla = rgb8_to_hsla(jandi_colors::OBY.rgb, 1.0); // #5D7F96
pub const JANDI: Hsla = rgb8_to_hsla(jandi_colors::PRIMARY.rgb, 1.0); // #3D5F80
pub const GENIPINA: Hsla = rgb8_to_hsla(jandi_colors::GENIPINA.rgb, 1.0); // #2C4A6E
pub const NHANDI: Hsla = rgb8_to_hsla(jandi_colors::NHANDI.rgb, 1.0); // #1E3452
pub const YANDI: Hsla = rgb8_to_hsla(jandi_colors::YANDI.rgb, 1.0); // #11203A
pub const GUERRA: Hsla = rgb8_to_hsla(jandi_colors::TINTA_GUERRA.rgb, 1.0); // #0A1424

// ---------------------------------------------------------------------------
// Semantic states — OKLCH → pre-computed Hsla
//
// Conversion verified via oklch → linear sRGB → sRGB (gamma) → Hsla.
//   ok(0.74, 0.10, 168)   → #65BF9F
//   warn(0.80, 0.10, 80)  → #E0B771
//   err(0.68, 0.13, 25)   → #DD766F
// ---------------------------------------------------------------------------

pub const OK: Hsla = Hsla {
    h: 0.4417,
    s: 0.3182,
    l: 0.5725,
    a: 1.0,
};
pub const OK_DIM: Hsla = Hsla {
    h: 0.4417,
    s: 0.3182,
    l: 0.5725,
    a: 0.16,
};
pub const WARN: Hsla = Hsla {
    h: 0.1056,
    s: 0.4895,
    l: 0.6608,
    a: 1.0,
};
pub const WARN_DIM: Hsla = Hsla {
    h: 0.1056,
    s: 0.4895,
    l: 0.6608,
    a: 0.16,
};
pub const ERR: Hsla = Hsla {
    h: 0.0111,
    s: 0.468,
    l: 0.651,
    a: 1.0,
};
pub const ERR_DIM: Hsla = Hsla {
    h: 0.0111,
    s: 0.468,
    l: 0.651,
    a: 0.16,
};

// ---------------------------------------------------------------------------
// Canon-sourced constants (literal colours that live in the palette, not the
// theme, because they are fixed in both modes — cited from the prototype)
// ---------------------------------------------------------------------------

/// Inset ring on the brand-mark (1px) — canon:
/// `rgba(255, 255, 255, 0.08)` = white at 8% alpha.
/// Fixed in both dark and light modes.
pub const BRAND_RING: Hsla = Hsla {
    h: 0.,
    s: 0.,
    l: 1.,
    a: 0.08,
};

// ---------------------------------------------------------------------------
// Dark theme (default) — tokens/colors.css [data-theme="dark"]
// ---------------------------------------------------------------------------

pub mod dark {
    use super::*;

    pub const BG: Hsla = GUERRA;
    pub const SURFACE: Hsla = Hsla {
        h: 0.6029,
        s: 0.5484,
        l: 0.1216,
        a: 1.0,
    };
    pub const SURFACE_2: Hsla = NHANDI;
    pub const SURFACE_3: Hsla = Hsla {
        h: 0.5994,
        s: 0.4488,
        l: 0.249,
        a: 1.0,
    };
    // BRISA at low alpha — the canon defines the dark borders as the ramp color
    // with transparency, so derive them instead of re-transcribing the channels.
    pub const BORDER: Hsla = rgb8_to_hsla(jandi_colors::BRISA.rgb, 0.14);
    pub const BORDER_2: Hsla = rgb8_to_hsla(jandi_colors::BRISA.rgb, 0.26);
    pub const TEXT: Hsla = Hsla {
        h: 0.3889,
        s: 0.1475,
        l: 0.8804,
        a: 1.0,
    };
    pub const TEXT_2: Hsla = BRISA;
    #[allow(clippy::approx_constant)]
    pub const TEXT_3: Hsla = Hsla {
        h: 0.5667,
        s: 0.1852,
        l: 0.5235,
        a: 1.0,
    };
    pub const PRIMARY: Hsla = JANDI;
    pub const PRIMARY_H: Hsla = OBY;
    pub const ON_PRIMARY: Hsla = Hsla {
        h: 0.375,
        s: 0.1818,
        l: 0.9569,
        a: 1.0,
    };
    pub const CODE_BG: Hsla = Hsla {
        h: 0.6042,
        s: 0.6316,
        l: 0.0745,
        a: 1.0,
    };
    pub const SHADOW: Hsla = Hsla {
        h: 0.0,
        s: 0.0,
        l: 0.0,
        a: 0.8,
    };
}

// ---------------------------------------------------------------------------
// Light theme — tokens/colors.css [data-theme="light"]
// ---------------------------------------------------------------------------

pub mod light {
    use super::*;

    pub const BG: Hsla = Hsla {
        h: 0.375,
        s: 0.125,
        l: 0.9373,
        a: 1.0,
    };
    pub const SURFACE: Hsla = Hsla {
        h: 0.0,
        s: 0.0,
        l: 1.0,
        a: 1.0,
    };
    pub const SURFACE_2: Hsla = Hsla {
        h: 0.3889,
        s: 0.1429,
        l: 0.9588,
        a: 1.0,
    };
    pub const SURFACE_3: Hsla = Hsla {
        h: 0.381,
        s: 0.1489,
        l: 0.9078,
        a: 1.0,
    };
    pub const BORDER: Hsla = Hsla {
        h: 0.5923,
        s: 0.4194,
        l: 0.3039,
        a: 0.14,
    };
    pub const BORDER_2: Hsla = Hsla {
        h: 0.5923,
        s: 0.4194,
        l: 0.3039,
        a: 0.24,
    };
    pub const TEXT: Hsla = YANDI;
    pub const TEXT_2: Hsla = GENIPINA;
    // #56758A — oby escurecido p/ AA(4.5) sobre surface branca (O-006).
    // Próprio (não alias de OBY): OBY segue como acento/link/ring/dark-primary-hover.
    pub const TEXT_3: Hsla = Hsla {
        h: 0.5673,
        s: 0.2321,
        l: 0.4392,
        a: 1.0,
    };
    pub const PRIMARY: Hsla = JANDI;
    pub const PRIMARY_H: Hsla = GENIPINA;
    pub const ON_PRIMARY: Hsla = Hsla {
        h: 0.375,
        s: 0.1818,
        l: 0.9569,
        a: 1.0,
    };
    pub const CODE_BG: Hsla = Hsla {
        h: 0.3889,
        s: 0.1579,
        l: 0.9627,
        a: 1.0,
    };
    // Canon light shadow is YANDI (rgb 17,32,58) at 45% — derive from the ramp.
    pub const SHADOW: Hsla = rgb8_to_hsla(jandi_colors::YANDI.rgb, 0.45);
}

// ---------------------------------------------------------------------------
// JSON syntax tokens — tokens/colors.css :root
//
//   tok-str(0.78, 0.08, 150)  → #93C69D
//   tok-num(0.80, 0.09, 80)   → #DCB87A
//   tok-bool(0.74, 0.10, 25)  → #E3928B
// ---------------------------------------------------------------------------

pub const TOK_KEY: Hsla = BRISA;
pub const TOK_STR: Hsla = Hsla {
    h: 0.3667,
    s: 0.2435,
    l: 0.6765,
    a: 1.0,
};
pub const TOK_NUM: Hsla = Hsla {
    h: 0.1056,
    s: 0.4486,
    l: 0.6706,
    a: 1.0,
};
pub const TOK_BOOL: Hsla = Hsla {
    h: 0.0139,
    s: 0.4815,
    l: 0.7176,
    a: 1.0,
};
// tok-punc is text-3 per mode — not a const here

// ---------------------------------------------------------------------------
// Contrast safety helpers — verify a foreground/background pair in test/audit
// ---------------------------------------------------------------------------

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    if s == 0.0 {
        return (l, l, l);
    }
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);
    (r, g, b)
}

fn hue_to_rgb(p: f32, q: f32, t: f32) -> f32 {
    let t = if t < 0.0 {
        t + 1.0
    } else if t > 1.0 {
        t - 1.0
    } else {
        t
    };
    if t < 1.0 / 6.0 {
        p + (q - p) * 6.0 * t
    } else if t < 1.0 / 2.0 {
        q
    } else if t < 2.0 / 3.0 {
        p + (q - p) * (2.0 / 3.0 - t) * 6.0
    } else {
        p
    }
}

fn linearize_srgb(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Relative luminance (WCAG 2.1 §relative-luminance) from an `Hsla`.
/// Converts HSL → sRGB → linear RGB → luminance.
pub fn relative_luminance(c: Hsla) -> f32 {
    let (r, g, b) = hsl_to_rgb(c.h, c.s, c.l);
    0.2126 * linearize_srgb(r) + 0.7152 * linearize_srgb(g) + 0.0722 * linearize_srgb(b)
}

/// Contrast ratio between two relative luminances (WCAG 2.1).
pub fn contrast_ratio(l1: f32, l2: f32) -> f32 {
    let lighter = l1.max(l2);
    let darker = l1.min(l2);
    (lighter + 0.05) / (darker + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warn_not_equal_err() {
        assert_ne!(WARN, ERR);
    }

    #[test]
    fn test_ok_warn_err_distinct() {
        assert_ne!(OK, WARN);
        assert_ne!(WARN, ERR);
        assert_ne!(OK, ERR);
    }

    #[test]
    fn test_dim_is_same_hue_16pct_alpha() {
        assert_eq!(OK_DIM.h, OK.h);
        assert_eq!(OK_DIM.s, OK.s);
        assert_eq!(OK_DIM.l, OK.l);
        assert_eq!(OK_DIM.a, 0.16);

        assert_eq!(WARN_DIM.h, WARN.h);
        assert_eq!(WARN_DIM.a, 0.16);

        assert_eq!(ERR_DIM.h, ERR.h);
        assert_eq!(ERR_DIM.a, 0.16);
    }

    #[test]
    fn test_dark_mode_bg_is_guerra() {
        assert_eq!(dark::BG, GUERRA);
    }

    #[test]
    fn test_light_mode_bg_not_dark() {
        assert_ne!(light::BG, dark::BG);
    }

    #[test]
    fn test_relative_luminance_white_is_one() {
        let white = Hsla {
            h: 0.,
            s: 0.,
            l: 1.,
            a: 1.,
        };
        assert!((relative_luminance(white) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_relative_luminance_black_is_zero() {
        let black = Hsla {
            h: 0.,
            s: 0.,
            l: 0.,
            a: 1.,
        };
        assert!((relative_luminance(black) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_contrast_ratio_white_on_black_is_21() {
        let white = Hsla {
            h: 0.,
            s: 0.,
            l: 1.,
            a: 1.,
        };
        let black = Hsla {
            h: 0.,
            s: 0.,
            l: 0.,
            a: 1.,
        };
        let l1 = relative_luminance(white);
        let l2 = relative_luminance(black);
        assert!((contrast_ratio(l1, l2) - 21.0).abs() < 0.05);
    }

    #[test]
    fn test_text3_on_surface_dark_contrast() {
        let l_text = relative_luminance(dark::TEXT_3);
        let l_surf = relative_luminance(dark::SURFACE);
        let ratio = contrast_ratio(l_text, l_surf);
        assert!(ratio >= 4.5, "dark text-3/surface ratio {ratio} < 4.5 (AA)");
    }

    #[test]
    fn test_text3_on_surface_light_contrast() {
        let l_text = relative_luminance(light::TEXT_3);
        let l_surf = relative_luminance(light::SURFACE);
        let ratio = contrast_ratio(l_text, l_surf);
        // light text-3 (#56758A, oby escurecido) sobre surface (#FFFFFF): AA (>= 4.5) — O-006.
        assert!(
            ratio >= 4.5,
            "light text-3/surface ratio {ratio} < 4.5 (AA)"
        );
    }

    #[test]
    fn test_light_text3_not_oby_alias() {
        // text-3 was decoupled from OBY (it was an alias) — O-006.
        assert_ne!(light::TEXT_3, OBY);
    }

    #[test]
    fn test_light_text3_hierarchy() {
        // 3-level hierarchy: text (darkest) < text-2 < text-3 (lightest).
        let l1 = relative_luminance(light::TEXT);
        let l2 = relative_luminance(light::TEXT_2);
        let l3 = relative_luminance(light::TEXT_3);
        assert!(l1 < l2, "text ({l1}) is not darker than text-2 ({l2})");
        assert!(l2 < l3, "text-2 ({l2}) is not darker than text-3 ({l3})");
    }

    // The ramp now comes from the `jandi-colors` crate. These guard against a
    // `cargo update` silently shifting the canon out from under the DS.
    #[test]
    fn test_jandi_colors_hexes_match_canon() {
        assert_eq!(jandi_colors::SUCO_VERDE.hex, "#C8D5CC");
        assert_eq!(jandi_colors::BRISA.hex, "#8FA7B3");
        assert_eq!(jandi_colors::OBY.hex, "#5D7F96");
        assert_eq!(jandi_colors::PRIMARY.hex, "#3D5F80");
        assert_eq!(jandi_colors::GENIPINA.hex, "#2C4A6E");
        assert_eq!(jandi_colors::NHANDI.hex, "#1E3452");
        assert_eq!(jandi_colors::YANDI.hex, "#11203A");
        assert_eq!(jandi_colors::TINTA_GUERRA.hex, "#0A1424");
    }

    #[test]
    fn test_rgb8_to_hsla_sanity() {
        let white = rgb8_to_hsla(
            Rgb {
                r: 255,
                g: 255,
                b: 255,
            },
            1.0,
        );
        assert_eq!(white.l, 1.0);
        assert_eq!(white.s, 0.0);

        let black = rgb8_to_hsla(Rgb { r: 0, g: 0, b: 0 }, 1.0);
        assert_eq!(black.l, 0.0);
        assert_eq!(black.s, 0.0);

        let gray = rgb8_to_hsla(
            Rgb {
                r: 128,
                g: 128,
                b: 128,
            },
            0.5,
        );
        assert_eq!(gray.s, 0.0);
        assert_eq!(gray.a, 0.5);

        // JANDI (#3D5F80) lands on the documented hsl fraction.
        assert!((JANDI.h - 0.5821).abs() < 1e-3);
        assert!((JANDI.s - 0.3545).abs() < 1e-3);
        assert!((JANDI.l - 0.3706).abs() < 1e-3);
    }
}
