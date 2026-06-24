//! Settings panel — overlay with scrim + elevated card (BUG-3).
//!
//! Renders on top of the app when `settings_open` is true. The scrim is the
//! full-window translucent backdrop that **occludes** the app behind (it does
//! NOT close on click). The card is centered, opens with a header (title +
//! close X), and carries the **only legitimate shadow** in the app (DS-flat:
//! shadow-only-on-overlays). Closing is the header X only.
//!
//! Controls — theme, density, primary colour, guided — apply live and
//! persist via `AppSettings`.

use gpui::{
    App, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement, Styled,
    Window, div, px,
};
use gpui_component::ActiveTheme as _;

use crate::app::i18n::{Lang, tr};
use crate::app::settings::{AppSettings, DensityChoice, PrimaryChoice, ThemeChoice};
use stand_in_mcp_explorer_ds::core::button::{Button, ButtonVariant, ClickHandler};
use stand_in_mcp_explorer_ds::core::icon::{Icon, IconName};
use stand_in_mcp_explorer_ds::core::icon_button::IconButton;
use stand_in_mcp_explorer_ds::forms::segmented_control::SegmentedControl;
use stand_in_mcp_explorer_ds::navigation::section_label::SectionLabel;
use stand_in_mcp_explorer_ds::theme::colors::JandiExt;
use stand_in_mcp_explorer_ds::theme::density::RADIUS_CARD;
use stand_in_mcp_explorer_ds::theme::palette::{GENIPINA, JANDI, OBY};

// ---------------------------------------------------------------------------
// Public render entry
// ---------------------------------------------------------------------------

/// Render the full-screen settings overlay.
///
/// The overlay consists of a scrim (occludes the background, does not close on
/// click) + a centered card with a header (title + close X) and `shadow_lg`
/// (BUG-3 — the only shadow in the app).
///
/// Handlers:
/// - `theme_handlers`: 2 handlers [Dark, Light] for the theme segmented control.
/// - `density_handlers`: 3 handlers [Compact, Regular, Comfy] for density.
/// - `primary_handlers`: 3 handlers [Jandi, Genipina, Oby] for the swatches.
/// - `on_guided_toggle`: toggles guided mode.
/// - `on_close`: closes the overlay (wired to the header X button).
#[allow(clippy::too_many_arguments)]
pub fn render_settings(
    settings: &AppSettings,
    lang: Lang,
    theme_handlers: Vec<ClickHandler>,
    density_handlers: Vec<ClickHandler>,
    primary_handlers: Vec<ClickHandler>,
    on_guided_toggle: ClickHandler,
    on_close: ClickHandler,
    _window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    // Extract theme data before mutable borrow by settings_body.
    let colors = cx.theme().colors;
    let ext = cx.global::<JandiExt>().clone();
    let font = cx.theme().font_family.clone();

    let theme_ix = match settings.theme {
        ThemeChoice::Dark => 0,
        ThemeChoice::Light => 1,
    };
    let density_ix = match settings.density {
        DensityChoice::Compact => 0,
        DensityChoice::Regular => 1,
        DensityChoice::Comfy => 2,
    };

    // Scrim — translucent backdrop. `occlude()` blocks interaction with the app
    // behind; it does NOT close on click. Closing is the X in the header only —
    // clicking a control inside no longer dismisses the modal (028 Item #10).
    div()
        .id("settings-overlay")
        .absolute()
        .inset_0()
        .bg(ext.shadow_overlay)
        .occlude()
        .flex()
        .items_center()
        .justify_center()
        // Card — elevated, the only shadow in the app (BUG-3 / DS-flat).
        .child(
            div()
                .id("settings-card")
                .flex()
                .flex_col()
                .bg(colors.secondary)
                .border_1()
                .border_color(colors.border)
                .rounded(px(RADIUS_CARD))
                .p(px(18.))
                .shadow_lg()
                .min_w(px(360.))
                .max_w(px(460.))
                // Header: title + close (X). Close = the X button only.
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .mb(px(14.))
                        .child(
                            div()
                                .text_size(px(15.))
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_color(colors.foreground)
                                .child(tr("tweaks.title", lang)),
                        )
                        .child(
                            IconButton::new(IconName::X, tr("tweaks.close", lang))
                                .id("settings-close")
                                .on_click_boxed(on_close),
                        ),
                )
                .child(settings_body(
                    settings,
                    lang,
                    theme_ix,
                    density_ix,
                    theme_handlers,
                    density_handlers,
                    primary_handlers,
                    on_guided_toggle,
                    font,
                    &colors,
                    &ext,
                    cx,
                )),
        )
}

// ---------------------------------------------------------------------------
// Card body
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn settings_body(
    settings: &AppSettings,
    lang: Lang,
    theme_ix: usize,
    density_ix: usize,
    theme_handlers: Vec<ClickHandler>,
    density_handlers: Vec<ClickHandler>,
    primary_handlers: Vec<ClickHandler>,
    on_guided_toggle: ClickHandler,
    font: gpui::SharedString,
    colors: &gpui_component::ThemeColor,
    ext: &JandiExt,
    _cx: &mut App,
) -> impl IntoElement {
    // --- Section: Appearance ---
    let appearance_label =
        SectionLabel::new(tr("tweaks.appearance", lang)).id("settings-section-appearance");

    let theme_seg = SegmentedControl::new(
        "settings-theme",
        vec![
            ("dark", tr("tweaks.themeDark", lang)),
            ("light", tr("tweaks.themeLight", lang)),
        ],
        theme_ix,
    )
    .handlers(theme_handlers);

    let density_seg = SegmentedControl::new(
        "settings-density",
        vec![
            ("compact", tr("tweaks.densityCompact", lang)),
            ("regular", tr("tweaks.densityRegular", lang)),
            ("comfy", tr("tweaks.densityComfy", lang)),
        ],
        density_ix,
    )
    .handlers(density_handlers);

    let primary_label_row = gpui::div().mt(px(14.)).child(
        gpui::div()
            .text_size(px(12.))
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(colors.secondary_foreground)
            .child(tr("tweaks.primaryColor", lang)),
    );

    let swatches = primary_swatches(settings.primary, primary_handlers, colors, ext);

    // --- Section: Experience ---
    let experience_label =
        SectionLabel::new(tr("tweaks.experience", lang)).id("settings-section-experience");

    let guided_row = guided_toggle_row(settings.guided, lang, on_guided_toggle, font, colors);

    div()
        .flex()
        .flex_col()
        .gap(px(16.))
        .child(appearance_label)
        .child(theme_seg)
        .child(density_seg)
        .child(primary_label_row)
        .child(swatches)
        .child(experience_label)
        .child(guided_row)
}

// ---------------------------------------------------------------------------
// Primary colour swatches
// ---------------------------------------------------------------------------

fn primary_swatches(
    selected: PrimaryChoice,
    handlers: Vec<ClickHandler>,
    colors: &gpui_component::ThemeColor,
    ext: &JandiExt,
) -> impl IntoElement {
    let choices = [
        (PrimaryChoice::Jandi, JANDI),
        (PrimaryChoice::Genipina, GENIPINA),
        (PrimaryChoice::Oby, OBY),
    ];

    let mut h_iter = handlers.into_iter();
    let mut row = gpui::div()
        .flex()
        .flex_row()
        .w_full()
        .gap(px(10.))
        .mt(px(4.));

    for (i, (choice, hsla)) in choices.iter().enumerate() {
        let is_selected = selected == *choice;
        let on_click = h_iter.next();

        // Full-width rectangles (flex_1) filling the row, matching the
        // prototype's TweakColor chips (028 Item #10) — not small squares.
        let mut square = div()
            .id(format!("settings-primary-{i}"))
            .flex_1()
            .h(px(44.))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(8.))
            .bg(*hsla)
            .border_1()
            .border_color(if is_selected {
                colors.foreground
            } else {
                colors.border
            })
            .cursor_pointer()
            .hover(|h| h.border_color(ext.border_2));

        if let Some(handler) = on_click {
            square = square.on_click(handler);
        }

        if is_selected {
            square = square.child(Icon::new(IconName::Check).with_px(px(14.)).color(
                if hsla.l > 0.55 {
                    colors.foreground
                } else {
                    colors.primary_foreground
                },
            ));
        }

        row = row.child(square);
    }

    row
}

// ---------------------------------------------------------------------------
// Guided toggle row
// ---------------------------------------------------------------------------

fn guided_toggle_row(
    guided: bool,
    lang: Lang,
    on_click: ClickHandler,
    _font: gpui::SharedString,
    colors: &gpui_component::ThemeColor,
) -> impl IntoElement {
    let label = tr("tweaks.guided", lang);

    let btn = Button::new(tr("topbar.guidedMode", lang))
        .id("settings-guided-toggle")
        .variant(if guided {
            ButtonVariant::Primary
        } else {
            ButtonVariant::Ghost
        })
        .on_click(on_click);

    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .child(
            gpui::div()
                .text_size(px(12.))
                .text_color(colors.secondary_foreground)
                .child(label),
        )
        .child(btn)
}
