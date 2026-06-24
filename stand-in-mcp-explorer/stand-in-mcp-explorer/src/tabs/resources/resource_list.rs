//! Resource list column — `ListSearch` (sticky) + windowed rows via
//! `gpui::uniform_list` (031/M3). Concrete resources and templates are
//! flattened into one `uniform_list`; only the visible subset is rendered.

use std::collections::HashSet;
use std::sync::Arc;

use gpui::{
    AnyElement, Context, Entity, InteractiveElement, IntoElement, ParentElement, SharedString,
    Styled, UniformListScrollHandle, Window, div, px, uniform_list,
};
use gpui_component::{ActiveTheme as _, v_flex};

use crate::app::i18n::Lang;
use crate::app::i18n::tr;
use stand_in_client::prelude::{Resource, ResourceTemplate};
use stand_in_mcp_explorer_ds::core::badge::{Badge, BadgeKind};
use stand_in_mcp_explorer_ds::data::{ListItem, ListSearch};
use stand_in_mcp_explorer_ds::theme::colors::JandiExt;

/// Render the resource list (left column of the split layout).
///
/// Concretes and templates are flattened into one `uniform_list`: concrete
/// resources occupy indices `0..n_concretes`, templates occupy the rest.
#[allow(clippy::too_many_arguments)]
pub fn render_resource_list<E: 'static>(
    item_count: usize,
    n_concretes: usize,
    filtered_concretes: Arc<[Resource]>,
    filtered_templates: Arc<[ResourceTemplate]>,
    selected_uri: Option<&str>,
    subscribed: &HashSet<String>,
    filter_input: &Entity<gpui_component::input::InputState>,
    lang: Lang,
    on_select_concrete: super::ResourceSelectFn,
    on_select_template: super::TemplateSelectFn,
    resources_scroll: UniformListScrollHandle,
    _window: &mut Window,
    cx: &mut Context<E>,
) -> impl IntoElement {
    let surface = cx.global::<JandiExt>().surface;
    let border = cx.theme().border;
    let success_color = cx.theme().colors.success;

    let body: AnyElement = if item_count == 0 {
        v_flex()
            .id("resource-list-empty")
            .p_4()
            .items_center()
            .justify_center()
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(SharedString::from(tr("resources.filter", lang))),
            )
            .into_any_element()
    } else {
        let selected = selected_uri.map(String::from);
        let sub: HashSet<String> = subscribed.clone();
        uniform_list(
            "resource-uniform-list",
            item_count,
            move |range, _window, _app| {
                let end = range.end.min(item_count);
                (range.start..end)
                    .filter_map(|abs_ix| {
                        if abs_ix < n_concretes {
                            filtered_concretes.get(abs_ix).map(|res| {
                                let uri = res.uri.clone();
                                let name = res.name.clone();
                                let desc = res.description.clone();
                                let is_sel = selected.as_deref() == Some(uri.as_str());
                                let is_sub = sub.contains(&uri);
                                let res = (*res).clone();
                                let on_select = on_select_concrete.clone();

                                let badge: Option<AnyElement> = res
                                    .mime_type
                                    .as_deref()
                                    .map(|m| Badge::new(m, BadgeKind::Mime).into_any_element());

                                let sub_dot: Option<AnyElement> = is_sub.then(|| {
                                    div()
                                        .ml_1()
                                        .child(
                                            div()
                                                .w(px(8.))
                                                .h(px(8.))
                                                .rounded_full()
                                                .bg(success_color),
                                        )
                                        .into_any_element()
                                });

                                let badge_elem = match (badge, sub_dot) {
                                    (Some(b), Some(d)) => Some(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_1()
                                            .child(b)
                                            .child(d)
                                            .into_any_element(),
                                    ),
                                    (Some(b), None) => Some(b),
                                    (None, Some(d)) => Some(d),
                                    (None, None) => None,
                                };

                                let mut item = ListItem::new(
                                    format!("res-{abs_ix}"),
                                    SharedString::from(name.clone()),
                                )
                                .desc(desc.map(SharedString::from).unwrap_or_default())
                                .selected(is_sel);

                                if let Some(e) = badge_elem {
                                    item = item.badge(e);
                                }

                                item.on_click(
                                    move |_: &gpui::ClickEvent,
                                          window: &mut Window,
                                          cx: &mut gpui::App| {
                                        on_select(&res, window, cx);
                                    },
                                )
                            })
                        } else {
                            let tpl_ix = abs_ix - n_concretes;
                            filtered_templates.get(tpl_ix).map(|tpl| {
                                let uri = tpl.uri_template.clone();
                                let name = tpl.name.clone();
                                let desc = tpl.description.clone();
                                let is_sel = selected.as_deref() == Some(uri.as_str());
                                let mime_badge = tpl
                                    .mime_type
                                    .as_deref()
                                    .map(|m| Badge::new(m, BadgeKind::Mime).into_any_element());
                                let tpl_badge =
                                    Badge::new("template", BadgeKind::Role).into_any_element();
                                let tpl = (*tpl).clone();
                                let on_select = on_select_template.clone();

                                let badge_elem: Option<AnyElement> = match mime_badge {
                                    Some(m) => Some(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_1()
                                            .child(tpl_badge)
                                            .child(m)
                                            .into_any_element(),
                                    ),
                                    None => Some(tpl_badge),
                                };

                                let mut item = ListItem::new(
                                    format!("tpl-{tpl_ix}"),
                                    SharedString::from(name.clone()),
                                )
                                .desc(desc.map(SharedString::from).unwrap_or_default())
                                .selected(is_sel);

                                if let Some(e) = badge_elem {
                                    item = item.badge(e);
                                }

                                item.on_click(
                                    move |_: &gpui::ClickEvent,
                                          window: &mut Window,
                                          cx: &mut gpui::App| {
                                        on_select(&tpl, window, cx);
                                    },
                                )
                            })
                        }
                    })
                    .collect::<Vec<_>>()
            },
        )
        .flex_1()
        .w_full()
        .track_scroll(&resources_scroll)
        .into_any_element()
    };

    v_flex()
        .id("resource-list-col")
        .w(px(260.))
        .min_w(px(260.))
        .flex_none()
        .h_full()
        .min_h(px(0.))
        .bg(surface)
        .border_r_1()
        .border_color(border)
        .child(ListSearch::new(filter_input).id("resources-search"))
        .child(body)
}
