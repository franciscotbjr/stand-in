//! EmptyState — centred empty-state panel with glyph, title, body, optional
//! numbered steps, and one primary action. Empty states *teach*, they never
//! dead-end.
//!
//! 1:1 with `data/EmptyState.prompt.md` + `.empty-*` / `.step-n` rules in
//! `data/data.css`.
//!
//! Anatomy:
//! - **Container:** height 100%, grid place-items center, padding 40.
//! - **Card:** max-w 460, text-align center.
//! - **Glyph 64×64:** radius 16, **the second (and last) legitimate gradient**
//!   (150deg surface-2 → surface — prohibition 5), border 1px `border`,
//!   icon 28px (IconSize::Lg) centred.
//! - **Title:** fs-title (20), weight 700, tracking tight.
//! - **Body:** 14px `text-2`, lh 1.6, max-w ∼40ch approximated as px(360).
//! - **Steps** (optional, 3 canonical): column gap 10, max-w 340, left-aligned.
//!   Each step = step-n pill (24×24, surface-2 bg, border-2 border, mono fs-sm
//!   700, number) + title (fs-md 600 `text`) + sub (fs-sm `text-3`, mt 2).
//! - **Action:** one slot (typically Button Primary; "teaches, never
//!   dead-ends" in the rustdoc).
//!
//! Small empties within panels use `.result-empty` (a muted single line), not
//! EmptyState — exposed as the free function [`result_empty`].
//!
//! ```ignore
//! use stand_in_mcp_explorer_ds::data::{EmptyState, EmptyStep};
//! use stand_in_mcp_explorer_ds::core::{Button, ButtonVariant, IconName};
//!
//! EmptyState::new("Pronto para inspecionar")
//!     .icon(IconName::Plug)
//!     .body("Escolha um servidor salvo ou ajuste a conex\u{e3}o na barra lateral.")
//!     .steps(vec![
//!         EmptyStep::new("1", "Escolha o transporte", "STDIO para locais; SSE/HTTP para remotos."),
//!         EmptyStep::new("2", "Conecte ao servidor", "npx -y @modelcontextprotocol/server-filesystem\u{2026}"),
//!         EmptyStep::new("3", "Teste tools, resources e prompts", "Formul\u{e1}rios gerados pelo schema."),
//!     ])
//!     .action(Button::new("Conectar agora").icon(IconName::Play).into_any_element());
//! ```

use gpui::{
    AnyElement, App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, Styled, Window, div, linear_color_stop, linear_gradient, px,
};
use gpui_component::{ActiveTheme as _, v_flex};

use crate::core::icon::{Icon, IconName, IconSize};
use crate::theme::colors::JandiExt;
use crate::theme::density::RADIUS_PILL;
use crate::theme::typography;

// ---------------------------------------------------------------------------
// EmptyStep
// ---------------------------------------------------------------------------

/// One numbered step inside an [`EmptyState`].
///
/// `n` is the step number (rendered in the step-n pill), `title` is the
/// heading (fs-md 600), `sub` is the muted description below it.
#[derive(Debug, Clone)]
pub struct EmptyStep {
    pub n: SharedString,
    pub title: SharedString,
    pub sub: SharedString,
}

impl EmptyStep {
    pub fn new(
        n: impl Into<SharedString>,
        title: impl Into<SharedString>,
        sub: impl Into<SharedString>,
    ) -> Self {
        Self {
            n: n.into(),
            title: title.into(),
            sub: sub.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// EmptyState
// ---------------------------------------------------------------------------

/// Centred empty-state panel that teaches the next step.
///
/// Always has a title; icon + body + steps + action are optional.
/// Three steps is canonical; zero steps for trivial empties
/// ("Nenhuma execu\u{e7}\u{e3}o ainda").
///
/// Small empties **inside** a Panel should use [`result_empty`] instead —
/// a single muted line, not the full EmptyState.
#[derive(IntoElement)]
pub struct EmptyState {
    title: SharedString,
    icon: Option<IconName>,
    body: Option<SharedString>,
    steps: Vec<EmptyStep>,
    action: Option<AnyElement>,
    id: ElementId,
}

impl EmptyState {
    /// Create an empty state with the given title.
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            icon: None,
            body: None,
            steps: Vec::new(),
            action: None,
            id: ElementId::from("empty-state"),
        }
    }

    /// Set the icon inside the 64×64 glyph (28px, IconSize::Lg).
    pub fn icon(mut self, name: IconName) -> Self {
        self.icon = Some(name);
        self
    }

    /// Set the short body text below the title.
    pub fn body(mut self, text: impl Into<SharedString>) -> Self {
        self.body = Some(text.into());
        self
    }

    /// Set the numbered steps. 3 is canonical; empty for trivial empties.
    pub fn steps(mut self, steps: Vec<EmptyStep>) -> Self {
        self.steps = steps;
        self
    }

    /// Set the single primary action (typically a Button).
    pub fn action(mut self, el: impl IntoElement) -> Self {
        self.action = Some(el.into_any_element());
        self
    }

    /// Set a stable element id.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }
}

impl RenderOnce for EmptyState {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = cx.theme();
        let colors = &t.colors;
        let ext = cx.global::<JandiExt>();

        // --- Glyph (64×64, gradient + border + centred icon) ---
        let mut glyph = div()
            .id("empty-glyph")
            .w(px(64.))
            .h(px(64.))
            .flex_none()
            .rounded(px(16.))
            .mb(px(20.))
            .mx_auto()
            // Canon: linear-gradient(150deg, var(--surface-2), var(--surface)).
            // surface-2 = colors.secondary; surface = colors.sidebar.
            .bg(linear_gradient(
                150.0,
                linear_color_stop(colors.secondary, 0.0),
                linear_color_stop(colors.sidebar, 1.0),
            ))
            .border_1()
            .border_color(colors.border)
            .flex()
            .items_center()
            .justify_center();

        if let Some(icon_name) = self.icon {
            glyph = glyph.child(
                Icon::new(icon_name)
                    .size(IconSize::Lg)
                    .color(colors.foreground),
            );
        }

        // --- Title ---
        let title_div = div()
            .id("empty-title")
            .text_size(px(typography::FS_TITLE))
            .font_weight(FontWeight::BOLD)
            .font_family(typography::sans_family())
            .mb(px(10.))
            .child(self.title.clone());

        // --- Card (max-w 460, text-center) ---
        // No `mx_auto` here: the card is a flex item of the outer container and
        // auto margins on the main axis fight `justify_center` (pushed the card
        // right — 028 Item #12). The outer flex_col + items_center centres it.
        let mut card = div()
            .id("empty-card")
            .max_w(px(460.))
            .child(glyph)
            .child(title_div);

        // --- Body ---
        if let Some(body) = self.body {
            card = card.child(
                div()
                    .id("empty-body")
                    .text_size(px(typography::FS_LG))
                    .text_color(colors.secondary_foreground) // text-2
                    .line_height(gpui::relative(1.6))
                    .max_w(px(360.)) // ∼40ch documented approximation
                    .mx_auto()
                    .mb(px(16.))
                    .child(body.clone()),
            );
        }

        // --- Steps ---
        if !self.steps.is_empty() {
            let mut steps_col = v_flex()
                .id("empty-steps")
                .gap(px(10.))
                .max_w(px(340.))
                .mx_auto();

            for step in &self.steps {
                let pill = div()
                    .id("step-n")
                    .w(px(24.))
                    .h(px(24.))
                    .flex_none()
                    .rounded(px(RADIUS_PILL))
                    .bg(colors.secondary) // surface-2
                    .border_1()
                    .border_color(ext.border_2)
                    .font_family(typography::mono_family())
                    .text_size(px(typography::FS_SM))
                    .font_weight(FontWeight::BOLD)
                    .text_color(colors.secondary_foreground)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(step.n.clone());

                let text_col = v_flex()
                    .id("step-text")
                    .gap(px(2.))
                    .child(
                        div()
                            .id("step-title")
                            .text_size(px(typography::FS_MD))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors.foreground)
                            .child(step.title.clone()),
                    )
                    .child(
                        div()
                            .id("step-sub")
                            .text_size(px(typography::FS_SM))
                            .text_color(colors.muted_foreground) // text-3
                            .child(step.sub.clone()),
                    );

                let row = div()
                    .id("empty-step")
                    .flex()
                    .gap(px(12.))
                    .items_start()
                    .child(pill)
                    .child(text_col);

                steps_col = steps_col.child(row);
            }

            card = card.child(steps_col);
        }

        // --- Action (wrapper adds top margin) ---
        if let Some(action) = self.action {
            card = card.child(div().id("empty-action").mt(px(24.)).mx_auto().child(action));
        }

        // --- Outer container (full size, centred both axes) ---
        // flex_col + items_center (horizontal) + justify_center (vertical) — the
        // proven center-a-block pattern (same as the notifications/history empty
        // states). 028 Item #12: a flex-row here let the card drift right.
        div()
            .id(self.id)
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .p(px(40.))
            .child(card)
    }
}

// ---------------------------------------------------------------------------
// result_empty — free helper
// ---------------------------------------------------------------------------

/// Render a muted single-line empty placeholder for use inside a Panel
/// (the `.result-empty` pattern).
///
/// Returns a `Div` with canonical padding and text size. The caller should
/// chain `.text_color(cx.theme().muted_foreground)` (text-3) for the
/// mode-appropriate muted colour — the helper does not set colour itself
/// because it cannot access the theme.
///
/// ```ignore
/// use crate::data::empty_state::result_empty;
///
/// result_empty("Nada por aqui.")
///     .text_color(cx.theme().muted_foreground);
/// ```
pub fn result_empty(text: impl Into<SharedString>) -> gpui::Div {
    div()
        .text_size(px(typography::FS_MD))
        .py(px(28.))
        .px(px(16.))
        .text_center()
        .font_family(typography::sans_family())
        .child(text.into())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_step_construction() {
        let step = EmptyStep::new("1", "Escolha", "um transporte");
        assert_eq!(step.n.as_ref(), "1");
        assert_eq!(step.title.as_ref(), "Escolha");
        assert_eq!(step.sub.as_ref(), "um transporte");
    }

    #[test]
    fn test_empty_state_defaults() {
        let es = EmptyState::new("T\u{ed}tulo");
        assert_eq!(es.title.as_ref(), "T\u{ed}tulo");
        assert!(es.icon.is_none());
        assert!(es.body.is_none());
        assert!(es.steps.is_empty());
        assert!(es.action.is_none());
        assert_eq!(es.id, ElementId::from("empty-state"));
    }

    #[test]
    fn test_empty_state_with_icon() {
        let es = EmptyState::new("T").icon(IconName::Plug);
        assert_eq!(es.icon, Some(IconName::Plug));
    }

    #[test]
    fn test_empty_state_with_body() {
        let es = EmptyState::new("T").body("Corpo do texto.");
        assert_eq!(es.body.as_deref(), Some("Corpo do texto."));
    }

    #[test]
    fn test_empty_state_with_steps() {
        let steps = vec![
            EmptyStep::new("1", "Passo 1", "Descri\u{e7}\u{e3}o 1"),
            EmptyStep::new("2", "Passo 2", "Descri\u{e7}\u{e3}o 2"),
            EmptyStep::new("3", "Passo 3", "Descri\u{e7}\u{e3}o 3"),
        ];
        let es = EmptyState::new("T").steps(steps);
        assert_eq!(es.steps.len(), 3);
    }

    #[test]
    fn test_empty_state_with_action() {
        let btn = div().child("OK");
        let es = EmptyState::new("T").action(btn);
        assert!(es.action.is_some());
    }

    #[test]
    fn test_empty_state_id_override() {
        let es = EmptyState::new("T").id("my-empty");
        assert_eq!(es.id, ElementId::from("my-empty"));
    }

    #[test]
    fn test_empty_state_zero_steps_canonical() {
        let es = EmptyState::new("Nenhuma execu\u{e7}\u{e3}o ainda");
        assert!(es.steps.is_empty());
    }

    #[test]
    fn test_empty_state_gradient_is_legitimate() {
        let es = EmptyState::new("T").icon(IconName::Plug);
        assert!(es.icon.is_some());
        // The gradient (prohibition-5-approved #2) is applied in render —
        // this test just confirms the component carries an icon.
    }

    #[test]
    fn test_result_empty_does_not_panic() {
        let _ = result_empty("Nada por aqui.");
    }
}
