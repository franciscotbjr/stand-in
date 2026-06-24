//! LogRow — terminal-style mono log line in a fixed grid.
//!
//! 1:1 with `data/LogRow.prompt.md` + `.log` / `.log-row` / `.log-*` rules
//! in `data/data.css`. Wrap several in a `div` (the `.log` container) and
//! compose the `.notif-toolbar` sticky header at the caller level (026).
//!
//! Anatomy (grid): time **86px** · level **74px** · message **1fr**, gap 12,
//! padding 10×16, border-bottom 1px `border`, baseline-aligned. Hover bg
//! `surface` (via `colors.secondary`). All text in mono 12.5, lh 1.55.
//!
//! Level colours (FIXED — never reinterpret):
//! - Info → OBY · Ok → ok · Warn → warn · Error → err · Debug → text-3
//!
//! Voice: lowercase everywhere, machine-style — the caller supplies the text
//! as-is; the component only styles it. Feeds: most recent on top (caller
//! ordering).
//!
//! ```ignore
//! use stand_in_mcp_explorer_ds::data::{LogRow, LogLevel};
//!
//! v_flex().child(LogRow::new("14:02:31", LogLevel::Info, "conectando via stdio…"));
//! v_flex().child(LogRow::new("14:02:32", LogLevel::Ok, "conectado a server-filesystem (57ms)"));
//! ```

use gpui::{
    App, ElementId, FontWeight, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Window, px,
};
use gpui_component::{ActiveTheme as _, h_flex};

use crate::theme::palette;
use crate::theme::typography;

// ---------------------------------------------------------------------------
// LogLevel
// ---------------------------------------------------------------------------

/// Semantic log level with a fixed colour mapping.
///
/// **Fixed semantics (never reinterpret):**
/// `Info` = OBY · `Ok` = ok · `Warn` = warn · `Error` = err · `Debug` = text-3.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Ok,
    Warn,
    Error,
    Debug,
}

impl LogLevel {
    /// The semantic colour for this level (one of the 5 fixed roles).
    pub fn color(self) -> Hsla {
        match self {
            LogLevel::Info => palette::OBY,
            LogLevel::Ok => palette::OK,
            LogLevel::Warn => palette::WARN,
            LogLevel::Error => palette::ERR,
            LogLevel::Debug => palette::dark::TEXT_3,
        }
    }
}

// ---------------------------------------------------------------------------
// LogRow
// ---------------------------------------------------------------------------

/// Terminal-style mono log line — time · level · message.
///
/// The caller passes the level label as-is (typically lowercase "info" /
/// "ok" / …); the component only colours it. Use several in a vertical
/// stack as the `.log` container; the sticky `.notif-toolbar` is the
/// caller's responsibility (026).
#[derive(IntoElement)]
pub struct LogRow {
    time: SharedString,
    level_label: SharedString,
    level: LogLevel,
    message: SharedString,
    id: ElementId,
}

impl LogRow {
    /// Create a log row. `level_label` is the display text for the level
    /// (e.g. "info", "ok"), `level` is the semantic kind that drives colour.
    pub fn new(
        time: impl Into<SharedString>,
        level: LogLevel,
        level_label: impl Into<SharedString>,
        message: impl Into<SharedString>,
    ) -> Self {
        Self {
            time: time.into(),
            level_label: level_label.into(),
            level,
            message: message.into(),
            id: ElementId::from("log-row"),
        }
    }

    /// Set a stable element id.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }
}

impl RenderOnce for LogRow {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = cx.theme();
        let colors = &t.colors;
        let lvl_color = self.level.color();

        h_flex()
            .id(self.id)
            .gap(px(12.))
            .px(px(16.))
            .py(px(10.))
            .border_b_1()
            .border_color(colors.border)
            .items_baseline()
            .font_family(typography::mono_family())
            .text_size(px(12.5))
            .line_height(gpui::relative(1.55))
            .hover(|h| h.bg(colors.secondary))
            .w_full()
            // Time (fixed 86px)
            .child(
                gpui::div()
                    .w(px(86.))
                    .flex_none()
                    .text_color(colors.muted_foreground)
                    .child(self.time.clone()),
            )
            // Level (fixed 74px, bold)
            .child(
                gpui::div()
                    .w(px(74.))
                    .flex_none()
                    .font_weight(FontWeight::BOLD)
                    .text_size(px(typography::FS_2XS))
                    .text_color(lvl_color)
                    .child(self.level_label),
            )
            // Message (flex, word-break)
            .child(
                gpui::div()
                    .flex_1()
                    .min_w(px(0.))
                    .text_color(colors.foreground)
                    .child(self.message),
            )
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_colors_are_distinct() {
        let levels = &[
            LogLevel::Info,
            LogLevel::Ok,
            LogLevel::Warn,
            LogLevel::Error,
            LogLevel::Debug,
        ];
        for i in 0..levels.len() {
            for j in i + 1..levels.len() {
                assert_ne!(
                    levels[i].color(),
                    levels[j].color(),
                    "{:?} and {:?} have the same colour",
                    levels[i],
                    levels[j],
                );
            }
        }
    }

    #[test]
    fn test_log_level_info_is_oby() {
        assert_eq!(LogLevel::Info.color(), palette::OBY);
    }

    #[test]
    fn test_log_level_ok_is_ok() {
        assert_eq!(LogLevel::Ok.color(), palette::OK);
    }

    #[test]
    fn test_log_level_warn_is_warn() {
        assert_eq!(LogLevel::Warn.color(), palette::WARN);
    }

    #[test]
    fn test_log_level_error_is_err() {
        assert_eq!(LogLevel::Error.color(), palette::ERR);
    }

    #[test]
    fn test_log_level_debug_is_text_3() {
        assert_eq!(LogLevel::Debug.color(), palette::dark::TEXT_3);
    }

    #[test]
    fn test_log_row_construction() {
        let row = LogRow::new("14:02:31", LogLevel::Info, "info", "conectando…");
        assert_eq!(row.time.as_ref(), "14:02:31");
        assert_eq!(row.level, LogLevel::Info);
        assert_eq!(row.level_label.as_ref(), "info");
        assert_eq!(row.message.as_ref(), "conectando…");
        assert_eq!(row.id, ElementId::from("log-row"));
    }

    #[test]
    fn test_log_row_id_setter() {
        let row = LogRow::new("14:02:32", LogLevel::Ok, "ok", "pronto.").id("notif-ok");
        assert_eq!(row.id, ElementId::from("notif-ok"));
    }

    #[test]
    fn test_log_level_eq() {
        assert_eq!(LogLevel::Info, LogLevel::Info);
        assert_ne!(LogLevel::Info, LogLevel::Ok);
    }
}
