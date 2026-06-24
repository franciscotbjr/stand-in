//! Prompt detail column — header + "Arguments" panel + "Messages" panel.
//!
//! The detail shows the selected prompt's metadata, an argument form (Fields),
//! and a messages panel that renders across all `PromptRun` lifecycle states
//! (Idle, Building, Messages with role-badge cards, Error).

use gpui::{
    Context, ElementId, Entity, FontWeight, InteractiveElement, IntoElement, ParentElement,
    SharedString, Styled, Window, div, px,
};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{ActiveTheme as _, h_flex, v_flex};

use crate::app::i18n::Lang;
use crate::app::i18n::tr;
use stand_in::prompt::PromptArgument;
use stand_in_client::prelude::{PromptContent, PromptDefinition};
use stand_in_mcp_explorer_ds::core::badge::{Badge, BadgeKind};
use stand_in_mcp_explorer_ds::core::button::ClickHandler;
use stand_in_mcp_explorer_ds::core::copy_button::CopyButton;
use stand_in_mcp_explorer_ds::core::icon::{Icon, IconName, IconSize};
use stand_in_mcp_explorer_ds::core::spinner::Spinner;
use stand_in_mcp_explorer_ds::core::{Button, ButtonSize, ButtonVariant};
use stand_in_mcp_explorer_ds::data::Panel;
use stand_in_mcp_explorer_ds::data::empty_state::result_empty;
use stand_in_mcp_explorer_ds::forms::Field;
use stand_in_mcp_explorer_ds::theme::colors::JandiExt;

use super::args::{PromptRun, role_label};

/// Render the prompt detail column (right side of the split layout).
#[allow(clippy::too_many_arguments)]
pub fn render_prompt_detail<E: 'static>(
    prompts: &[PromptDefinition],
    selected_prompt: Option<&str>,
    prompt_args: &[(PromptArgument, Entity<gpui_component::input::InputState>)],
    lang: Lang,
    _capture_state: Option<&str>,
    prompt_run: &PromptRun,
    prompt_validation: Option<&str>,
    on_generate: Option<ClickHandler>,
    _window: &mut Window,
    cx: &mut Context<E>,
) -> impl IntoElement {
    let selected = selected_prompt.and_then(|name| prompts.iter().find(|p| p.name == name));
    let has_args = !prompt_args.is_empty();
    let border = cx.theme().border;

    // Bounded column (mirrors the list-col): h_full + min_h(0), no top-level scroll.
    let mut col = v_flex()
        .id("prompt-detail-col")
        .flex_1()
        .min_w(px(0.))
        .h_full()
        .min_h(px(0.));

    // Fixed action bar (only with a selection): prompt name (left) + Generate
    // (right). The button stays on top so the scroll never hides it (028 #18).
    if let Some(prompt) = selected {
        col = col.child(
            h_flex()
                .id("prompt-detail-bar")
                .flex_none()
                .items_center()
                .gap_2()
                .px(px(22.))
                .py(px(14.))
                .border_b_1()
                .border_color(border)
                .child(render_detail_title(prompt, cx))
                .child(render_generate_button(lang, prompt_run, on_generate)),
        );
    }

    // Scroll body (Dialog pattern, §5b): description + Arguments + Messages.
    col.child(
        div().flex_1().min_h(px(0.)).overflow_hidden().child(
            // BLOCK div (not v_flex) so children block-fill the width even when
            // the scroll is not overflowing (028 #19).
            div()
                .id("prompt-detail-scroll")
                .size_full()
                .overflow_y_scrollbar()
                .px(px(22.))
                .child(render_detail_body(selected, lang, cx))
                .child(render_args_panel(prompt_args, lang, has_args, cx))
                .child(render_messages_panel(
                    lang,
                    prompt_run,
                    prompt_validation,
                    cx,
                )),
        ),
    )
}

// ---------------------------------------------------------------------------
// Detail header — title (fixed bar) + body (scroll: description / empty-state)
// ---------------------------------------------------------------------------

/// Scroll-body header: the prompt description (the name moved to the fixed bar),
/// or the "select a prompt" empty-state when nothing is selected.
fn render_detail_body<E: 'static>(
    selected: Option<&PromptDefinition>,
    lang: Lang,
    cx: &mut Context<E>,
) -> impl IntoElement {
    let muted = cx.theme().muted_foreground;
    match selected {
        Some(prompt) => v_flex().id("prompt-detail-desc").pt_4().pb_2().child(
            div()
                .text_sm()
                .text_color(muted)
                .child(SharedString::from(prompt.description.clone())),
        ),
        None => v_flex()
            .id("prompt-detail-empty")
            .p_4()
            .items_center()
            .justify_center()
            .child(
                div()
                    .text_sm()
                    .text_color(muted)
                    .child(SharedString::from(tr("prompts.selectPrompt", lang))),
            ),
    }
}

/// Title for the fixed action bar: chat icon + name (mono). `flex_1`/`min_w(0)`
/// + truncate so a long name ellipsizes and the Generate button stays right.
fn render_detail_title<E: 'static>(
    prompt: &PromptDefinition,
    cx: &mut Context<E>,
) -> impl IntoElement {
    let mono = cx.theme().mono_font_family.clone();
    h_flex()
        .id("prompt-detail-title")
        .flex_1()
        .min_w(px(0.))
        .gap_2()
        .items_center()
        .child(Icon::new(IconName::Chat).size(IconSize::Sm))
        .child(
            div()
                .flex_1()
                .min_w(px(0.))
                .text_sm()
                .font_family(mono)
                .font_weight(FontWeight::SEMIBOLD)
                .text_ellipsis()
                .whitespace_nowrap()
                .child(SharedString::from(prompt.name.clone())),
        )
}

// ---------------------------------------------------------------------------
// Arguments panel — Field per PromptArgument (always string)
// ---------------------------------------------------------------------------

fn render_args_panel<E: 'static>(
    prompt_args: &[(PromptArgument, Entity<gpui_component::input::InputState>)],
    lang: Lang,
    has_args: bool,
    cx: &mut Context<E>,
) -> impl IntoElement {
    let title = SharedString::from(tr("prompts.args", lang));

    let body: gpui::AnyElement = if has_args {
        v_flex()
            .id("prompt-args-form")
            .w_full()
            .gap_2()
            .children(prompt_args.iter().enumerate().map(|(k, (arg, state))| {
                let hint = arg
                    .description
                    .clone()
                    .map(SharedString::from)
                    .unwrap_or_default();
                let mut f = Field::new(state)
                    .id(format!("prompt-arg-{k}"))
                    .label(SharedString::from(arg.name.clone()))
                    .hint(hint)
                    .mono(true);

                if arg.required == Some(true) {
                    f = f.required();
                }

                f.long().into_any_element()
            }))
            .into_any_element()
    } else {
        let muted = cx.theme().muted_foreground;
        v_flex()
            .id("prompt-args-empty")
            .p_4()
            .items_center()
            .justify_center()
            .child(
                div()
                    .text_sm()
                    .text_color(muted)
                    .child(SharedString::from(tr("prompts.noArgs", lang))),
            )
            .into_any_element()
    };

    Panel::new()
        .id("panel-prompt-args")
        .title(title)
        .icon(IconName::Bolt)
        .children(vec![body])
}

// ---------------------------------------------------------------------------
// Messages panel — renders across PromptRun lifecycle states
// ---------------------------------------------------------------------------

fn render_messages_panel<E: 'static>(
    lang: Lang,
    prompt_run: &PromptRun,
    prompt_validation: Option<&str>,
    cx: &mut Context<E>,
) -> impl IntoElement {
    let title = SharedString::from(tr("prompts.messages", lang));
    let colors = &cx.theme().colors;
    let j = cx.global::<JandiExt>();
    let mono = cx.theme().mono_font_family.clone();

    // --- Right slot: CopyButton (only when there are messages) ---
    let mut right = Vec::new();
    if let PromptRun::Messages(result) = prompt_run {
        let json_str = serde_json::to_string_pretty(result.as_ref())
            .unwrap_or_else(|_| String::from("serialization error"));
        right.push(
            CopyButton::new("btn-copy-messages", SharedString::from(json_str))
                .label(SharedString::from(tr("prompts.copyMessages", lang)))
                .copied_label(SharedString::from(tr("prompts.copied", lang)))
                .into_any_element(),
        );
    }

    let body: gpui::AnyElement = match prompt_run {
        PromptRun::Idle => {
            if let Some(msg) = prompt_validation {
                let muted = colors.muted_foreground;
                v_flex()
                    .id("prompt-messages-validation")
                    .p_3()
                    .child(
                        div()
                            .text_sm()
                            .text_color(muted)
                            .child(SharedString::from(msg.to_string())),
                    )
                    .into_any_element()
            } else {
                result_empty(tr("prompts.noMsgs", lang)).into_any_element()
            }
        }
        PromptRun::Building => v_flex()
            .id("prompt-building")
            .py(px(16.))
            .gap_2()
            .items_center()
            .justify_center()
            .child(Spinner::new())
            .child(
                div()
                    .text_sm()
                    .text_color(colors.muted_foreground)
                    .child(SharedString::from(tr("prompts.building", lang))),
            )
            .into_any_element(),
        PromptRun::Messages(result) => {
            if result.messages.is_empty() {
                result_empty(tr("prompts.noMsgs", lang)).into_any_element()
            } else {
                v_flex()
                    .id("prompt-messages-list")
                    .gap_3()
                    .children(result.messages.iter().enumerate().map(|(i, msg)| {
                        let role = role_label(&msg.role);
                        let body_text = match &msg.content {
                            PromptContent::Text { text } => text.as_str(),
                        };

                        v_flex()
                            .id(ElementId::from(format!("prompt-msg-{i}")))
                            .p_3()
                            .gap_2()
                            .rounded_lg()
                            .bg(colors.secondary) // surface-2
                            .border_1()
                            .border_color(colors.border)
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(Badge::new(role, BadgeKind::Role)),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .child(SharedString::from(body_text.to_string())),
                            )
                    }))
                    .into_any_element()
            }
        }
        PromptRun::Error(e) => v_flex()
            .id("prompt-error")
            .gap_2()
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(SharedString::from(tr("tools.errorProtocol", lang))),
            )
            .child(
                div()
                    .id("prompt-error-code")
                    .rounded(px(6.))
                    .p_3()
                    .bg(j.code_bg)
                    .child(
                        div()
                            .text_xs()
                            .font_family(mono)
                            .child(SharedString::from(e.clone())),
                    ),
            )
            .into_any_element(),
    };

    Panel::new()
        .id("panel-prompt-messages")
        .title(title)
        .icon(IconName::Chat)
        .right_children(right)
        .children(vec![body])
}

// ---------------------------------------------------------------------------
// Generate button — enabled, wired to the bridge
// ---------------------------------------------------------------------------

/// Generate button for the fixed action bar (Primary). Built only when a prompt
/// is selected; disabled while building or when no handler is wired.
fn render_generate_button(
    lang: Lang,
    prompt_run: &PromptRun,
    on_generate: Option<ClickHandler>,
) -> impl IntoElement {
    let is_running = matches!(prompt_run, PromptRun::Building);
    let label = if is_running {
        SharedString::from(tr("prompts.generating", lang))
    } else {
        SharedString::from(tr("prompts.generate", lang))
    };

    let btn = Button::new(label)
        .id("btn-generate")
        .variant(ButtonVariant::Primary)
        .size(ButtonSize::Md)
        .icon(IconName::Play);

    match (is_running, on_generate) {
        (false, Some(handler)) => btn.on_click(handler),
        _ => btn.disabled(),
    }
}
