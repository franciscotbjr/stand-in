//! Resources tab — split layout with live search, selection, metadata,
//! subscription management, and inline content viewing.
//!
//! Renders the list+detail pattern for the Resources tab, mirroring the
//! Tools tab architecture (M9/M10). The list column shows concrete resources
//! and URI templates; the detail column shows metadata, subscription
//! controls, and content (rendered across `ResourceRead` lifecycle states).

pub mod content;
mod resource_detail;
mod resource_list;

use std::collections::HashSet;

use gpui::{
    App, Context, Entity, InteractiveElement, IntoElement, ParentElement, Styled,
    UniformListScrollHandle, Window, px,
};
use gpui_component::h_flex;
use std::sync::Arc;

use crate::app::i18n::Lang;
use stand_in_client::prelude::{ReadResourceResult, Resource, ResourceTemplate};
use stand_in_mcp_explorer_ds::core::button::ClickHandler;

/// Resource read lifecycle state — mirrors `ToolRun`.
#[derive(Debug, Clone, Default)]
pub enum ResourceRead {
    #[default]
    Idle,
    Loading,
    Content(Box<ReadResourceResult>),
    Error(String),
}

/// Pure reducer for `ResourceRead` — zero gpui context.
pub fn reduce_resource_read(
    state: ResourceRead,
    event: &crate::app::events::EngineEvent,
) -> ResourceRead {
    match event {
        crate::app::events::EngineEvent::ResourceResult(r) => ResourceRead::Content(r.clone()),
        crate::app::events::EngineEvent::ResourceError(e) => ResourceRead::Error(e.clone()),
        crate::app::events::EngineEvent::Connected(_)
        | crate::app::events::EngineEvent::Disconnected => ResourceRead::Idle,
        _ => state,
    }
}

/// Type alias for concrete resource selection callback.
pub type ResourceSelectFn = Arc<dyn Fn(&Resource, &mut Window, &mut App) + Send + Sync>;

/// Type alias for template selection callback.
pub type TemplateSelectFn = Arc<dyn Fn(&ResourceTemplate, &mut Window, &mut App) + Send + Sync>;

/// Render the Resources tab content — split layout with list + detail.
#[allow(clippy::too_many_arguments)]
pub fn render_resources<E: 'static>(
    concretes: &[Resource],
    templates: &[ResourceTemplate],
    selected_concrete_uri: Option<&str>,
    selected_template_uri: Option<&str>,
    subscribed: &HashSet<String>,
    filter_input: &Entity<gpui_component::input::InputState>,
    template_param_entities: &[(String, Entity<gpui_component::input::InputState>)],
    resource_read: &ResourceRead,
    lang: Lang,
    _capture_state: Option<&str>,
    on_select_concrete: ResourceSelectFn,
    on_select_template: TemplateSelectFn,
    on_read: Option<ClickHandler>,
    on_subscribe: Option<ClickHandler>,
    on_unsubscribe: Option<ClickHandler>,
    resources_scroll: &UniformListScrollHandle,
    window: &mut Window,
    cx: &mut Context<E>,
) -> impl IntoElement {
    let filter_text = filter_input.read(cx).text().to_string();
    let filtered_concretes: Arc<[Resource]> = filter_resources(concretes, &filter_text)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>()
        .into();
    let filtered_templates: Arc<[ResourceTemplate]> =
        filter_resource_templates(templates, &filter_text)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>()
            .into();
    let n_concretes = filtered_concretes.len();
    let item_count = n_concretes + filtered_templates.len();
    let selected_uri = selected_concrete_uri.or(selected_template_uri);
    // Look up from the original slices — avoids a borrow/move conflict
    // with the Arc passed to render_resource_list.
    let sel_concrete =
        selected_concrete_uri.and_then(|uri| concretes.iter().find(|r| r.uri == uri));
    let sel_template =
        selected_template_uri.and_then(|uri| templates.iter().find(|t| t.uri_template == uri));

    // No gap: the divider is the list-col's border-right; the detail-col's
    // 22px gutter (canon `.detail-col`) provides the breathing room (028 #17).
    h_flex()
        .id("resources-split")
        .flex_1()
        .min_h(px(0.))
        .child(resource_list::render_resource_list(
            item_count,
            n_concretes,
            filtered_concretes,
            filtered_templates,
            selected_uri,
            subscribed,
            filter_input,
            lang,
            on_select_concrete,
            on_select_template,
            resources_scroll.clone(),
            window,
            cx,
        ))
        .child(resource_detail::render_resource_detail(
            sel_concrete,
            sel_template,
            template_param_entities,
            subscribed,
            resource_read,
            lang,
            on_read,
            on_subscribe,
            on_unsubscribe,
            window,
            cx,
        ))
}

// ---------------------------------------------------------------------------
// Pure filters — testable without gpui
// ---------------------------------------------------------------------------

/// Live filter: returns resources whose name, uri, or description matches the
/// query case-insensitively. Empty query → all.
pub fn filter_resources<'a>(resources: &'a [Resource], query: &str) -> Vec<&'a Resource> {
    if query.is_empty() {
        return resources.iter().collect();
    }
    let q = query.to_lowercase();
    resources
        .iter()
        .filter(|r| {
            r.name.to_lowercase().contains(&q)
                || r.uri.to_lowercase().contains(&q)
                || r.description
                    .as_deref()
                    .is_some_and(|d| d.to_lowercase().contains(&q))
        })
        .collect()
}

/// Live filter for templates — same logic as `filter_resources`.
pub fn filter_resource_templates<'a>(
    templates: &'a [ResourceTemplate],
    query: &str,
) -> Vec<&'a ResourceTemplate> {
    if query.is_empty() {
        return templates.iter().collect();
    }
    let q = query.to_lowercase();
    templates
        .iter()
        .filter(|t| {
            t.name.to_lowercase().contains(&q)
                || t.uri_template.to_lowercase().contains(&q)
                || t.description
                    .as_deref()
                    .is_some_and(|d| d.to_lowercase().contains(&q))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_resource(name: &str, uri: &str, desc: &str) -> Resource {
        Resource {
            name: name.into(),
            uri: uri.into(),
            description: Some(desc.into()),
            mime_type: None,
            size: None,
            annotations: None,
        }
    }

    fn make_template(name: &str, uri_template: &str, desc: &str) -> ResourceTemplate {
        ResourceTemplate {
            name: name.into(),
            uri_template: uri_template.into(),
            description: Some(desc.into()),
            mime_type: None,
        }
    }

    // --- filter_resources --------------------------------------------------

    #[test]
    fn test_filter_resources_empty_returns_all() {
        let r = vec![
            make_resource("readme", "file:///readme", "README file"),
            make_resource("config", "file:///config", "Config"),
        ];
        let filtered = filter_resources(&r, "");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filter_resources_by_name_case_insensitive() {
        let r = vec![
            make_resource("Readme", "file:///a", ""),
            make_resource("other", "file:///b", ""),
        ];
        let filtered = filter_resources(&r, "read");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Readme");
    }

    #[test]
    fn test_filter_resources_by_uri() {
        let r = vec![
            make_resource("a", "docs://rust/readme", ""),
            make_resource("b", "file:///other", ""),
        ];
        let filtered = filter_resources(&r, "rust");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].uri, "docs://rust/readme");
    }

    #[test]
    fn test_filter_resources_by_description() {
        let r = vec![
            make_resource("a", "file:///a", "hello world"),
            make_resource("b", "file:///b", "config"),
        ];
        let filtered = filter_resources(&r, "hello");
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_filter_resources_no_match() {
        let r = vec![make_resource("a", "file:///a", "desc")];
        let filtered = filter_resources(&r, "zzz");
        assert!(filtered.is_empty());
    }

    // --- filter_resource_templates -----------------------------------------

    #[test]
    fn test_filter_templates_empty_returns_all() {
        let t = vec![make_template("files", "file:///{path}", "any file")];
        let filtered = filter_resource_templates(&t, "");
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_filter_templates_by_name() {
        let t = vec![
            make_template("Files", "a://{x}", ""),
            make_template("other", "b://{y}", ""),
        ];
        let filtered = filter_resource_templates(&t, "file");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Files");
    }

    // --- reduce_resource_read -----------------------------------------------

    #[test]
    fn test_reduce_resource_result() {
        let r = ReadResourceResult::text("u", "text");
        let event = crate::app::events::EngineEvent::ResourceResult(Box::new(r));
        let state = reduce_resource_read(ResourceRead::Idle, &event);
        assert!(matches!(state, ResourceRead::Content(_)));
    }

    #[test]
    fn test_reduce_resource_error() {
        let event = crate::app::events::EngineEvent::ResourceError("fail".into());
        let state = reduce_resource_read(ResourceRead::Idle, &event);
        assert!(matches!(state, ResourceRead::Error(e) if e == "fail"));
    }

    #[test]
    fn test_reduce_resource_idle_on_disconnect() {
        let r = ReadResourceResult::text("u", "text");
        let event = crate::app::events::EngineEvent::ResourceResult(Box::new(r));
        let s = reduce_resource_read(ResourceRead::Idle, &event);
        assert!(matches!(s, ResourceRead::Content(_)));
        let s2 = reduce_resource_read(s, &crate::app::events::EngineEvent::Disconnected);
        assert!(matches!(s2, ResourceRead::Idle));
    }
}
