//! Foundations section — ramp, semantic states, surfaces, typography, density, radii, interaction.
//!
//! Renders the 7 sub-sections that mirror the `guidelines/` cards.
//! All colors flow from `cx.theme()`, theme extension (`JandiExt`), and `density` — zero literals.

use gpui::{FontWeight, IntoElement, ParentElement, SharedString, Styled, div};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{ActiveTheme as _, h_flex, v_flex};
use stand_in_mcp_explorer_ds::theme::colors::JandiExt;
use stand_in_mcp_explorer_ds::theme::typography;
use stand_in_mcp_explorer_ds::theme::{density, palette};

use super::util::{px, section_body, section_label};
use crate::shell::GalleryShell;

pub fn render_foundations(
    _state: &str,
    _mode: &str,
    _this: &GalleryShell,
    cx: &mut gpui::Context<GalleryShell>,
) -> gpui::AnyElement {
    let t = cx.theme().clone();
    let mono = t.mono_font_family.clone();
    let jx = cx.global::<JandiExt>().clone();

    v_flex()
        .flex_1()
        .min_w(px(0.))
        .h_full()
        .overflow_y_scrollbar()
        .child(
            section_body()
                .child(section_label("Ramp", &t, &mono))
                .child(render_ramp(&t, &mono))
                .child(section_label("Semantic States", &t, &mono))
                .child(render_semantic_states(&t, &mono))
                .child(section_label("Surfaces & Text", &t, &mono))
                .child(render_surfaces(&t, &jx, &mono))
                .child(section_label("Typography", &t, &mono))
                .child(render_typography(&t, &mono))
                .child(section_label("Density", &t, &mono))
                .child(render_density_demo(&t, &mono))
                .child(section_label("Radii", &t, &mono))
                .child(render_radii(&t, &mono))
                .child(section_label("Interaction", &t, &mono))
                .child(render_interaction(&t, &mono)),
        )
        .into_any_element()
}

fn swatch(
    label: &str,
    color: gpui::Hsla,
    t: &gpui_component::Theme,
    mono: &SharedString,
) -> impl IntoElement + use<> {
    v_flex()
        .w(px(80.))
        .gap_1()
        .child(
            div()
                .w(px(64.))
                .h(px(40.))
                .bg(color)
                .rounded(px(typography::FS_XS))
                .border_1()
                .border_color(t.border),
        )
        .child(
            div()
                .text_size(px(typography::FS_2XS))
                .text_color(t.muted_foreground)
                .font_family(mono.clone())
                .child(SharedString::from(label)),
        )
}

fn render_ramp(t: &gpui_component::Theme, mono: &SharedString) -> impl IntoElement + use<> {
    let ramp: &[(&str, gpui::Hsla)] = &[
        ("suco", palette::SUCO),
        ("brisa", palette::BRISA),
        ("oby", palette::OBY),
        ("jandi", palette::JANDI),
        ("genipina", palette::GENIPINA),
        ("nhandi", palette::NHANDI),
        ("yandi", palette::YANDI),
        ("guerra", palette::GUERRA),
    ];
    h_flex()
        .px(px(typography::FS_LG))
        .gap_3()
        .flex_wrap()
        .children(ramp.iter().map(|(name, c)| swatch(name, *c, t, mono)))
}

fn render_semantic_states(
    t: &gpui_component::Theme,
    mono: &SharedString,
) -> impl IntoElement + use<> {
    v_flex()
        .px(px(typography::FS_LG))
        .gap_2()
        .child(
            h_flex()
                .gap_3()
                .child(label_swatch("ok", palette::OK, t, mono))
                .child(label_swatch("ok-dim", palette::OK_DIM, t, mono)),
        )
        .child(
            h_flex()
                .gap_3()
                .child(label_swatch("warn", palette::WARN, t, mono))
                .child(label_swatch("warn-dim", palette::WARN_DIM, t, mono)),
        )
        .child(
            h_flex()
                .gap_3()
                .child(label_swatch("err", palette::ERR, t, mono))
                .child(label_swatch("err-dim", palette::ERR_DIM, t, mono)),
        )
        .child(
            div()
                .mt_2()
                .text_size(px(typography::FS_XS))
                .text_color(t.muted_foreground)
                .font_family(mono.clone())
                .child("warn \u{2260} err — distinct by hue + label"),
        )
}

fn label_swatch(
    label: &str,
    color: gpui::Hsla,
    t: &gpui_component::Theme,
    mono: &SharedString,
) -> impl IntoElement + use<> {
    h_flex()
        .gap_2()
        .items_center()
        .child(
            div()
                .w(px(32.))
                .h(px(24.))
                .bg(color)
                .rounded(px(typography::FS_XS))
                .border_1()
                .border_color(t.border),
        )
        .child(
            div()
                .text_size(px(typography::FS_2XS))
                .text_color(t.muted_foreground)
                .font_family(mono.clone())
                .child(SharedString::from(label)),
        )
}

fn surface_card(
    label: &str,
    bg: gpui::Hsla,
    text: gpui::Hsla,
    t: &gpui_component::Theme,
    mono: &SharedString,
) -> impl IntoElement + use<> {
    v_flex()
        .w(px(140.))
        .p_3()
        .gap_2()
        .bg(bg)
        .rounded(px(typography::FS_LG))
        .border_1()
        .border_color(t.border)
        .child(
            div()
                .text_size(px(typography::FS_2XS))
                .text_color(t.muted_foreground)
                .font_family(mono.clone())
                .child(SharedString::from(label)),
        )
        .child(
            div()
                .text_size(px(typography::FS_SM))
                .text_color(text)
                .child("The quick brown fox"),
        )
}

fn render_surfaces(
    t: &gpui_component::Theme,
    jx: &JandiExt,
    mono: &SharedString,
) -> impl IntoElement + use<> {
    h_flex()
        .px(px(typography::FS_LG))
        .gap_3()
        .flex_wrap()
        .child(surface_card("bg", t.background, t.foreground, t, mono))
        .child(surface_card("surface", t.secondary, t.foreground, t, mono))
        .child(surface_card(
            "surface-2",
            t.muted,
            t.secondary_foreground,
            t,
            mono,
        ))
        .child(surface_card(
            "surface-3",
            jx.surface_3,
            t.foreground,
            t,
            mono,
        ))
}

fn render_typography(t: &gpui_component::Theme, mono: &SharedString) -> impl IntoElement + use<> {
    let sizes: &[(&str, f32, FontWeight)] = &[
        ("fs-2xs 10.5", typography::FS_2XS, FontWeight::NORMAL),
        ("fs-xs 11", typography::FS_XS, FontWeight::NORMAL),
        ("fs-sm 12", typography::FS_SM, FontWeight::NORMAL),
        ("fs-md 13", typography::FS_MD, FontWeight::NORMAL),
        ("fs-lg 14", typography::FS_LG, FontWeight::NORMAL),
        ("fs-xl 15 bold", typography::FS_XL, FontWeight::BOLD),
        ("fs-title 20", typography::FS_TITLE, FontWeight::SEMIBOLD),
    ];
    v_flex()
        .px(px(typography::FS_LG))
        .gap_2()
        .child(
            h_flex()
                .gap_3()
                .child(
                    div()
                        .text_size(px(typography::FS_SM))
                        .text_color(t.muted_foreground)
                        .font_family(mono.clone())
                        .child("Sans (Hanken Grotesk)"),
                )
                .child(
                    div()
                        .text_size(px(typography::FS_SM))
                        .text_color(t.muted_foreground)
                        .font_family(mono.clone())
                        .child("Mono (JetBrains Mono)"),
                ),
        )
        .child(
            v_flex()
                .gap_1()
                .children(sizes.iter().map(|(label, size, weight)| {
                    let lbl = SharedString::from(*label);
                    h_flex()
                        .gap_3()
                        .child(
                            div()
                                .text_size(px(typography::FS_2XS))
                                .text_color(t.muted_foreground)
                                .font_family(mono.clone())
                                .w(px(100.))
                                .child(lbl),
                        )
                        .child(
                            div()
                                .text_size(px(*size))
                                .font_weight(*weight)
                                .child("Lorem ipsum"),
                        )
                        .child(
                            div()
                                .text_size(px(*size))
                                .font_weight(*weight)
                                .font_family(mono.clone())
                                .child("read_file"),
                        )
                })),
        )
}

fn render_density_demo(t: &gpui_component::Theme, mono: &SharedString) -> impl IntoElement + use<> {
    let levels = [
        density::Density::Compact,
        density::Density::Regular,
        density::Density::Comfy,
    ];
    let names = ["Compact", "Regular", "Comfy"];
    v_flex()
        .px(px(typography::FS_LG))
        .gap_2()
        .children(levels.iter().enumerate().map(|(i, d)| {
            let pad = d.pad();
            let row_h = d.row_h();
            let gap = d.gap();
            h_flex()
                .gap_2()
                .items_center()
                .child(
                    div()
                        .text_size(px(typography::FS_2XS))
                        .text_color(t.muted_foreground)
                        .font_family(mono.clone())
                        .w(px(64.))
                        .child(SharedString::from(names[i])),
                )
                .child(
                    div()
                        .px(px(pad))
                        .h(px(row_h))
                        .bg(t.sidebar)
                        .rounded(px(d.radius()))
                        .border_1()
                        .border_color(t.border)
                        .flex()
                        .items_center()
                        .child(
                            div()
                                .text_size(px(d.fs()))
                                .text_color(t.muted_foreground)
                                .font_family(mono.clone())
                                .child(format!(
                                    "pad={pad} row={row_h} gap={gap} fs={} r={}",
                                    d.fs(),
                                    d.radius()
                                )),
                        ),
                )
        }))
}

fn render_radii(t: &gpui_component::Theme, mono: &SharedString) -> impl IntoElement + use<> {
    let items: &[(&str, f32)] = &[
        ("badge 6", density::RADIUS_BADGE),
        ("chip 7", density::RADIUS_CHIP),
        ("input 8", density::RADIUS_INPUT),
        ("btn 9", density::RADIUS_BTN),
        ("card 10", density::RADIUS_CARD),
        ("pill 99", density::RADIUS_PILL),
    ];
    h_flex()
        .px(px(typography::FS_LG))
        .gap_3()
        .flex_wrap()
        .children(items.iter().map(|(label, r)| {
            let lbl = SharedString::from(*label);
            v_flex()
                .gap_1()
                .items_center()
                .child(
                    div()
                        .w(px(48.))
                        .h(px(48.))
                        .bg(t.primary)
                        .rounded(px(*r))
                        .border_1()
                        .border_color(t.border),
                )
                .child(
                    div()
                        .text_size(px(typography::FS_2XS))
                        .text_color(t.muted_foreground)
                        .font_family(mono.clone())
                        .child(lbl),
                )
        }))
}

fn render_interaction(t: &gpui_component::Theme, mono: &SharedString) -> impl IntoElement + use<> {
    v_flex()
        .px(px(typography::FS_LG))
        .gap_2()
        .child(
            h_flex()
                .gap_3()
                .items_center()
                .child(
                    div()
                        .text_size(px(typography::FS_2XS))
                        .text_color(t.muted_foreground)
                        .font_family(mono.clone())
                        .child("focus ring"),
                )
                .child(
                    div()
                        .px_3()
                        .py_2()
                        .bg(t.background)
                        .border_1()
                        .border_color(t.ring)
                        .rounded(px(typography::FS_XS))
                        .child(
                            div()
                                .text_size(px(typography::FS_SM))
                                .text_color(t.foreground)
                                .child("Focused input"),
                        ),
                ),
        )
        .child(
            div()
                .mt_1()
                .text_size(px(typography::FS_2XS))
                .text_color(t.muted_foreground)
                .font_family(mono.clone())
                .child(
                    "selection = inset 2px oby + surface-2 fill · hover = surface step, no shadow",
                ),
        )
}
