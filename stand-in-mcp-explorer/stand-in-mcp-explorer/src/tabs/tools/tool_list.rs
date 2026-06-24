//! Tool list column — `ListSearch` (sticky) + windowed `ListItem` rows via
//! `gpui::uniform_list` (031/M2). Only the visible subset of items is
//! rendered; memory is limited by the viewport, not the total item count.

use std::sync::Arc;

use gpui::{
    AnyElement, Context, Entity, InteractiveElement, IntoElement, ParentElement, SharedString,
    Styled, UniformListScrollHandle, Window, div, px, uniform_list,
};
use gpui_component::{ActiveTheme as _, v_flex};

use crate::app::i18n::Lang;
use crate::app::i18n::tr;
use stand_in_client::prelude::ToolDefinition;
use stand_in_mcp_explorer_ds::data::{ListItem, ListSearch};
use stand_in_mcp_explorer_ds::theme::colors::JandiExt;

use super::ToolSelectFn;

/// Render the tool list column — sticky `ListSearch` + windowed `uniform_list`.
///
#[allow(clippy::too_many_arguments)]
pub fn render_tool_list<E: 'static>(
    item_count: usize,
    filtered_tools: Arc<[ToolDefinition]>,
    selected_tool: Option<String>,
    tool_filter_input: &Entity<gpui_component::input::InputState>,
    lang: Lang,
    _capture_state: Option<&str>,
    on_select: ToolSelectFn,
    tools_scroll: UniformListScrollHandle,
    _window: &mut Window,
    cx: &mut Context<E>,
) -> impl IntoElement {
    let surface = cx.global::<JandiExt>().surface;
    let border = cx.theme().border;

    let body: AnyElement = if item_count == 0 {
        v_flex()
            .id("tool-list-empty")
            .p_4()
            .items_center()
            .justify_center()
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(SharedString::from(tr("tools.filter", lang))),
            )
            .into_any_element()
    } else {
        let selected = selected_tool;
        uniform_list(
            "tool-uniform-list",
            item_count,
            move |range, _window, _app| {
                #[cfg(feature = "perf")]
                let perf_t0 = std::time::Instant::now();
                let total = filtered_tools.len();
                let start = range.start;
                let end = range.end.min(total);
                // Bound to `items` so the perf hook can read its length; the bare
                // let→return is only visible in non-perf builds.
                #[allow(clippy::let_and_return)]
                let items = (start..end)
                    .filter_map(|abs_ix| {
                        filtered_tools.get(abs_ix).map(|tool| {
                            let name = tool.name.clone();
                            let desc = tool.description.clone();
                            let is_sel = selected.as_deref() == Some(name.as_str());
                            let tool = (*tool).clone();
                            let on_select = on_select.clone();

                            ListItem::new(format!("tool-{abs_ix}"), SharedString::from(name))
                                .desc(SharedString::from(desc))
                                .selected(is_sel)
                                .on_click(
                                    move |_: &gpui::ClickEvent,
                                          window: &mut Window,
                                          cx: &mut gpui::App| {
                                        on_select(&tool, window, cx);
                                    },
                                )
                        })
                    })
                    .collect::<Vec<_>>();
                #[cfg(feature = "perf")]
                if let Some(p) = crate::app::perf::get() {
                    p.record_items(
                        "tools",
                        items.len() as u64,
                        total as u64,
                        perf_t0.elapsed().as_micros(),
                    );
                }
                items
            },
        )
        .flex_1()
        .w_full()
        .track_scroll(&tools_scroll)
        .into_any_element()
    };

    v_flex()
        .id("tool-list-col")
        .w(px(260.))
        .min_w(px(260.))
        // Fixed-width column (NOT flex_1): the detail-col is the flexible 1fr.
        // With flex_1 here, both columns grew 50/50 and the list doubled (028 #18).
        .flex_none()
        .h_full()
        .min_h(px(0.))
        .bg(surface)
        .border_r_1()
        .border_color(border)
        // Sticky search (fixed, outside the scroll → keeps full column width).
        .child(ListSearch::new(tool_filter_input).id("tools-search"))
        .child(body)
}
