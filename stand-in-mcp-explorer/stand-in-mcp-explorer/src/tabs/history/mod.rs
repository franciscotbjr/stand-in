//! History tab (M14) — accordion request→response feed, windowed via
//! `gpui::list` (031/M4). Row height is variable (expand/collapse changes
//! height), so `uniform_list` is not applicable.
//!
//! Single-column tab populated by real tool calls and prompt generations.
//! Each entry is a collapsible row: kind badge + name + timing + timestamp
//! + chevron. When expanded, the request and response are shown side by
//!   side as `JsonView` panels.
//!
//! The buffer is filled at drain time from `ToolResult` and `PromptMessages`
//! events (see `app/history.rs`). Newest entries appear first.
//!
//! The toggle handler calls `ListState::splice(i..i+1, 1)` on the
//! `history_list` held in `StudioApp` so the list re-measures the resized
//! row (the critical requirement — without it, height is stale).

use std::rc::Rc;
use std::sync::Arc;

use crate::app::history::{HistKind, HistoryEntry};
use crate::app::i18n::{Lang, tr, tr_args};
use gpui::prelude::FluentBuilder;
use gpui::{
    App, ClickEvent, Hsla, InteractiveElement, IntoElement, ListState, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, Transformation, Window, percentage, px,
};
use gpui_component::{ActiveTheme as _, h_flex, v_flex};
use stand_in_mcp_explorer_ds::core::badge::{Badge, BadgeKind};
use stand_in_mcp_explorer_ds::core::button::{Button, ButtonVariant, ClickHandler};
use stand_in_mcp_explorer_ds::core::icon::{Icon, IconName, IconSize};
use stand_in_mcp_explorer_ds::data::ListScrollHoverSuppressed;
use stand_in_mcp_explorer_ds::data::json_view::JsonView;
use stand_in_mcp_explorer_ds::navigation::section_label::SectionLabel;

#[allow(clippy::too_many_arguments)]
pub fn render_history(
    entries: &[HistoryEntry],
    history_list: &ListState,
    lang: Lang,
    _capture_history_state: Option<&str>,
    on_clear: Option<ClickHandler>,
    on_toggle: Vec<ClickHandler>,
    cx: &mut App,
) -> impl IntoElement {
    let item_count = entries.len();
    let colors = cx.theme().colors;
    // O-025: suppress row hover during mouse-wheel scroll (set by the app).
    let suppress_hover = cx
        .try_global::<ListScrollHoverSuppressed>()
        .is_some_and(|g| g.0);
    let list_state = history_list.clone();

    // Clone entries into Arc so the `list` closure (which is 'static) can
    // index them. This is 1×/render, O(N) clone — like the M2/M3 pattern.
    #[cfg(feature = "perf")]
    let _c0 = std::time::Instant::now();
    let arc_entries: Arc<[HistoryEntry]> = entries.to_vec().into();
    #[cfg(feature = "perf")]
    if let Some(p) = crate::app::perf::get() {
        p.record_clone(_c0.elapsed().as_micros());
    }

    // Pre-built toggle handlers shared across all rows via Arc. Each row
    // clones the Arc and calls `handlers[i]` by index.
    let toggle_handlers: Rc<Vec<ClickHandler>> = Rc::new(on_toggle);

    v_flex()
        .id("history-tab")
        .flex_1()
        .min_h(px(0.))
        .child(render_toolbar(
            lang,
            item_count,
            on_clear,
            colors.list_head,
            colors.border,
        ))
        .child({
            if item_count == 0 {
                render_empty_state(lang).into_any_element()
            } else {
                let border = colors.border;
                let hover_bg = colors.secondary;
                let entries_for_list = Arc::clone(&arc_entries);
                gpui::list(list_state, move |i, _window, _app| {
                    #[cfg(feature = "perf")]
                    let perf_t0 = std::time::Instant::now();
                    #[cfg(feature = "perf")]
                    let total = entries_for_list.len();
                    let Some(entry) = entries_for_list.get(i) else {
                        return gpui::div().into_any_element();
                    };
                    let handlers_for_row = Rc::clone(&toggle_handlers);
                    let on_row_toggle = move |ev: &ClickEvent, w: &mut Window, a: &mut App| {
                        (handlers_for_row[i])(ev, w, a);
                    };
                    // Bound to `el` so the perf hook can run after the build; the
                    // bare let→return is only visible in non-perf builds.
                    #[allow(clippy::let_and_return)]
                    let el = render_history_row(
                        i,
                        entry,
                        lang,
                        on_row_toggle,
                        border,
                        hover_bg,
                        suppress_hover,
                    )
                    .into_any_element();
                    #[cfg(feature = "perf")]
                    if let Some(p) = crate::app::perf::get() {
                        p.record_items("history", 1, total as u64, perf_t0.elapsed().as_micros());
                    }
                    el
                })
                .flex_1()
                .w_full()
                .into_any_element()
            }
        })
}

fn render_toolbar(
    lang: Lang,
    count: usize,
    on_clear: Option<ClickHandler>,
    head_bg: Hsla,
    head_border: Hsla,
) -> impl IntoElement {
    let count_label = tr_args(
        "notifications.eventsCount",
        lang,
        &[("n", &count.to_string())],
    );

    h_flex()
        .id("history-toolbar")
        .w_full()
        .bg(head_bg)
        .border_b_1()
        .border_color(head_border)
        .px(px(16.))
        .py(px(8.))
        .gap(px(8.))
        .items_center()
        .child(
            gpui::div()
                .flex_1()
                .child(gpui::div().text_xs().child(count_label)),
        )
        .when_some(on_clear, |bar, on_clear| {
            bar.child(
                Button::new(tr("history.clear", lang))
                    .id("history-clear")
                    .variant(ButtonVariant::Ghost)
                    .icon(IconName::X)
                    .on_click(move |ev, w, cx| on_clear(ev, w, cx)),
            )
        })
}

#[allow(clippy::too_many_arguments)]
fn render_history_row(
    index: usize,
    entry: &HistoryEntry,
    lang: Lang,
    on_toggle: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    border: Hsla,
    hover_bg: Hsla,
    suppress_hover: bool,
) -> impl IntoElement {
    let kind_label = match entry.kind {
        HistKind::Tool => SharedString::from(tr("history.kind.tool", lang)),
        HistKind::Prompt => SharedString::from(tr("history.kind.prompt", lang)),
    };
    let badge_kind = match (entry.kind, entry.has_error) {
        (HistKind::Prompt, _) => BadgeKind::Role,
        (_, true) => BadgeKind::Write,
        (_, false) => BadgeKind::Read,
    };
    let timing_str = entry.timing_ms.map_or(String::new(), |ms| {
        tr_args("history.timing", lang, &[("ms", &ms.to_string())])
    });
    let name_mono = SharedString::from(entry.name.as_str());
    let time_str = SharedString::from(entry.time.as_str());

    let chevron = if entry.expanded {
        Icon::new(IconName::Chevron)
            .size(IconSize::Sm)
            .rotate(Transformation::rotate(percentage(0.25)))
    } else {
        Icon::new(IconName::Chevron).size(IconSize::Sm)
    };

    let row_id = format!("history-row-{}", index);

    let req_pretty = serde_json::to_string_pretty(&entry.request).unwrap_or_default();
    let res_pretty = serde_json::to_string_pretty(&entry.response).unwrap_or_default();

    v_flex()
        .id(row_id)
        .w_full()
        .border_b_1()
        .border_color(border)
        .child(
            // Collapsed row header — always visible
            h_flex()
                .id(format!("history-header-{}", index))
                .w_full()
                .px(px(16.))
                .py(px(10.))
                .gap_2()
                .items_center()
                .cursor_pointer()
                .when(!suppress_hover, |row| row.hover(|h| h.bg(hover_bg)))
                .on_click(on_toggle)
                .child(Badge::new(kind_label.clone(), badge_kind))
                .child(
                    gpui::div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(
                            gpui::div()
                                .text_sm()
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .child(name_mono.clone()),
                        )
                        .when(entry.timing_ms.is_some(), |row| {
                            row.child(
                                gpui::div()
                                    .text_xs()
                                    .child(SharedString::from(timing_str.as_str())),
                            )
                        }),
                )
                .child(gpui::div().text_xs().child(time_str.clone()))
                .child(chevron),
        )
        .when(entry.expanded, |col| {
            col.child(
                h_flex()
                    .id(format!("history-detail-{}", index))
                    .w_full()
                    .min_w(px(0.))
                    .gap(px(6.))
                    .px(px(16.))
                    .pb(px(12.))
                    .child(
                        // Request column
                        v_flex()
                            .flex_1()
                            .min_w(px(0.))
                            .gap(px(4.))
                            .child(SectionLabel::new(tr("history.request", lang)))
                            .child(
                                JsonView::new(SharedString::from(req_pretty.as_str()))
                                    .id(format!("history-req-{}", index)),
                            ),
                    )
                    .child(
                        // Response column
                        v_flex()
                            .flex_1()
                            .min_w(px(0.))
                            .gap(px(4.))
                            .child(SectionLabel::new(tr("history.response", lang)))
                            .child(
                                JsonView::new(SharedString::from(res_pretty.as_str()))
                                    .id(format!("history-res-{}", index)),
                            ),
                    ),
            )
        })
}

fn render_empty_state(lang: Lang) -> impl IntoElement {
    v_flex()
        .id("history-empty")
        .flex_1()
        .items_center()
        .justify_center()
        .p(px(24.))
        .gap(px(8.))
        .child(
            gpui::div()
                .text_sm()
                .child(SharedString::from(tr("history.empty", lang))),
        )
}
