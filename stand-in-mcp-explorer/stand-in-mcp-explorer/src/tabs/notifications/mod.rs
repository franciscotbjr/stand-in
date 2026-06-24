//! Notifications tab (M13) — single-column log feed, windowed via
//! `gpui::list` (031/M4). Row height is variable (`LogRow` wraps long
//! messages), so `uniform_list` is not applicable.
//!
//! Toolbar: SegmentedControl filter [all·info·ok·warn·error] + event count + Clear.
//! Feed is pre-filtered into an `Arc<[LogEntry]>` once per render; the `list`
//! closure indexes it by item index (newest-first). The closure never calls
//! `filter_logs` — O(visible) per scroll-frame, not O(N).

use std::sync::Arc;

use crate::app::i18n::{Lang, tr, tr_args};
use crate::app::log::{LogEntry, LogFilter, filter_logs};
use gpui::prelude::FluentBuilder;
use gpui::{
    App, Hsla, InteractiveElement, IntoElement, ListState, ParentElement, SharedString, Styled, px,
};
use gpui_component::{ActiveTheme as _, h_flex, v_flex};
use stand_in_mcp_explorer_ds::core::button::{Button, ButtonVariant, ClickHandler};
use stand_in_mcp_explorer_ds::core::icon::IconName;
use stand_in_mcp_explorer_ds::data::{LogLevel, LogRow};
use stand_in_mcp_explorer_ds::forms::segmented_control::SegmentedControl;

#[allow(clippy::too_many_arguments)]
pub fn render_notifications(
    logs: &[LogEntry],
    log_filter: LogFilter,
    notifications_list: &ListState,
    lang: Lang,
    _capture_notifications_state: Option<&str>,
    on_filter_change: Vec<ClickHandler>,
    on_clear_logs: Option<ClickHandler>,
    cx: &mut App,
) -> impl IntoElement {
    let filtered: Arc<[LogEntry]> = filter_logs(logs, &log_filter)
        .iter()
        .rev()
        .map(|e| (*e).clone())
        .collect::<Vec<_>>()
        .into();
    let item_count = filtered.len();
    let colors = cx.theme().colors;
    let list_state = notifications_list.clone();

    v_flex()
        .id("notifications-tab")
        .flex_1()
        .min_h(px(0.))
        .child(render_toolbar(
            log_filter,
            lang,
            item_count,
            on_filter_change,
            on_clear_logs,
            colors.list_head,
            colors.border,
        ))
        .child({
            if item_count == 0 {
                render_empty_state(lang).into_any_element()
            } else {
                gpui::list(list_state, move |i, _window, _app| {
                    let Some(entry) = filtered.get(i) else {
                        return gpui::div().into_any_element();
                    };
                    let level_label = level_label(entry.level);
                    let time = entry.time.clone();
                    let message = entry.message.clone();
                    LogRow::new(time, entry.level, level_label, message)
                        .id(format!("notif-row-{}", i))
                        .into_any_element()
                })
                .flex_1()
                .w_full()
                .into_any_element()
            }
        })
}

fn render_toolbar(
    log_filter: LogFilter,
    lang: Lang,
    count: usize,
    on_filter_change: Vec<ClickHandler>,
    on_clear_logs: Option<ClickHandler>,
    head_bg: Hsla,
    head_border: Hsla,
) -> impl IntoElement {
    let events_count = tr_args(
        "notifications.eventsCount",
        lang,
        &[("n", &count.to_string())],
    );

    h_flex()
        .id("notifications-toolbar")
        .w_full()
        .bg(head_bg)
        .border_b_1()
        .border_color(head_border)
        .px(px(16.))
        .py(px(8.))
        .gap(px(8.))
        .items_center()
        .child(
            SegmentedControl::new(
                "seg-log-filter",
                vec![
                    (
                        SharedString::from("all"),
                        SharedString::from(tr("notifications.all", lang)),
                    ),
                    (SharedString::from("info"), level_label(LogLevel::Info)),
                    (SharedString::from("ok"), level_label(LogLevel::Ok)),
                    (SharedString::from("warn"), level_label(LogLevel::Warn)),
                    (SharedString::from("error"), level_label(LogLevel::Error)),
                ],
                log_filter.selected_ix(),
            )
            .handlers(on_filter_change),
        )
        .child(
            gpui::div()
                .flex_1()
                .child(gpui::div().text_xs().child(events_count)),
        )
        .when_some(on_clear_logs, |bar, on_clear| {
            bar.child(
                Button::new(tr("notifications.clear", lang))
                    .id("notif-clear")
                    .variant(ButtonVariant::Ghost)
                    .icon(IconName::X)
                    .on_click(move |ev, w, cx| on_clear(ev, w, cx)),
            )
        })
}

fn render_empty_state(lang: Lang) -> impl IntoElement {
    v_flex()
        .id("notifications-empty")
        .flex_1()
        .items_center()
        .justify_center()
        .p(px(24.))
        .gap(px(8.))
        .child(
            gpui::div()
                .text_sm()
                .child(SharedString::from(tr("notifications.empty", lang))),
        )
}

fn level_label(level: LogLevel) -> SharedString {
    match level {
        LogLevel::Info => SharedString::from("info"),
        LogLevel::Ok => SharedString::from("ok"),
        LogLevel::Warn => SharedString::from("warn"),
        LogLevel::Error => SharedString::from("error"),
        LogLevel::Debug => SharedString::from("debug"),
    }
}
