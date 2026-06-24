//! Resource detail column — header, metadata panel, subscription panel,
//! and content panel (rendered across `ResourceRead` lifecycle states).

use std::collections::HashSet;

use gpui::prelude::FluentBuilder;
use gpui::{
    Context, Entity, FontWeight, InteractiveElement, IntoElement, ParentElement, SharedString,
    Styled, Window, div, px,
};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{ActiveTheme as _, h_flex, v_flex};

use crate::app::i18n::Lang;
use crate::app::i18n::tr;
use stand_in_client::prelude::{Resource, ResourceTemplate};
use stand_in_mcp_explorer_ds::core::button::ClickHandler;
use stand_in_mcp_explorer_ds::core::button::{Button, ButtonSize, ButtonVariant};
use stand_in_mcp_explorer_ds::core::icon::{Icon, IconName, IconSize};
use stand_in_mcp_explorer_ds::core::spinner::Spinner;
use stand_in_mcp_explorer_ds::data::Panel;
use stand_in_mcp_explorer_ds::data::json_view::JsonView;
use stand_in_mcp_explorer_ds::forms::Field;
use stand_in_mcp_explorer_ds::theme::colors::JandiExt;

use super::ResourceRead;
use super::content::{ContentKind, classify_content, template_params};

/// Render the resource detail column (right side of the split layout).
#[allow(clippy::too_many_arguments)]
pub fn render_resource_detail<E: 'static>(
    selected_concrete: Option<&Resource>,
    selected_template: Option<&ResourceTemplate>,
    template_param_entities: &[(String, Entity<gpui_component::input::InputState>)],
    subscribed: &HashSet<String>,
    resource_read: &ResourceRead,
    lang: Lang,
    on_read: Option<ClickHandler>,
    on_subscribe: Option<ClickHandler>,
    on_unsubscribe: Option<ClickHandler>,
    _window: &mut Window,
    cx: &mut Context<E>,
) -> impl IntoElement {
    let selection_uri = selected_concrete.map(|r| r.uri.as_str());
    let is_subscribed = selection_uri.is_some_and(|uri| subscribed.contains(uri));

    // Bounded column (mirrors the list-col) + Dialog-pattern scroll body so the
    // whole detail scrolls (028 #18). Resources has no execute button, so there
    // is no fixed top bar — everything lives in the scroll body. The 22px gutter
    // (#17) is on the leaf.
    v_flex()
        .id("resource-detail-col")
        .flex_1()
        .min_w(px(0.))
        .h_full()
        .min_h(px(0.))
        .child(
            div().flex_1().min_h(px(0.)).overflow_hidden().child(
                // BLOCK div (not v_flex) so children block-fill the width even
                // when the scroll is not overflowing (028 #19).
                div()
                    .id("resource-detail-scroll")
                    .size_full()
                    .overflow_y_scrollbar()
                    .px(px(22.))
                    .child(render_detail_header(
                        selected_concrete,
                        selected_template,
                        lang,
                        cx,
                    ))
                    .child(render_metadata_panel(
                        selected_concrete,
                        selected_template,
                        resource_read,
                        lang,
                        cx,
                    ))
                    .child(render_template_params_panel(
                        selected_template,
                        template_param_entities,
                        lang,
                        cx,
                    ))
                    .child(render_subscription_panel(
                        selection_uri,
                        is_subscribed,
                        lang,
                        on_subscribe,
                        on_unsubscribe,
                        cx,
                    ))
                    .child(render_content_panel(resource_read, lang, on_read, cx)),
            ),
        )
}

// ---------------------------------------------------------------------------
// Detail header — doc icon + name (mono) + uri (mono, sub)
// ---------------------------------------------------------------------------

fn render_detail_header<E: 'static>(
    concrete: Option<&Resource>,
    template: Option<&ResourceTemplate>,
    lang: Lang,
    cx: &mut Context<E>,
) -> impl IntoElement {
    let mono = cx.theme().mono_font_family.clone();
    let muted = cx.theme().muted_foreground;

    match (concrete, template) {
        (Some(res), _) => v_flex()
            .id("resource-detail-header")
            .p_4()
            .gap_2()
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(Icon::new(IconName::File).size(IconSize::Sm)),
            )
            .child(
                div()
                    .text_sm()
                    .font_family(mono.clone())
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(SharedString::from(res.name.clone())),
            )
            .child(
                div()
                    .text_xs()
                    .font_family(mono)
                    .text_color(muted)
                    .child(SharedString::from(res.uri.clone())),
            ),
        (None, Some(tpl)) => v_flex()
            .id("resource-detail-header")
            .p_4()
            .gap_2()
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(Icon::new(IconName::File).size(IconSize::Sm)),
            )
            .child(
                div()
                    .text_sm()
                    .font_family(mono.clone())
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(SharedString::from(tpl.name.clone())),
            )
            .child(
                div()
                    .text_xs()
                    .font_family(mono)
                    .text_color(muted)
                    .child(SharedString::from(tpl.uri_template.clone())),
            ),
        (None, None) => v_flex()
            .id("resource-detail-empty")
            .p_4()
            .items_center()
            .justify_center()
            .child(
                div()
                    .text_sm()
                    .text_color(muted)
                    .child(SharedString::from(tr("resources.selectResource", lang))),
            ),
    }
}

// ---------------------------------------------------------------------------
// Metadata panel — grid of type, MIME, size
// ---------------------------------------------------------------------------

fn render_metadata_panel<E: 'static>(
    concrete: Option<&Resource>,
    template: Option<&ResourceTemplate>,
    resource_read: &ResourceRead,
    lang: Lang,
    cx: &mut Context<E>,
) -> impl IntoElement {
    let mono = cx.theme().mono_font_family.clone();
    let title = SharedString::from(tr("resources.metadata", lang));

    let mime = concrete
        .and_then(|r| r.mime_type.as_deref())
        .or_else(|| template.and_then(|t| t.mime_type.as_deref()))
        .unwrap_or("—");

    let kind = classify_read_label(resource_read);

    let size_str = match resource_read {
        ResourceRead::Content(result) => match classify_content(result) {
            Some(ContentKind::Binary { bytes }) => format!("{bytes} B"),
            _ => format_content_size(result),
        },
        _ => String::from("—"),
    };

    Panel::new()
        .id("panel-metadata")
        .title(title)
        .icon(IconName::Info)
        .children([h_flex()
            .gap_4()
            .child(meta_row(tr("resources.type", lang), kind, &mono))
            .child(meta_row(tr("resources.mimeType", lang), mime, &mono))
            .child(meta_row(tr("resources.size", lang), &size_str, &mono))
            .into_any_element()])
}

fn meta_row(label: &str, value: &str, mono_family: &SharedString) -> impl IntoElement {
    v_flex()
        .gap_1()
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .child(SharedString::from(label.to_string())),
        )
        .child(
            div()
                .text_sm()
                .font_family(mono_family.clone())
                .child(SharedString::from(value.to_string())),
        )
}

fn classify_read_label(read: &ResourceRead) -> &'static str {
    match read {
        ResourceRead::Content(result) => match classify_content(result) {
            Some(ContentKind::Binary { .. }) => "binary",
            _ => "text",
        },
        _ => "—",
    }
}

fn format_content_size(result: &stand_in_client::prelude::ReadResourceResult) -> String {
    let total: usize = result
        .contents
        .iter()
        .map(|c| match c {
            stand_in_client::prelude::ResourceContents::Text { text, .. } => text.len(),
            stand_in_client::prelude::ResourceContents::Blob { blob, .. } => blob.len(),
        })
        .sum();
    format!("{total} B")
}

// ---------------------------------------------------------------------------
// Template parameters panel — one Field per {param}
// ---------------------------------------------------------------------------

fn render_template_params_panel<E: 'static>(
    template: Option<&ResourceTemplate>,
    param_entities: &[(String, Entity<gpui_component::input::InputState>)],
    lang: Lang,
    _cx: &mut Context<E>,
) -> impl IntoElement {
    let Some(tpl) = template else {
        return gpui::div().into_any_element();
    };
    let params = template_params(&tpl.uri_template);
    if params.is_empty() {
        return gpui::div().into_any_element();
    }

    let title = SharedString::from(tr("tools.params", lang));

    Panel::new()
        .id("panel-template-params")
        .title(title)
        .icon(IconName::Bolt)
        .children([v_flex()
            .w_full()
            .gap_3()
            .children(params.iter().enumerate().map(|(i, param_name)| {
                let entity = param_entities
                    .iter()
                    .find(|(n, _)| n.as_str() == param_name);
                if let Some((_, entity)) = entity {
                    Field::new(entity)
                        .id(format!("tpl-param-{i}"))
                        .label(SharedString::from(param_name.clone()))
                        .long()
                        .mono(true)
                        .into_any_element()
                } else {
                    div().into_any_element()
                }
            }))
            .into_any_element()])
        .into_any_element()
}

// ---------------------------------------------------------------------------
// Subscription panel — subscribe / unsubscribe toggle
// ---------------------------------------------------------------------------

fn render_subscription_panel<E: 'static>(
    selection_uri: Option<&str>,
    is_subscribed: bool,
    lang: Lang,
    on_subscribe: Option<ClickHandler>,
    on_unsubscribe: Option<ClickHandler>,
    cx: &mut Context<E>,
) -> impl IntoElement {
    let Some(_uri) = selection_uri else {
        return gpui::div().into_any_element();
    };

    let title = SharedString::from(tr("resources.subscription", lang));

    let action_el: gpui::AnyElement = if is_subscribed {
        let mut btn = Button::new(tr("resources.cancel", lang))
            .variant(ButtonVariant::Danger)
            .size(ButtonSize::Sm);
        btn = btn.when_some(on_unsubscribe, |b, h| b.on_click(h));
        btn.into_any_element()
    } else {
        let mut btn = Button::new(tr("resources.subscribe", lang))
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Sm);
        btn = btn.when_some(on_subscribe, |b, h| b.on_click(h));
        btn.into_any_element()
    };

    let hint = SharedString::from(if is_subscribed {
        tr("resources.subOn", lang)
    } else {
        tr("resources.subOff", lang)
    });

    Panel::new()
        .id("panel-subscription")
        .title(title)
        .icon(IconName::Bell)
        .children([v_flex()
            .gap_2()
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(hint),
            )
            .child(action_el)
            .into_any_element()])
        .into_any_element()
}

// ---------------------------------------------------------------------------
// Content panel — renders across ResourceRead states
// ---------------------------------------------------------------------------

fn render_content_panel<E: 'static>(
    resource_read: &ResourceRead,
    lang: Lang,
    on_read: Option<ClickHandler>,
    cx: &mut Context<E>,
) -> impl IntoElement {
    let title = SharedString::from(tr("resources.content", lang));
    let mono = cx.theme().mono_font_family.clone();
    let j = cx.global::<JandiExt>();
    let colors = cx.theme().colors;

    let body = match resource_read {
        ResourceRead::Idle => v_flex()
            .gap_2()
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(SharedString::from(tr("resources.noContent", lang))),
            )
            .when_some(on_read, |col, handler| {
                col.child(
                    Button::new(tr("resources.read", lang))
                        .variant(ButtonVariant::Primary)
                        .size(ButtonSize::Sm)
                        .on_click(handler),
                )
            })
            .into_any_element(),
        ResourceRead::Loading => v_flex()
            .gap_2()
            .items_center()
            .child(Spinner::new())
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(SharedString::from(tr("resources.reading", lang))),
            )
            .into_any_element(),
        ResourceRead::Content(result) => match classify_content(result) {
            Some(ContentKind::Json { text }) => {
                JsonView::new(SharedString::from(text)).into_any_element()
            }
            Some(ContentKind::Text { text }) => div()
                .p_2()
                .rounded_md()
                .bg(j.code_bg)
                .font_family(mono.clone())
                .text_sm()
                .child(SharedString::from(text))
                .into_any_element(),
            Some(ContentKind::Binary { bytes }) => v_flex()
                .gap_2()
                .child(
                    div()
                        .text_sm()
                        .text_color(colors.warning)
                        .child(SharedString::from(tr("resources.binary", lang))),
                )
                .child(
                    div()
                        .text_xs()
                        .font_family(mono)
                        .text_color(cx.theme().muted_foreground)
                        .child(SharedString::from(format!("{bytes} bytes (base64)"))),
                )
                .into_any_element(),
            None => div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(SharedString::from(tr("resources.noContent", lang)))
                .into_any_element(),
        },
        ResourceRead::Error(msg) => v_flex()
            .gap_2()
            .child(
                div()
                    .text_sm()
                    .text_color(colors.danger)
                    .font_weight(FontWeight::MEDIUM)
                    .child(SharedString::from(tr("tools.errorProtocol", lang))),
            )
            .child(
                div()
                    .text_xs()
                    .font_family(mono)
                    .text_color(cx.theme().muted_foreground)
                    .child(SharedString::from(msg.clone())),
            )
            .into_any_element(),
    };

    Panel::new()
        .id("panel-content")
        .title(title)
        .icon(IconName::Doc)
        .children([body])
        .into_any_element()
}
