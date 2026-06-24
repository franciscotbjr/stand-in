//! Prompts tab — split layout with live search, selection, argument form,
//! and generated message preview with role badges.
//!
//! Mirrors the Tools tab architecture (M9/M10) — simpler because prompt
//! arguments are always strings (no JSON Schema type inference).

pub mod args;
mod prompt_detail;
mod prompt_list;

use gpui::{
    App, AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Styled,
    UniformListScrollHandle, Window, px,
};
use gpui_component::h_flex;
use std::sync::Arc;

use crate::app::i18n::Lang;
use stand_in::prompt::PromptArgument;
use stand_in_client::prelude::PromptDefinition;
use stand_in_mcp_explorer_ds::core::button::ClickHandler;

pub use args::PromptRun;

/// Type alias for the prompt selection callback.
pub type PromptSelectFn = Arc<dyn Fn(&PromptDefinition, &mut Window, &mut App) + Send + Sync>;

/// Render the Prompts tab content — split layout with list + detail.
#[allow(clippy::too_many_arguments)]
pub fn render_prompts<E: 'static>(
    prompts: &[PromptDefinition],
    selected_prompt: Option<&str>,
    prompt_filter_input: &Entity<gpui_component::input::InputState>,
    prompt_args: &[(PromptArgument, Entity<gpui_component::input::InputState>)],
    lang: Lang,
    capture_state: Option<&str>,
    on_select: PromptSelectFn,
    prompt_run: &PromptRun,
    prompts_scroll: &UniformListScrollHandle,
    prompt_validation: Option<&str>,
    on_generate: Option<ClickHandler>,
    window: &mut Window,
    cx: &mut Context<E>,
) -> impl IntoElement {
    let filter_text = prompt_filter_input.read(cx).text().to_string();
    let filtered: Arc<[PromptDefinition]> = filter_prompts(prompts, &filter_text)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>()
        .into();
    let item_count = filtered.len();

    // No gap: the divider is the list-col's border-right; the detail-col's
    // 22px gutter (canon `.detail-col`) provides the breathing room (028 #17).
    h_flex()
        .id("prompts-split")
        .flex_1()
        .min_h(px(0.))
        .child(prompt_list::render_prompt_list(
            item_count,
            filtered,
            selected_prompt,
            prompt_filter_input,
            lang,
            capture_state,
            on_select.clone(),
            prompts_scroll.clone(),
            window,
            cx,
        ))
        .child(prompt_detail::render_prompt_detail(
            prompts,
            selected_prompt,
            prompt_args,
            lang,
            capture_state,
            prompt_run,
            prompt_validation,
            on_generate,
            window,
            cx,
        ))
}

/// Rebuild argument entities when the selected prompt changes.
pub fn rebuild_prompt_args<E: 'static>(
    selected_prompt: Option<&str>,
    prompts: &[PromptDefinition],
    window: &mut Window,
    cx: &mut Context<E>,
) -> Vec<(PromptArgument, Entity<gpui_component::input::InputState>)> {
    let Some(name) = selected_prompt else {
        return vec![];
    };
    let Some(prompt) = prompts.iter().find(|p| p.name == name) else {
        return vec![];
    };
    let args = prompt.arguments.clone().unwrap_or_default();
    args.into_iter()
        .map(|arg| {
            let state = cx.new(|cx| gpui_component::input::InputState::new(window, cx));
            (arg, state)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Pure filter — testable without gpui
// ---------------------------------------------------------------------------

/// Live filter: returns prompts whose name or description matches the query
/// case-insensitively. Empty query → all prompts.
pub fn filter_prompts<'a>(
    prompts: &'a [PromptDefinition],
    query: &str,
) -> Vec<&'a PromptDefinition> {
    if query.is_empty() {
        return prompts.iter().collect();
    }
    let q = query.to_lowercase();
    prompts
        .iter()
        .filter(|p| p.name.to_lowercase().contains(&q) || p.description.to_lowercase().contains(&q))
        .collect()
}

/// Pure reducer for `PromptRun` — zero gpui context.
pub fn reduce_prompt_run(state: PromptRun, event: &crate::app::events::EngineEvent) -> PromptRun {
    match event {
        crate::app::events::EngineEvent::PromptMessages(r) => PromptRun::Messages(r.clone()),
        crate::app::events::EngineEvent::PromptError(e) => PromptRun::Error(e.clone()),
        crate::app::events::EngineEvent::Connected(_)
        | crate::app::events::EngineEvent::Disconnected => PromptRun::Idle,
        _ => state,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pd(name: &str, desc: &str) -> PromptDefinition {
        PromptDefinition {
            name: name.into(),
            description: desc.into(),
            arguments: None,
        }
    }

    #[test]
    fn test_filter_empty_returns_all() {
        let prompts = vec![
            pd("greeting", "Generate a greeting"),
            pd("review", "Code review"),
        ];
        let filtered = filter_prompts(&prompts, "");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filter_by_name_case_insensitive() {
        let prompts = vec![pd("Greeting", "Hello"), pd("review", "Code review")];
        let filtered = filter_prompts(&prompts, "greet");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Greeting");
    }

    #[test]
    fn test_filter_by_description() {
        let prompts = vec![pd("greeting", "Say hello"), pd("review", "Code review")];
        let filtered = filter_prompts(&prompts, "hello");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "greeting");
    }

    #[test]
    fn test_filter_no_match() {
        let prompts = vec![pd("greeting", "Say hello")];
        let filtered = filter_prompts(&prompts, "zzz");
        assert!(filtered.is_empty());
    }

    #[test]
    fn test_reduce_prompt_run_messages() {
        use crate::app::events::EngineEvent;
        use stand_in_client::prelude::{GetPromptResult, PromptContent, PromptMessage, PromptRole};
        let result = GetPromptResult {
            description: None,
            messages: vec![PromptMessage {
                role: PromptRole::User,
                content: PromptContent::Text {
                    text: "Write a friendly greeting for stand-in.".into(),
                },
            }],
        };
        let event = EngineEvent::PromptMessages(Box::new(result));
        let state = reduce_prompt_run(PromptRun::Building, &event);
        assert!(matches!(state, PromptRun::Messages(_)));
    }

    #[test]
    fn test_reduce_prompt_run_error() {
        use crate::app::events::EngineEvent;
        let event = EngineEvent::PromptError("protocol error".into());
        let state = reduce_prompt_run(PromptRun::Building, &event);
        match state {
            PromptRun::Error(e) => assert!(e.contains("protocol")),
            _ => panic!("expected Error"),
        }
    }

    #[test]
    fn test_reduce_prompt_run_reset_on_disconnect() {
        use crate::app::events::EngineEvent;
        let event = EngineEvent::Disconnected;
        let state = reduce_prompt_run(
            PromptRun::Messages(Box::new(stand_in_client::prelude::GetPromptResult {
                description: None,
                messages: vec![],
            })),
            &event,
        );
        assert!(matches!(state, PromptRun::Idle));
    }
}
