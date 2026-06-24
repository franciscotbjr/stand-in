//! Tools tab — split layout with live search, selection, and parameter details.
//!
//! Renders the list+detail pattern for the Tools tab. The list column has a
//! `ListSearch` (sticky) and `ListItem`s filtered by a live query; the detail
//! column shows the selected tool's header, parameter form, and result panel
//! (empty / no-run until M10).

mod params_form;
pub mod schema;
mod tool_detail;
mod tool_list;

use gpui::{
    App, AppContext as _, Context, Entity, InteractiveElement, IntoElement, ParentElement, Styled,
    UniformListScrollHandle, Window, px,
};
use gpui_component::h_flex;
use std::sync::Arc;

use crate::app::i18n::Lang;
use stand_in_client::prelude::ToolDefinition;
use stand_in_mcp_explorer_ds::core::button::ClickHandler;

use crate::app::studio_app::ResultView;

/// Type alias for the tool selection callback.
pub type ToolSelectFn = Arc<dyn Fn(&ToolDefinition, &mut Window, &mut App) + Send + Sync>;

/// Render the Tools tab content — split layout with list + detail.
#[allow(clippy::too_many_arguments)]
pub fn render_tools<E: 'static>(
    tools: Arc<[ToolDefinition]>,
    selected_tool: Option<&str>,
    tool_filter_input: &Entity<gpui_component::input::InputState>,
    tool_params: &[(
        schema::ParamField,
        Entity<gpui_component::input::InputState>,
    )],
    lang: Lang,
    guided: bool,
    capture_state: Option<&str>,
    on_select: ToolSelectFn,
    tools_scroll: &UniformListScrollHandle,
    tool_run: &schema::ToolRun,
    tool_validation: Option<&str>,
    result_view: ResultView,
    on_run: Option<ClickHandler>,
    on_result_view_toggle: Option<ClickHandler>,
    window: &mut Window,
    cx: &mut Context<E>,
) -> impl IntoElement {
    let filter_text = tool_filter_input.read(cx).text().to_string();
    // O-013: with no filter, reuse the snapshot Arc (O(1)) instead of cloning
    // all N tools into a fresh Arc every frame; only clone-filter when querying.
    #[cfg(feature = "perf")]
    let _c0 = std::time::Instant::now();
    let (item_count, filtered_arc): (usize, Arc<[ToolDefinition]>) = if filter_text.is_empty() {
        (tools.len(), Arc::clone(&tools))
    } else {
        let filtered = filter_tools(&tools, &filter_text);
        let count = filtered.len();
        (
            count,
            filtered.into_iter().cloned().collect::<Vec<_>>().into(),
        )
    };
    #[cfg(feature = "perf")]
    if let Some(p) = crate::app::perf::get() {
        p.record_clone(_c0.elapsed().as_micros());
    }

    // No gap: the divider is the list-col's border-right; the detail-col's
    // 22px gutter (canon `.detail-col`) provides the breathing room (028 #17).
    h_flex()
        .id("tools-split")
        .flex_1()
        .min_h(px(0.))
        .child(tool_list::render_tool_list(
            item_count,
            filtered_arc,
            selected_tool.map(String::from),
            tool_filter_input,
            lang,
            capture_state,
            on_select.clone(),
            tools_scroll.clone(),
            window,
            cx,
        ))
        .child(tool_detail::render_tool_detail(
            &tools,
            selected_tool,
            tool_params,
            lang,
            guided,
            capture_state,
            tool_run,
            tool_validation,
            result_view,
            on_run,
            on_result_view_toggle,
            window,
            cx,
        ))
}

/// Rebuild parameters when the selected tool changes.
pub fn rebuild_params<E: 'static>(
    selected_tool: Option<&str>,
    tools: &[ToolDefinition],
    window: &mut Window,
    cx: &mut Context<E>,
) -> Vec<(
    schema::ParamField,
    Entity<gpui_component::input::InputState>,
)> {
    let Some(name) = selected_tool else {
        return vec![];
    };

    let Some(tool) = tools.iter().find(|t| t.name == name) else {
        return vec![];
    };

    schema::parse_params(&tool.input_schema)
        .into_iter()
        .map(|field| {
            let state = cx.new(|cx| gpui_component::input::InputState::new(window, cx));
            (field, state)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Pure filter — testable without gpui
// ---------------------------------------------------------------------------

/// Live filter: returns tools whose name or description matches the query
/// case-insensitively. Empty query → all tools.
pub fn filter_tools<'a>(tools: &'a [ToolDefinition], query: &str) -> Vec<&'a ToolDefinition> {
    if query.is_empty() {
        return tools.iter().collect();
    }
    let q = query.to_lowercase();
    tools
        .iter()
        .filter(|t| t.name.to_lowercase().contains(&q) || t.description.to_lowercase().contains(&q))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn td(name: &str, desc: &str) -> ToolDefinition {
        ToolDefinition {
            name: name.into(),
            description: desc.into(),
            input_schema: stand_in_client::prelude::InputSchema::object(),
        }
    }

    #[test]
    fn test_filter_empty_returns_all() {
        let tools = vec![td("greet", "Say hello"), td("weather", "Get weather")];
        let filtered = filter_tools(&tools, "");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filter_by_name_case_insensitive() {
        let tools = vec![td("Greet", "Say hello"), td("weather", "Get weather")];
        let filtered = filter_tools(&tools, "gr");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Greet");
    }

    #[test]
    fn test_filter_by_description() {
        let tools = vec![td("greet", "Say hello"), td("weather", "Get weather")];
        let filtered = filter_tools(&tools, "hello");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "greet");
    }

    #[test]
    fn test_filter_no_match() {
        let tools = vec![td("greet", "Say hello")];
        let filtered = filter_tools(&tools, "zzz");
        assert!(filtered.is_empty());
    }
}
