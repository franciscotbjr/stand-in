//! Data components — list→detail pattern primitives.
//!
//! Panel: bordered surface container with uppercase header + action slot.
//! ListItem: interactive data row with mono identifier + 2-line description.
//! ListSearch: sticky search input with magnifier (NO shadow — O-001
//!   lesson). Filtro imediato via InputEvent::Change.
//! PresetCard: selectable configuration card with mono name + muted badge +
//!   short description. Selection = oby border + 2px composite ring
//!   (NOT shadow_* — the M4 halo technique). NEVER swap with ListItem
//!   (filete = navigation, ring = configuration choice).
//! JsonView: syntax-highlighted JSON code block with .code surface
//!   (mono 12.5, bg code_bg, pre-wrap). Uses `gpui::StyledText` +
//!   `with_highlights` — the pinned GPUI's built-in multi-color text
//!   mechanism. Also `JsonView::plain` for unhighlighted monospace text.
//!
//! Filled: M13 (Panel + ListItem), M14 (ListSearch + PresetCard),
//! M15 (JsonView).
//! Also: LogRow (terminal-style mono log line with 5 fixed level colours),
//! EmptyState (centred empty panel — glyph, title, steps, one action),
//! HintBar (guided-mode hint with 10% oby tint — never permanent).
//!
//! Filled: M13 (Panel + ListItem), M14 (ListSearch + PresetCard),
//! M15 (JsonView), M16 (LogRow + EmptyState + HintBar).

pub mod empty_state;
pub mod hint_bar;
pub mod json_tokens;
pub mod json_view;
pub mod list_item;
pub mod list_search;
pub mod log_row;
pub mod panel;
pub mod preset_card;

pub use empty_state::{EmptyState, result_empty};
pub use hint_bar::HintBar;
pub use json_view::JsonView;
pub use list_item::{
    DESC_RESERVED_H, LIST_ROW_HEIGHT, ListItem, ListScrollHoverSuppressed, NAME_LINE_H,
};
pub use list_search::ListSearch;
pub use log_row::{LogLevel, LogRow};
pub use panel::Panel;
pub use preset_card::PresetCard;
