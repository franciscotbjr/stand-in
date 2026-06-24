//! Parameter form — builds `Field` components from parsed `ParamField`s
//! paired with caller-owned `InputState` entities.
//!
//! Used by the detail column to render the "Parameters" panel content.

use gpui::InteractiveElement;
use gpui::{Context, Entity, IntoElement, ParentElement, SharedString, Styled, Window};
use gpui_component::{ActiveTheme as _, v_flex};

use crate::app::i18n::Lang;
use crate::app::i18n::tr;

use super::schema::ParamField;
use stand_in_mcp_explorer_ds::forms::Field;

/// Build the parameters form content (list of Fields).
///
/// Each field is labelled with the parameter name (mono), a `*` for required,
/// and a hint showing type + description. Empty vec → "sem parâmetros" line.
pub fn render_params_form<E: 'static>(
    params: &[(ParamField, Entity<gpui_component::input::InputState>)],
    lang: Lang,
    _window: &mut Window,
    cx: &mut Context<E>,
) -> impl IntoElement {
    if params.is_empty() {
        let muted = cx.theme().muted_foreground;
        return v_flex()
            .id("params-empty")
            .p_4()
            .items_center()
            .justify_center()
            .child(
                gpui::div()
                    .text_sm()
                    .text_color(muted)
                    .child(SharedString::from(tr("tools.noParams", lang))),
            )
            .into_any_element();
    }

    v_flex()
        .id("params-form")
        .w_full()
        .gap_2()
        .children(params.iter().enumerate().map(|(k, (field, state))| {
            let hint = if field.description.is_empty() {
                field.type_str.clone()
            } else {
                format!("{} — {}", field.type_str, field.description)
            };

            let mut f = Field::new(state)
                .id(format!("param-{k}"))
                .label(SharedString::from(field.name.clone()))
                .hint(SharedString::from(hint));

            if field.required {
                f = f.required();
            }

            if field.type_str == "string" {
                f = f.long();
            }

            f.mono(true).into_any_element()
        }))
        .into_any_element()
}
