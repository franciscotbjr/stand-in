//! Authorization panel — floating overlay to the right.
//!
//! Renders on top of the app when `auth_panel_open`. The overlay is a
//! transparent click-catcher (no scrim — it's a floating panel, the app
//! remains visible) + a card anchored to the right. The card uses `.occlude()`
//! so clicks inside don't bubble to the catcher. Close: X button or click outside.
//!
//! Body: Select (method) + conditional fields per method + scrollable body
//! (Dialog pattern for OAuth's 5+ fields).

use crate::app::i18n::{Lang, tr};
use crate::bars::sidebar::auth_state::{self, AuthDraft, AuthMethod, AuthStatus};
use gpui::{
    App, ElementId, FontWeight, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, SharedString, Styled, Window, div, prelude::FluentBuilder, px, relative,
};
use gpui_component::scroll::ScrollableElement as _;
use gpui_component::{ActiveTheme as _, ThemeMode, v_flex};
use stand_in_mcp_explorer_ds::core::button::{Button, ButtonVariant, ClickHandler};
use stand_in_mcp_explorer_ds::core::copy_button::CopyButton;
use stand_in_mcp_explorer_ds::core::icon::IconName;
use stand_in_mcp_explorer_ds::core::icon_button::IconButton;
use stand_in_mcp_explorer_ds::forms::field::Field;
use stand_in_mcp_explorer_ds::forms::select::Select;
use stand_in_mcp_explorer_ds::theme::colors::JandiExt;
use stand_in_mcp_explorer_ds::theme::density::RADIUS_CARD;

/// Uniform width of every control inside the panel card (Select + Fields +
/// redirect box). The card is `w(340)`; minus the 1px borders and the body's
/// 18px horizontal padding each side → 302. A single definite width keeps every
/// control aligned regardless of `w_full` flex-resolution quirks (028 #16).
const FIELD_W: f32 = 302.0;

/// Handler for click-outside-to-close on the auth overlay catcher.
pub type ClickOutsideHandler = Box<dyn Fn(&mut Window, &mut App) + 'static>;

#[allow(clippy::too_many_arguments)]
pub fn render_auth_panel(
    draft: &AuthDraft,
    lang: Lang,
    method_change_handler: Option<stand_in_mcp_explorer_ds::forms::select::SelectHandler>,
    on_authorize: ClickHandler,
    on_close_x: ClickHandler,
    on_close_save: ClickHandler,
    on_click_outside: ClickOutsideHandler,
    _window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let t = cx.theme().clone();
    let mode = t.mode;
    let colors = t.colors;
    let ext = cx.global::<JandiExt>().clone();
    drop(t);

    let method_ix = draft.method.selected_ix();

    let method_opts: Vec<(SharedString, SharedString)> = AuthMethod::ALL
        .iter()
        .map(|m| {
            (
                SharedString::from(m.label_key()),
                SharedString::from(tr(m.label_key(), lang)),
            )
        })
        .collect();

    let redirect = auth_state::redirect_uri();

    // Click-catcher layer: transparent, absolute, fills window.
    // Click outside the card → close.
    div()
        .id("auth-overlay")
        .absolute()
        .inset_0()
        .flex()
        .justify_end()
        .items_start()
        .on_mouse_down(
            MouseButton::Left,
            move |_ev: &MouseDownEvent, window: &mut Window, app: &mut App| {
                // The card `.occlude()` prevents inner clicks from reaching here.
                // This handler fires only on clicks outside the card → close.
                on_click_outside(window, app);
            },
        )
        .child(
            div()
                .id("auth-panel-wrapper")
                .pt(px(72.))
                .pr(px(12.))
                .child(
                    div()
                        .id("auth-panel")
                        .occlude()
                        .flex()
                        .flex_col()
                        .w(px(340.))
                        .max_h(relative(0.85))
                        .bg(colors.secondary)
                        .border_1()
                        .border_color(colors.border)
                        .rounded(px(RADIUS_CARD))
                        .shadow_lg()
                        .child(auth_header(lang, on_close_x, &colors, &ext))
                        .child(auth_body(
                            draft,
                            lang,
                            method_ix,
                            method_opts,
                            method_change_handler,
                            on_authorize,
                            mode,
                            &colors,
                            &ext,
                            &redirect,
                            cx,
                        ))
                        .child(auth_footer(lang, on_close_save, &colors)),
                ),
        )
}

// ---------------------------------------------------------------------------
// Header
// ---------------------------------------------------------------------------

fn auth_header(
    lang: Lang,
    on_close: ClickHandler,
    colors: &gpui_component::ThemeColor,
    _ext: &JandiExt,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .p(px(18.))
        .pb(px(14.))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(4.))
                .child(
                    div()
                        .text_size(px(15.))
                        .font_weight(FontWeight::BOLD)
                        .text_color(colors.foreground)
                        .child(tr("auth.title", lang)),
                )
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(colors.secondary_foreground)
                        .child(tr("auth.subtitle", lang)),
                ),
        )
        .child(
            IconButton::new(IconName::X, tr("tweaks.close", lang))
                .id("auth-close")
                .on_click_boxed(on_close),
        )
}

// ---------------------------------------------------------------------------
// Scrollable body (Dialog pattern)
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn auth_body(
    draft: &AuthDraft,
    lang: Lang,
    method_ix: usize,
    method_opts: Vec<(SharedString, SharedString)>,
    method_change_handler: Option<stand_in_mcp_explorer_ds::forms::select::SelectHandler>,
    on_authorize: ClickHandler,
    _mode: ThemeMode,
    colors: &gpui_component::ThemeColor,
    ext: &JandiExt,
    redirect: &str,
    cx: &mut App,
) -> impl IntoElement {
    div().flex_1().overflow_hidden().child(
        div()
            .size_full()
            .overflow_y_scrollbar()
            .px(px(18.))
            .child(auth_body_inner(
                draft,
                lang,
                method_ix,
                method_opts,
                method_change_handler,
                on_authorize,
                colors,
                ext,
                redirect,
                cx,
            )),
    )
}

#[allow(clippy::too_many_arguments)]
fn auth_body_inner(
    draft: &AuthDraft,
    lang: Lang,
    method_ix: usize,
    method_opts: Vec<(SharedString, SharedString)>,
    method_change_handler: Option<stand_in_mcp_explorer_ds::forms::select::SelectHandler>,
    on_authorize: ClickHandler,
    colors: &gpui_component::ThemeColor,
    _ext: &JandiExt,
    redirect: &str,
    cx: &mut App,
) -> impl IntoElement {
    let mut sel = Select::new("auth-method", method_opts, method_ix).width(px(FIELD_W));
    if let Some(ref h) = method_change_handler {
        sel = sel.on_change({
            let h = h.clone();
            move |ix, val, w, app| h(ix, val, w, app)
        });
    }

    let method = draft.method;

    v_flex()
        .id("auth-body-inner")
        .w_full()
        .gap(px(12.))
        .child(sel)
        .child(render_method_fields(
            draft,
            method,
            lang,
            on_authorize,
            colors,
            redirect,
            cx,
        ))
}

fn render_method_fields(
    draft: &AuthDraft,
    method: AuthMethod,
    lang: Lang,
    on_authorize: ClickHandler,
    colors: &gpui_component::ThemeColor,
    redirect: &str,
    cx: &mut App,
) -> impl IntoElement {
    match method {
        AuthMethod::NoAuth => v_flex()
            .id("auth-fields-none")
            .w_full()
            .gap(px(8.))
            .child(
                div()
                    .text_size(px(11.5))
                    .text_color(colors.muted_foreground)
                    .line_height(relative(1.4))
                    .child(tr("auth.noneHint", lang)),
            )
            .into_any_element(),

        AuthMethod::Basic => v_flex()
            .id("auth-fields-basic")
            .w_full()
            .gap(px(12.))
            .child(
                Field::new(&draft.basic_username)
                    .label(tr("auth.username", lang))
                    .hint(tr("auth.usernamePh", lang))
                    .mono(false)
                    .width(px(FIELD_W))
                    .id(ElementId::from("auth-username")),
            )
            .child(
                Field::new(&draft.basic_password)
                    .label(tr("auth.password", lang))
                    .hint(tr("auth.passwordPh", lang))
                    .secret()
                    .width(px(FIELD_W))
                    .id(ElementId::from("auth-password")),
            )
            .child(
                div()
                    .text_size(px(11.5))
                    .text_color(colors.muted_foreground)
                    .line_height(relative(1.4))
                    .child(tr("auth.basicHint", lang)),
            )
            .into_any_element(),

        AuthMethod::Bearer => v_flex()
            .id("auth-fields-bearer")
            .w_full()
            .gap(px(12.))
            .child(
                Field::new(&draft.bearer_token)
                    .label(tr("auth.token", lang))
                    .hint(tr("auth.tokenPh", lang))
                    .secret()
                    .width(px(FIELD_W))
                    .id(ElementId::from("auth-token")),
            )
            .child(
                div()
                    .text_size(px(11.5))
                    .text_color(colors.muted_foreground)
                    .line_height(relative(1.4))
                    .child(tr("auth.bearerHint", lang)),
            )
            .into_any_element(),

        AuthMethod::OAuth => {
            let is_authorizing = draft.oauth_status == AuthStatus::Authorizing;
            let authorize_label = if is_authorizing {
                tr("auth.authorizing", lang)
            } else {
                tr("auth.authorize", lang)
            };
            let variant = if is_authorizing {
                ButtonVariant::Ghost
            } else {
                ButtonVariant::Primary
            };

            // Status line below the button
            let status_line = match draft.oauth_status {
                AuthStatus::Idle => None,
                AuthStatus::Authorizing => Some((
                    colors.muted_foreground,
                    tr("auth.authorizing", lang).to_string(),
                )),
                AuthStatus::Authorized => {
                    let detail = crate::bars::sidebar::auth_state::format_oauth_expiry(
                        &draft.oauth_tokens,
                        lang,
                    );
                    Some((colors.success, detail))
                }
                AuthStatus::Failed => {
                    Some((colors.danger, draft.oauth_error.clone().unwrap_or_default()))
                }
            };

            v_flex()
                .id("auth-fields-oauth")
                .w_full()
                .gap(px(12.))
                .child(
                    Field::new(&draft.oauth_client_id)
                        .label(tr("auth.clientId", lang))
                        .hint(tr("auth.clientIdPh", lang))
                        .width(px(FIELD_W))
                        .id(ElementId::from("auth-client-id")),
                )
                .child(
                    Field::new(&draft.oauth_auth_url)
                        .label(tr("auth.authUrl", lang))
                        .width(px(FIELD_W))
                        .id(ElementId::from("auth-auth-url")),
                )
                .child(
                    Field::new(&draft.oauth_token_url)
                        .label(tr("auth.tokenUrl", lang))
                        .width(px(FIELD_W))
                        .id(ElementId::from("auth-token-url")),
                )
                .child(
                    Field::new(&draft.oauth_scopes)
                        .label(tr("auth.scope", lang))
                        .hint(tr("auth.scopePh", lang))
                        .width(px(FIELD_W))
                        .id(ElementId::from("auth-scopes")),
                )
                .child(render_redirect_uri(redirect, lang, colors, cx))
                .child(
                    Button::new(authorize_label)
                        .variant(variant)
                        .id("auth-authorize")
                        .on_click({
                            let h = on_authorize;
                            move |ev, w, cx| h(ev, w, cx)
                        }),
                )
                .when_some(status_line, |col, (color, msg)| {
                    col.child(div().text_size(px(12.)).text_color(color).child(msg))
                })
                .child(
                    div()
                        .text_size(px(11.5))
                        .text_color(colors.muted_foreground)
                        .line_height(relative(1.4))
                        .child(tr("auth.oauthHint", lang)),
                )
                .into_any_element()
        }
    }
}

// ---------------------------------------------------------------------------
// Redirect URI (OAuth, D7) — read-only box + CopyButton
// ---------------------------------------------------------------------------

fn render_redirect_uri(
    redirect: &str,
    lang: Lang,
    colors: &gpui_component::ThemeColor,
    cx: &mut App,
) -> impl IntoElement {
    let redirect_owned = redirect.to_string();
    let mono_font = cx.theme().mono_font_family.clone();

    v_flex()
        .id("auth-redirect-section")
        .gap(px(6.))
        .child(
            div()
                .text_size(px(12.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(colors.secondary_foreground)
                .child(tr("auth.redirectUri", lang)),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(6.))
                .child(
                    div()
                        .id("auth-redirect")
                        .w(px(FIELD_W))
                        .flex()
                        .items_center()
                        .bg(colors.background)
                        .border_1()
                        .border_color(colors.border)
                        .rounded(px(stand_in_mcp_explorer_ds::theme::density::RADIUS_INPUT))
                        .px(px(11.))
                        .py(px(9.))
                        .text_size(px(13.))
                        .text_color(colors.foreground)
                        .font_family(mono_font)
                        .child(redirect_owned.clone()),
                )
                .child(
                    CopyButton::new("auth-redirect-copy", redirect_owned)
                        .label(tr("common.copy", lang))
                        .copied_label(tr("common.copied", lang)),
                )
                .child(
                    div()
                        .text_size(px(11.5))
                        .text_color(colors.muted_foreground)
                        .line_height(relative(1.4))
                        .child(tr("auth.redirectUriHint", lang)),
                ),
        )
}

// ---------------------------------------------------------------------------
// Footer
// ---------------------------------------------------------------------------

fn auth_footer(
    lang: Lang,
    on_close: ClickHandler,
    colors: &gpui_component::ThemeColor,
) -> impl IntoElement {
    div()
        .flex()
        .justify_end()
        .p(px(18.))
        .pt(px(14.))
        .border_t_1()
        .border_color(colors.border)
        .child(
            Button::new(tr("auth.done", lang))
                .variant(ButtonVariant::Primary)
                .id("auth-save")
                .on_click(move |ev, w, cx| on_close(ev, w, cx)),
        )
}
