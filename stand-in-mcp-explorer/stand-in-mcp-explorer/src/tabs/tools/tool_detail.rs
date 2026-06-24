//! Tool detail column — header + "Parameters" panel + "Result" panel (live M10).
//!
//! The detail shows the selected tool's metadata, a parameter form, and a
//! result panel that renders across all `ToolRun` lifecycle states (Idle,
//! Running, Result-friendly, Result-raw, is_error data, protocol/transport error).

use gpui::{
    Context, ElementId, Entity, FontWeight, InteractiveElement, IntoElement, ParentElement,
    SharedString, Styled, Window, px,
};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{ActiveTheme as _, h_flex, v_flex};

use crate::app::i18n::Lang;
use crate::app::i18n::tr;
use crate::app::studio_app::ResultView;
use stand_in_client::prelude::ToolDefinition;
use stand_in_mcp_explorer_ds::core::button::ClickHandler;
use stand_in_mcp_explorer_ds::core::spinner::Spinner;
use stand_in_mcp_explorer_ds::core::toggle_link::ToggleLink;
use stand_in_mcp_explorer_ds::core::{
    Button, ButtonSize, ButtonVariant, IconName,
    icon::{Icon, IconSize},
};
use stand_in_mcp_explorer_ds::data::Panel;
use stand_in_mcp_explorer_ds::data::empty_state::result_empty;
use stand_in_mcp_explorer_ds::data::json_view::JsonView;
use stand_in_mcp_explorer_ds::theme::colors::JandiExt;

use super::params_form::render_params_form;
use super::schema::{ParamField, ToolRun};

/// Render the tool detail column (right side of the split layout).
#[allow(clippy::too_many_arguments)]
pub fn render_tool_detail<E: 'static>(
    tools: &[ToolDefinition],
    selected_tool: Option<&str>,
    tool_params: &[(ParamField, Entity<gpui_component::input::InputState>)],
    lang: Lang,
    _guided: bool,
    _capture_state: Option<&str>,
    tool_run: &ToolRun,
    tool_validation: Option<&str>,
    result_view: ResultView,
    on_run: Option<ClickHandler>,
    on_result_view_toggle: Option<ClickHandler>,
    window: &mut Window,
    cx: &mut Context<E>,
) -> impl IntoElement {
    let selected = selected_tool.and_then(|name| tools.iter().find(|t| t.name == name));
    let border = cx.theme().border;

    // Bounded column (mirrors the list-col): h_full so it fills the h_flex row,
    // min_h(0) so the scroll body can clip. No top-level overflow scroll.
    let mut col = v_flex()
        .id("tool-detail-col")
        .flex_1()
        .min_w(px(0.))
        .h_full()
        .min_h(px(0.));

    // Fixed action bar (only with a selection): tool name (left) + Run (right).
    // The button stays on top so the scroll never hides it (028 #18).
    if let Some(tool) = selected {
        col = col.child(
            h_flex()
                .id("tool-detail-bar")
                .flex_none()
                .items_center()
                .gap_2()
                .px(px(22.))
                .py(px(14.))
                .border_b_1()
                .border_color(border)
                .child(render_detail_title(tool, cx))
                .child(render_run_button(lang, tool_run, on_run)),
        );
    }

    // Scroll body (Dialog pattern, §5b): everything else scrolls — description,
    // Parameters and Result. The 22px gutter (#17) lives on the leaf now.
    col.child(
        gpui::div().flex_1().min_h(px(0.)).overflow_hidden().child(
            // BLOCK div (not v_flex) so children block-fill the width: a flex
            // leaf leaves its children at content width when the scroll is not
            // overflowing (gpui stretch is unreliable here), so short-content
            // panels shrink to their hint. Block-flow fills, like the description
            // does (028 #19). Vertical stacking still works (block flow).
            gpui::div()
                .id("tool-detail-scroll")
                .size_full()
                .overflow_y_scrollbar()
                .px(px(22.))
                .child(render_detail_body(selected, lang, cx))
                .child(render_params_panel(tool_params, lang, window, cx))
                .child(render_result_panel(
                    lang,
                    tool_run,
                    tool_validation,
                    result_view,
                    on_result_view_toggle,
                    cx,
                )),
        ),
    )
}

// ---------------------------------------------------------------------------
// Detail header — tool name (mono), description (sans)
// ---------------------------------------------------------------------------

/// Title for the fixed action bar: tool icon + name (mono). `flex_1`/`min_w(0)`
/// + truncate so a long name ellipsizes and the Run button stays right-aligned.
fn render_detail_title<E: 'static>(tool: &ToolDefinition, cx: &mut Context<E>) -> impl IntoElement {
    let mono = cx.theme().mono_font_family.clone();
    h_flex()
        .id("tool-detail-title")
        .flex_1()
        .min_w(px(0.))
        .gap_2()
        .items_center()
        .child(Icon::new(IconName::Tool).size(IconSize::Sm))
        .child(
            gpui::div()
                .flex_1()
                .min_w(px(0.))
                .text_sm()
                .font_family(mono)
                .font_weight(FontWeight::SEMIBOLD)
                .text_ellipsis()
                .whitespace_nowrap()
                .child(SharedString::from(tool.name.clone())),
        )
}

/// Scroll-body header: the tool description (the name moved to the fixed bar),
/// or the "select a tool" empty-state when nothing is selected.
fn render_detail_body<E: 'static>(
    selected: Option<&ToolDefinition>,
    lang: Lang,
    cx: &mut Context<E>,
) -> impl IntoElement {
    let muted = cx.theme().muted_foreground;
    match selected {
        Some(tool) => v_flex().id("tool-detail-desc").pt_4().pb_2().child(
            gpui::div()
                .text_sm()
                .text_color(muted)
                .child(SharedString::from(tool.description.clone())),
        ),
        None => v_flex()
            .id("tool-detail-empty")
            .p_4()
            .items_center()
            .justify_center()
            .child(
                gpui::div()
                    .text_sm()
                    .text_color(muted)
                    .child(SharedString::from(tr("tools.selectTool", lang))),
            ),
    }
}

// ---------------------------------------------------------------------------
// Parameters panel — driven by parsed schema
// ---------------------------------------------------------------------------

fn render_params_panel<E: 'static>(
    tool_params: &[(ParamField, Entity<gpui_component::input::InputState>)],
    lang: Lang,
    window: &mut Window,
    cx: &mut Context<E>,
) -> impl IntoElement {
    let title = SharedString::from(tr("tools.params", lang));
    let icon = IconName::Bolt;

    let body = render_params_form(tool_params, lang, window, cx);

    Panel::new()
        .id("panel-params")
        .title(title)
        .icon(icon)
        .children(vec![body.into_any_element()])
}

// ---------------------------------------------------------------------------
// Result panel — renders across ToolRun lifecycle states
// ---------------------------------------------------------------------------

fn render_result_panel<E: 'static>(
    lang: Lang,
    tool_run: &ToolRun,
    tool_validation: Option<&str>,
    result_view: ResultView,
    on_result_view_toggle: Option<ClickHandler>,
    cx: &mut Context<E>,
) -> impl IntoElement {
    let title = SharedString::from(tr("tools.result", lang));
    let icon = IconName::Play;
    let colors = &cx.theme().colors;
    let j = cx.global::<JandiExt>();

    // --- Right slot: toggle + copy (only when there is a result) ---
    let mut right = Vec::new();

    if let ToolRun::Result(_) = tool_run
        && let Some(toggle) = on_result_view_toggle
    {
        let label = match result_view {
            ResultView::Friendly => SharedString::from(tr("tools.rawJson", lang)),
            ResultView::Raw => SharedString::from(tr("tools.friendly", lang)),
        };
        let toggle_id = ElementId::from("btn-result-toggle");
        right.push(
            ToggleLink::new(toggle_id, label)
                .on_click(move |ev, w, cx| (toggle)(ev, w, cx))
                .into_any_element(),
        );
    }

    let body: gpui::AnyElement = match tool_run {
        ToolRun::Idle => {
            // Show validation error if present
            if let Some(msg) = tool_validation {
                let err_color = colors.danger_foreground;
                v_flex()
                    .id("result-validation")
                    .p_3()
                    .child(
                        gpui::div()
                            .text_sm()
                            .text_color(err_color)
                            .child(SharedString::from(msg.to_string())),
                    )
                    .into_any_element()
            } else {
                result_empty(tr("tools.noRun", lang)).into_any_element()
            }
        }
        ToolRun::Running => v_flex()
            .id("result-running")
            .py(px(16.))
            .gap_2()
            .items_center()
            .justify_center()
            .child(Spinner::new())
            .child(
                gpui::div()
                    .text_sm()
                    .text_color(colors.muted_foreground)
                    .child(SharedString::from(tr("tools.awaiting", lang))),
            )
            .into_any_element(),
        ToolRun::Result(r) => {
            let is_error_data = r.is_error == Some(true);

            if is_error_data {
                // Data-plane error — the tool returned is_error
                let content_text = r
                    .content
                    .iter()
                    .map(|c| match c {
                        stand_in_client::prelude::Content::Text { text } => text.as_str(),
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                v_flex()
                    .id("result-error-data")
                    .gap_2()
                    .child(
                        gpui::div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(SharedString::from(tr("tools.errorData", lang))),
                    )
                    .child(
                        gpui::div()
                            .id("result-error-data-code")
                            .rounded(px(6.))
                            .p_3()
                            .bg(j.code_bg)
                            .child(
                                gpui::div()
                                    .text_xs()
                                    .child(SharedString::from(content_text)),
                            ),
                    )
                    .into_any_element()
            } else {
                // Successful result
                let content_text = r
                    .content
                    .iter()
                    .map(|c| match c {
                        stand_in_client::prelude::Content::Text { text } => text.as_str(),
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                match result_view {
                    ResultView::Friendly => gpui::div()
                        .id("result-friendly")
                        .p_3()
                        .child(
                            gpui::div()
                                .text_sm()
                                .child(SharedString::from(content_text)),
                        )
                        .into_any_element(),
                    ResultView::Raw => {
                        let json_str = serde_json::to_string_pretty(r.as_ref())
                            .unwrap_or_else(|_| String::from("serialization error"));
                        JsonView::new(json_str).into_any_element()
                    }
                }
            }
        }
        ToolRun::Error(e) => {
            // Protocol/transport error
            v_flex()
                .id("result-error-protocol")
                .gap_2()
                .child(
                    gpui::div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(SharedString::from(tr("tools.errorProtocol", lang))),
                )
                .child(
                    gpui::div()
                        .id("result-error-protocol-code")
                        .rounded(px(6.))
                        .p_3()
                        .bg(j.code_bg)
                        .child(gpui::div().text_xs().child(SharedString::from(e.clone()))),
                )
                .into_any_element()
        }
    };

    Panel::new()
        .id("panel-result")
        .title(title)
        .icon(icon)
        .right_children(right)
        .children(vec![body])
}

// ---------------------------------------------------------------------------
// Run button — enabled, wired to the bridge
// ---------------------------------------------------------------------------

/// Run button for the fixed action bar (Primary). Built only when a tool is
/// selected; disabled while running or when no handler is wired (capture mode).
fn render_run_button(
    lang: Lang,
    tool_run: &ToolRun,
    on_run: Option<ClickHandler>,
) -> impl IntoElement {
    let is_running = matches!(tool_run, ToolRun::Running);
    let label = if is_running {
        SharedString::from(tr("tools.running", lang))
    } else {
        SharedString::from(tr("tools.run", lang))
    };

    let btn = Button::new(label)
        .id("btn-run")
        .variant(ButtonVariant::Primary)
        .size(ButtonSize::Md)
        .icon(IconName::Play);

    match (is_running, on_run) {
        (false, Some(handler)) => btn.on_click(handler),
        _ => btn.disabled(),
    }
}
