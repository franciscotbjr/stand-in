//! Privacy badge — persistent footer callout reassuring the user
//! that the app is local-first with no registration or cloud dependency.

use crate::app::i18n::Lang;
use crate::app::i18n::tr;
use gpui::{
    App, FontWeight, InteractiveElement, IntoElement, ParentElement, Styled, Window, div, px,
};
use gpui_component::{ActiveTheme as _, h_flex, v_flex};
use stand_in_mcp_explorer_ds::core::icon::{Icon, IconName};
use stand_in_mcp_explorer_ds::theme::colors::JandiExt;
use stand_in_mcp_explorer_ds::theme::density::RADIUS_BADGE;
use stand_in_mcp_explorer_ds::theme::typography;

pub fn render_privacy_badge(lang: Lang, _window: &mut Window, cx: &mut App) -> impl IntoElement {
    let t = cx.theme().clone();
    let ext = cx.global::<JandiExt>();

    h_flex()
        .id("privacy-badge")
        .w_full()
        .p(px(11.))
        .gap(px(9.))
        .items_center()
        .bg(ext.ok_dim)
        .rounded(px(RADIUS_BADGE))
        .child(
            Icon::new(IconName::Lock)
                .with_px(px(14.))
                .color(t.colors.success_foreground),
        )
        .child(
            v_flex().gap(px(2.)).child(
                div()
                    .id("privacy-title")
                    .text_size(px(typography::FS_SM))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(t.colors.success_foreground)
                    .child(tr("sidebar.privacyTitle", lang)),
            ),
        )
}
