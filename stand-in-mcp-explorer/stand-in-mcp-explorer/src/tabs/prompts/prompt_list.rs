//! Prompt list column — `ListSearch` (sticky) + windowed `ListItem` rows via
//! `gpui::uniform_list` (031/M3). Only the visible subset of items is
//! rendered; memory is limited by the viewport, not the total item count.

use std::sync::Arc;

use gpui::{
    AnyElement, Context, Entity, InteractiveElement, IntoElement, ParentElement, SharedString,
    Styled, UniformListScrollHandle, Window, div, px, uniform_list,
};
use gpui_component::{ActiveTheme as _, v_flex};

use crate::app::i18n::Lang;
use crate::app::i18n::tr;
use stand_in_client::prelude::PromptDefinition;
use stand_in_mcp_explorer_ds::core::badge::{Badge, BadgeKind};
use stand_in_mcp_explorer_ds::data::{ListItem, ListSearch};
use stand_in_mcp_explorer_ds::theme::colors::JandiExt;

use super::PromptSelectFn;

/// Render the prompt list (left column of the split layout).
#[allow(clippy::too_many_arguments)]
pub fn render_prompt_list<E: 'static>(
    item_count: usize,
    filtered_prompts: Arc<[PromptDefinition]>,
    selected_prompt: Option<&str>,
    filter_input: &Entity<gpui_component::input::InputState>,
    lang: Lang,
    _capture_state: Option<&str>,
    on_select: PromptSelectFn,
    prompts_scroll: UniformListScrollHandle,
    _window: &mut Window,
    cx: &mut Context<E>,
) -> impl IntoElement {
    let surface = cx.global::<JandiExt>().surface;
    let border = cx.theme().border;

    let body: AnyElement = if item_count == 0 {
        v_flex()
            .id("prompt-list-empty")
            .p_4()
            .items_center()
            .justify_center()
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(SharedString::from(tr("prompts.filter", lang))),
            )
            .into_any_element()
    } else {
        let selected = selected_prompt.map(String::from);
        uniform_list(
            "prompt-uniform-list",
            item_count,
            move |range, _window, _app| {
                let end = range.end.min(filtered_prompts.len());
                (range.start..end)
                    .filter_map(|abs_ix| {
                        filtered_prompts.get(abs_ix).map(|prompt| {
                            let name = prompt.name.clone();
                            let desc = prompt.description.clone();
                            let is_sel = selected.as_deref() == Some(name.as_str());
                            let args_count =
                                prompt.arguments.as_ref().map(|a| a.len()).unwrap_or(0);
                            let prompt = (*prompt).clone();
                            let on_select = on_select.clone();

                            let badge: Option<AnyElement> = (args_count > 0).then(|| {
                                let label = crate::app::i18n::tr_args(
                                    "prompts.argsCount",
                                    lang,
                                    &[("n", &args_count.to_string())],
                                );
                                Badge::new(label, BadgeKind::Muted).into_any_element()
                            });

                            let mut item = ListItem::new(
                                format!("prompt-{abs_ix}"),
                                SharedString::from(name.clone()),
                            )
                            .desc(SharedString::from(desc))
                            .selected(is_sel);

                            if let Some(b) = badge {
                                item = item.badge(b);
                            }

                            item.on_click(
                                move |_: &gpui::ClickEvent,
                                      window: &mut Window,
                                      cx: &mut gpui::App| {
                                    on_select(&prompt, window, cx);
                                },
                            )
                        })
                    })
                    .collect::<Vec<_>>()
            },
        )
        .flex_1()
        .w_full()
        .track_scroll(&prompts_scroll)
        .into_any_element()
    };

    v_flex()
        .id("prompt-list-col")
        .w(px(260.))
        .min_w(px(260.))
        .flex_none()
        .h_full()
        .min_h(px(0.))
        .bg(surface)
        .border_r_1()
        .border_color(border)
        .child(ListSearch::new(filter_input).id("prompts-search"))
        .child(body)
}
