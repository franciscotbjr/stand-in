//! JsonView — syntax-highlighted JSON renderer with a `.code` surface.
//!
//! 1:1 with `data/JsonView.prompt.md` + `.code` / `.tok-*` rules in `data/data.css`.
//! The canonical JSON display surface for the MCP Explorer.
//!
//! ## Investigation — GPUI pinned-revision multi-color text rendering
//!
//! The gpui pin (`3f5705b9`) provides `gpui::StyledText` (in `elements/text.rs`),
//! `gpui::HighlightStyle` (in `style.rs`), and `gpui::TextRun` (in
//! `text_system.rs`). All three are re-exported at the crate root via
//! `pub use elements::*;` / `pub use style::*;` / `pub use text_system::*;`.
//!
//! `StyledText` supports two highlight approaches:
//! - `with_highlights(highlights)` — delayed `(Range<usize>, HighlightStyle)`
//!   pairs, resolved at layout time against `window.text_style()`. This is the
//!   idiomatic path for syntax highlighting.
//! - `with_runs(runs)` — pre-computed `Vec<TextRun>` with explicit font/color
//!   per run.
//!
//! `HighlightStyle::color` accepts `Hsla` (via `From<Hsla>`). Container text
//! style (font_family, font_size, line_height, white_space) cascades through
//! the Window's `text_style_stack` to the `StyledText` default style.
//!
//! **Verdict:** compose — `StyledText` is the pinned gpui's built-in mechanism
//! for multi-colored inline text, and `with_highlights` is its intended API.
//! No hand-rolled span-per-token layout needed. The gpui-component highlighter
//! is heavier and coupled to its own token semantics — not applicable here.
//!
//! ## Surface
//!
//! Anatomy: JetBrains Mono 12.5, lh 1.6, bg `code_bg`, border 1px `border`,
//! radius `RADIUS_INPUT` (8), padding 13×15, pre-wrap + word-break. No margins.
//! Shadow prohibited (in-flow card — prohibition 4).
//!
//! ## Colors (FIXED — never invent)
//!
//! Key → `tok_key` (BRISA), Str → `tok_str`, Num → `tok_num`,
//! Bool/Null → `tok_bool`, Punc → `text-3`. All via JandiExt/theme.
//!
//! ## Usage
//!
//! ```ignore
//! use stand_in_mcp_explorer_ds::data::JsonView;
//!
//! JsonView::new(pretty_json).id("my-result");
//! JsonView::plain(text).id("plain-view");
//! ```

use std::ops::Range;

use gpui::{
    ElementId, HighlightStyle, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, StyledText, px,
};
use gpui_component::ActiveTheme as _;

use crate::theme::colors::JandiExt;
use crate::theme::density;
use crate::theme::typography;

use super::json_tokens::{self, JsonToken};

/// Syntax-highlighted JSON code block.
///
/// The caller pretty-prints/stringifies the JSON; JsonView only colors it.
/// Use `JsonView::plain(text)` for unhighlighted monospace text with the same
/// `.code` surface (logs, error traces, raw output).
#[derive(IntoElement)]
pub struct JsonView {
    text: SharedString,
    tokenize: bool,
    id: ElementId,
}

impl JsonView {
    /// Create a syntax-highlighted JSON view.
    ///
    /// The input must be a valid (or mostly valid) JSON string. The tokenizer
    /// is tolerant — malformed content degrades to uncolored text rather than
    /// panicking. To render non-JSON monospace text with the same `.code`
    /// surface, use [`JsonView::plain`].
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            tokenize: true,
            id: ElementId::Name("json-view".into()),
        }
    }

    /// Create an unhighlighted plain-text code block.
    ///
    /// Same `.code` surface (mono 12.5, code_bg, pre-wrap) with a single text
    /// color — no syntax coloring.
    pub fn plain(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            tokenize: false,
            id: ElementId::Name("plain-code".into()),
        }
    }

    /// Set a stable element id for testing / accessibility.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }
}

impl Default for JsonView {
    fn default() -> Self {
        Self::new("{}")
    }
}

impl RenderOnce for JsonView {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let j = cx.global::<JandiExt>();
        let t = cx.theme();

        let container = gpui::div()
            .id(self.id)
            .font_family(typography::mono_family())
            .text_size(px(12.5))
            .line_height(gpui::relative(1.6))
            .bg(j.code_bg)
            .border_1()
            .border_color(t.border)
            .rounded(px(density::RADIUS_INPUT))
            .px(px(15.0))
            .py(px(13.0));

        if self.tokenize {
            let highlights = token_highlights(&self.text, j, t.muted_foreground);
            container.child(StyledText::new(self.text.clone()).with_highlights(highlights))
        } else {
            container.child(self.text.clone())
        }
    }
}

fn token_highlights(text: &str, j: &JandiExt, text_3: Hsla) -> Vec<(Range<usize>, HighlightStyle)> {
    json_tokens::tokenize(text)
        .into_iter()
        .map(|(range, kind)| {
            let color = match kind {
                JsonToken::Key => j.tok_key,
                JsonToken::Str => j.tok_str,
                JsonToken::Num => j.tok_num,
                JsonToken::Bool => j.tok_bool,
                JsonToken::Punc => text_3,
            };
            (range, HighlightStyle::from(color))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_does_not_panic() {
        let view = JsonView::plain("some text");
        assert!(!view.tokenize);
    }

    #[test]
    fn test_new_tokenizes_by_default() {
        let view = JsonView::new("{}");
        assert!(view.tokenize);
    }

    #[test]
    fn test_id_setter() {
        let view = JsonView::new("{}").id("my-id");
        assert_eq!(view.id, ElementId::Name("my-id".into()));
    }
}
