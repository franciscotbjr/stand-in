//! Data-extra section — LogRow + EmptyState + HintBar.
//!
//! LogRow: 5 lines covering the full level vocabulary (info / ok / warn /
//! error / debug) in the `.log` container, proving each colour is visible.
//!
//! EmptyState: canonical onboarding example with plug glyph, title, body,
//! 3 steps (choose transport / connect / test), and a primary action button.
//!
//! HintBar: guided-mode didactic with bold terms inline, proving the
//! 10% oby tint and the text slot with children.

use gpui::{FontWeight, IntoElement, ParentElement, Styled, div};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{ActiveTheme as _, v_flex};
use stand_in_mcp_explorer_ds::core::{Button, ButtonVariant, IconName};
use stand_in_mcp_explorer_ds::data::{
    EmptyState, HintBar, LogLevel, LogRow, empty_state::EmptyStep,
};

use super::util::{px, section_body, section_label};
use crate::shell::GalleryShell;

pub fn render_data_extra(
    _state: &str,
    _mode: &str,
    _this: &GalleryShell,
    cx: &mut gpui::Context<GalleryShell>,
) -> gpui::AnyElement {
    let t = cx.theme().clone();
    let mono = t.mono_font_family.clone();

    // --- LogRow: 5 lines covering all levels ---
    let log_block = v_flex()
        .w_full()
        .child(LogRow::new(
            "14:02:31",
            LogLevel::Info,
            "info",
            "conectando via STDIO\u{2026}",
        ))
        .child(LogRow::new(
            "14:02:32",
            LogLevel::Ok,
            "ok",
            "conectado a stand-in-reference (57ms)",
        ))
        .child(LogRow::new(
            "14:02:33",
            LogLevel::Warn,
            "warn",
            "ferramenta \u{2018}legacy\u{2019} marcada como obsoleta",
        ))
        .child(LogRow::new(
            "14:02:34",
            LogLevel::Error,
            "error",
            "falha ao ler recurso: permiss\u{e3}o negada (EACCES)",
        ))
        .child(LogRow::new(
            "14:02:35",
            LogLevel::Debug,
            "debug",
            "raw payload: {\u{201c}capabilities\u{201d}:{\u{201c}tools\u{201d}:3}}",
        ));

    // --- EmptyState: canonical onboarding example ---
    let empty = EmptyState::new("Pronto para inspecionar")
        .icon(IconName::Plug)
        .body("Escolha um servidor salvo ou ajuste a conex\u{e3}o na barra lateral.")
        .steps(vec![
            EmptyStep::new(
                "1",
                "Escolha o transporte",
                "STDIO para servidores locais; SSE/HTTP para remotos.",
            ),
            EmptyStep::new(
                "2",
                "Conecte ao servidor",
                "npx -y @modelcontextprotocol/server-filesystem\u{2026}",
            ),
            EmptyStep::new(
                "3",
                "Teste tools, resources e prompts",
                "Formul\u{e1}rios gerados pelo schema.",
            ),
        ])
        .action(
            Button::new("Conectar agora")
                .variant(ButtonVariant::Primary)
                .icon(IconName::Play)
                .id("empty-action-btn"),
        );

    // --- HintBar: didactic text with bold highlights ---
    let bold_children: Vec<gpui::AnyElement> = vec![
        div().font_weight(FontWeight::BOLD).child("Tools").into_any_element(),
        div().child(" s\u{e3}o fun\u{e7}\u{f5}es que o servidor exp\u{f5}e. Preencha os par\u{e2}metros e clique em ").into_any_element(),
        div().font_weight(FontWeight::BOLD).child("Executar").into_any_element(),
        div().child(".").into_any_element(),
    ];

    let hint = HintBar::new().children(bold_children);

    v_flex()
        .flex_1()
        .min_w(px(0.))
        .h_full()
        .overflow_y_scrollbar()
        .child(
            section_body()
                // LogRow — 5 lines, all levels
                .child(section_label(
                    "LogRow \u{b7} 5 n\u{ed}veis (info / ok / warn / error / debug)",
                    &t,
                    &mono,
                ))
                .child(
                    div()
                        .mt(px(8.))
                        .rounded(px(8.))
                        .border_1()
                        .border_color(t.border)
                        .bg(t.background)
                        .overflow_hidden()
                        .child(log_block),
                )
                // EmptyState — canonical example
                .child(section_label(
                    "EmptyState \u{b7} exemplo can\u{f4}nico (glyph + steps + action)",
                    &t,
                    &mono,
                ))
                .child(
                    div()
                        .mt(px(8.))
                        .rounded(px(8.))
                        .border_1()
                        .border_color(t.border)
                        .bg(t.background)
                        .overflow_hidden()
                        .h(px(520.))
                        .child(empty),
                )
                // HintBar — guided mode
                .child(section_label(
                    "HintBar \u{b7} modo guiado (oby-10% + destaques em negrito)",
                    &t,
                    &mono,
                ))
                .child(div().mt(px(8.)).px(px(14.)).child(hint)),
        )
        .into_any_element()
}
