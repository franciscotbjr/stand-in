//! Work area shell — the scrollable content region that hosts tab content
//! (M9+). Dispatches to the correct mode based on connection state and active
//! tab: **Disconnected/Error** → onboarding/error EmptyState;
//! **Connected Split** (Tools/Resources/Prompts) → list+detail scaffold;
//! **Connected Single** (Notifications/History) → single-column scaffold.
//!
//! The scroll chain follows §5b: every vertical segment uses explicit
//! `.flex().flex_col()`, each level carries `flex_1 + min_h(px(0.))`. The
//! scroll-leaf is `size_full() + overflow_y_scrollbar()`, but it MUST sit inside
//! its own `flex_1 + overflow_hidden` wrapper that is the sibling of any sticky
//! toolbar/header. Reason: `overflow_y_scrollbar()` re-wraps the leaf in a
//! `size_full` div that inherits only `size` (not `flex_1`/`min_h`), so the leaf
//! cannot itself be the flex child that shares a column with the toolbar — it
//! would take 100% height, ignore the toolbar, and push content off-screen
//! instead of scrolling (028 QA Erro #3). This mirrors the gpui-component
//! Dialog body (`flex_1 + overflow_hidden` wrapper around a `size_full` leaf).

use std::collections::HashSet;
use std::sync::Arc;

use crate::app::active_tab::{Tab, WorkMode, work_mode};
use crate::app::conn_state::ConnState;
use crate::app::events::ConnConfig;
use crate::app::history::HistoryEntry;
use crate::app::i18n::Lang;
use gpui::{
    Context, Entity, FontWeight, InteractiveElement, IntoElement, ListState, ParentElement,
    SharedString, Styled, UniformListScrollHandle, Window, div, px,
};
use gpui_component::input::InputState;
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex};
use stand_in::prompt::PromptArgument;
use stand_in_client::prelude::{PromptDefinition, Resource, ResourceTemplate, ToolDefinition};
use stand_in_mcp_explorer_ds::core::button::ClickHandler;

use super::prompts::PromptRun;
use super::resources::{ResourceRead, ResourceSelectFn, TemplateSelectFn};
use super::tools::ToolSelectFn;
use super::tools::schema::{ParamField, ToolRun};
use crate::app::log::{LogEntry, LogFilter};
use crate::app::studio_app::ResultView;

/// Render the work area — dispatches L-shaped to mode, then content shape.
#[allow(clippy::too_many_arguments)]
pub fn render_work_area<E: 'static>(
    state: &ConnState,
    active_tab: Tab,
    last_dispatched: Option<&ConnConfig>,
    lang: Lang,
    connect_handler: Option<ClickHandler>,
    capture_longtext: bool,
    capture_tools_state: Option<&str>,
    tools_data: Option<Arc<[ToolDefinition]>>,
    selected_tool: Option<&str>,
    tool_filter_input: Option<&Entity<InputState>>,
    tool_params: Option<&[(ParamField, Entity<InputState>)]>,
    guided: bool,
    on_select_tool: Option<ToolSelectFn>,
    tools_scroll: &UniformListScrollHandle,
    tool_run: &ToolRun,
    tool_validation: Option<&str>,
    result_view: ResultView,
    on_run: Option<ClickHandler>,
    on_result_view_toggle: Option<ClickHandler>,
    // Resources tab (M11) parameters
    resources_data: Option<&[Resource]>,
    templates_data: Option<&[ResourceTemplate]>,
    selected_resource_uri: Option<&str>,
    selected_template_uri: Option<&str>,
    subscribed_resources: &HashSet<String>,
    resource_filter_input: Option<&Entity<InputState>>,
    template_param_entities: Option<&[(String, Entity<InputState>)]>,
    resource_read: &ResourceRead,
    capture_resources_state: Option<&str>,
    on_select_concrete: Option<ResourceSelectFn>,
    on_select_template: Option<TemplateSelectFn>,
    on_resource_read: Option<ClickHandler>,
    on_subscribe: Option<ClickHandler>,
    on_unsubscribe: Option<ClickHandler>,
    resources_scroll: &UniformListScrollHandle,
    // Prompts tab (M12) parameters
    prompts_data: Option<&[PromptDefinition]>,
    selected_prompt: Option<&str>,
    prompt_filter_input: Option<&Entity<InputState>>,
    prompt_args: Option<&[(PromptArgument, Entity<InputState>)]>,
    prompt_run: &PromptRun,
    prompts_scroll: &UniformListScrollHandle,
    prompt_validation: Option<&str>,
    capture_prompts_state: Option<&str>,
    on_select_prompt: Option<super::prompts::PromptSelectFn>,
    on_generate: Option<ClickHandler>,
    // Notifications tab (M13) parameters
    logs: &[LogEntry],
    log_filter: LogFilter,
    notifications_list: &ListState,
    capture_notifications_state: Option<&str>,
    on_filter_change: Vec<ClickHandler>,
    on_clear_logs: Option<ClickHandler>,
    // History tab (M14)
    history_entries: &[HistoryEntry],
    history_list: &ListState,
    capture_history_state: Option<&str>,
    on_clear_history: Option<ClickHandler>,
    on_history_toggle: Vec<ClickHandler>,
    window: &mut Window,
    cx: &mut Context<E>,
) -> impl IntoElement {
    let mode = work_mode(state, active_tab);

    // Long-text capture fixture — proves scroll doesn't push siblings (BUG-11)
    if capture_longtext {
        return render_longtext_fixture(state, lang, window, cx).into_any_element();
    }

    match mode {
        WorkMode::Disconnected => crate::screens::onboarding::render_onboarding(
            state,
            last_dispatched,
            lang,
            connect_handler,
        )
        .into_any_element(),
        WorkMode::Error => crate::screens::error_state::render_error_state(
            state,
            last_dispatched,
            lang,
            connect_handler,
        )
        .into_any_element(),
        WorkMode::Split => match active_tab {
            Tab::Tools => {
                let tools = tools_data.unwrap_or_else(|| Vec::new().into());
                let filter =
                    tool_filter_input.expect("tool_filter_input must be initialised for Tools tab");
                let params = tool_params.unwrap_or(&[]);
                let sel = selected_tool;
                let handler =
                    on_select_tool.expect("on_select_tool must be provided for Tools tab");
                super::tools::render_tools(
                    tools,
                    sel,
                    filter,
                    params,
                    lang,
                    guided,
                    capture_tools_state,
                    handler,
                    tools_scroll,
                    tool_run,
                    tool_validation,
                    result_view,
                    on_run,
                    on_result_view_toggle,
                    window,
                    cx,
                )
                .into_any_element()
            }
            Tab::Resources => {
                let concretes = resources_data.unwrap_or(&[]);
                let templates = templates_data.unwrap_or(&[]);
                let filter = resource_filter_input
                    .expect("resource_filter_input must be initialised for Resources tab");
                let tpl_params = template_param_entities.unwrap_or(&[]);
                let sel_concrete = on_select_concrete
                    .expect("on_select_concrete must be provided for Resources tab");
                let sel_template = on_select_template
                    .expect("on_select_template must be provided for Resources tab");
                super::resources::render_resources(
                    concretes,
                    templates,
                    selected_resource_uri,
                    selected_template_uri,
                    subscribed_resources,
                    filter,
                    tpl_params,
                    resource_read,
                    lang,
                    capture_resources_state,
                    sel_concrete,
                    sel_template,
                    on_resource_read,
                    on_subscribe,
                    on_unsubscribe,
                    resources_scroll,
                    window,
                    cx,
                )
                .into_any_element()
            }
            Tab::Prompts => {
                let prompts = prompts_data.unwrap_or(&[]);
                let filter = prompt_filter_input
                    .expect("prompt_filter_input must be initialised for Prompts tab");
                let args = prompt_args.unwrap_or(&[]);
                let handler =
                    on_select_prompt.expect("on_select_prompt must be provided for Prompts tab");
                super::prompts::render_prompts(
                    prompts,
                    selected_prompt,
                    filter,
                    args,
                    lang,
                    capture_prompts_state,
                    handler,
                    prompt_run,
                    prompts_scroll,
                    prompt_validation,
                    on_generate,
                    window,
                    cx,
                )
                .into_any_element()
            }
            _ => render_split_scaffold(state, active_tab).into_any_element(),
        },
        WorkMode::Single => match active_tab {
            Tab::Notifications => super::notifications::render_notifications(
                logs,
                log_filter,
                notifications_list,
                lang,
                capture_notifications_state,
                on_filter_change,
                on_clear_logs,
                cx,
            )
            .into_any_element(),
            Tab::History => super::history::render_history(
                history_entries,
                history_list,
                lang,
                capture_history_state,
                on_clear_history,
                on_history_toggle,
                cx,
            )
            .into_any_element(),
            _ => render_single_scaffold(state, active_tab).into_any_element(),
        },
    }
}

// ---------------------------------------------------------------------------
// Split scaffold (list + detail side by side, each scrolling independently)
// ---------------------------------------------------------------------------

fn render_split_scaffold(_state: &ConnState, tab: Tab) -> impl IntoElement {
    let list_label = list_placeholder(tab);
    let detail_label = detail_placeholder(tab);

    h_flex()
        .id("split-scaffold")
        .flex_1()
        .min_h(px(0.))
        .gap_2()
        .child(
            // List column — fixed min width, scrolls independently
            v_flex()
                .id("split-list")
                .w(px(260.))
                .min_w(px(260.))
                .flex_1()
                .min_h(px(0.))
                .overflow_y_scrollbar()
                .child(scaffold_placeholder(list_label)),
        )
        .child(
            // Detail column — takes remaining width, scrolls independently
            v_flex()
                .id("split-detail")
                .flex_1()
                .min_w(px(0.))
                .min_h(px(0.))
                .overflow_y_scrollbar()
                .child(scaffold_placeholder(detail_label)),
        )
}

// ---------------------------------------------------------------------------
// Single-column scaffold (full width, scrolls)
// ---------------------------------------------------------------------------

fn render_single_scaffold(_state: &ConnState, tab: Tab) -> impl IntoElement {
    let label = single_placeholder(tab);

    v_flex()
        .id("single-scaffold")
        .flex_1()
        .min_h(px(0.))
        .size_full()
        .overflow_y_scrollbar()
        .child(scaffold_placeholder(label))
}

// ---------------------------------------------------------------------------
// Placeholder label helpers
// ---------------------------------------------------------------------------

fn list_placeholder(tab: Tab) -> SharedString {
    match tab {
        Tab::Tools => SharedString::from("tools list"),
        Tab::Resources => SharedString::from("resources list"),
        Tab::Prompts => SharedString::from("prompts list"),
        _ => SharedString::from("list"),
    }
}

fn detail_placeholder(tab: Tab) -> SharedString {
    match tab {
        Tab::Tools => SharedString::from("tool detail"),
        Tab::Resources => SharedString::from("resource detail"),
        Tab::Prompts => SharedString::from("prompt detail"),
        _ => SharedString::from("detail"),
    }
}

fn single_placeholder(tab: Tab) -> SharedString {
    match tab {
        Tab::Notifications => SharedString::from("notifications feed"),
        Tab::History => SharedString::from("history accordion"),
        _ => SharedString::from("content"),
    }
}

fn scaffold_placeholder(label: SharedString) -> impl IntoElement {
    v_flex()
        .id("scaffold-placeholder")
        .p_4()
        .gap_4()
        .items_center()
        .justify_center()
        .child(div().text_sm().font_weight(FontWeight::MEDIUM).child(label))
}

// ---------------------------------------------------------------------------
// Long-text capture fixture — fills both columns with vertical repetition
// ---------------------------------------------------------------------------

pub fn render_longtext_fixture<E: 'static>(
    _state: &ConnState,
    _lang: Lang,
    _window: &mut Window,
    _cx: &mut Context<E>,
) -> impl IntoElement {
    let long_text: Vec<String> = (0..80)
        .map(|i| format!("Row {} — esta linha existe apenas para provar que o scroll funciona corretamente sob o padrao §5b do BUG-11. O conteudo rola internamente e nao empurra a sidebar nem a topbar.", i + 1))
        .collect();

    // Split layout, both columns filled with long text
    h_flex()
        .id("longtext-fixture")
        .flex_1()
        .min_h(px(0.))
        .gap_2()
        .child(
            v_flex()
                .id("longtext-list")
                .w(px(260.))
                .min_w(px(260.))
                .flex_1()
                .min_h(px(0.))
                .overflow_y_scrollbar()
                .child(
                    v_flex().p_3().gap_2().children(
                        long_text
                            .iter()
                            .map(|s| div().text_xs().child(SharedString::from(s.clone()))),
                    ),
                ),
        )
        .child(
            v_flex()
                .id("longtext-detail")
                .flex_1()
                .min_w(px(0.))
                .min_h(px(0.))
                .overflow_y_scrollbar()
                .child(
                    v_flex().p_3().gap_2().children(
                        long_text
                            .iter()
                            .map(|s| div().text_xs().child(SharedString::from(s.clone()))),
                    ),
                ),
        )
        .into_any_element()
}
