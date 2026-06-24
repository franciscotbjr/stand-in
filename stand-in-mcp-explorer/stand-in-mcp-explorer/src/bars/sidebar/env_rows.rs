//! Environment variable rows — N × `KeyValueRow` plus a `ToggleLink`
//! to add a new row. Only displayed when transport is `Stdio`.

use crate::app::i18n::{Lang, tr};
use gpui::{App, InteractiveElement, IntoElement, ParentElement, Styled, Window};
use gpui_component::v_flex;
use stand_in_mcp_explorer_ds::core::button::ClickHandler;
use stand_in_mcp_explorer_ds::core::toggle_link::ToggleLink;
use stand_in_mcp_explorer_ds::forms::key_value_row::KeyValueRow;

use super::sidebar_state::SidebarState;

pub fn render_env_rows(
    state: &SidebarState,
    lang: Lang,
    on_add: Option<ClickHandler>,
    mut on_remove: Vec<ClickHandler>,
    _window: &mut Window,
    _cx: &mut App,
) -> impl IntoElement {
    let mut col = v_flex().id("env-rows").w_full().gap_2();

    for (i, row) in state.env_rows.iter().enumerate() {
        let mut kv = KeyValueRow::new(&row.key, &row.value).id(("env-row", i));

        if i < on_remove.len() {
            let handler = std::mem::replace(&mut on_remove[i], Box::new(|_, _, _| {}));
            kv = kv.on_remove(Box::new(move |ev, w, cx| handler(ev, w, cx)));
        }

        col = col.child(kv);
    }

    let mut add_link = ToggleLink::new("add-env-var", tr("sidebar.addVar", lang));
    if let Some(h) = on_add {
        add_link = add_link.on_click(move |ev, w, cx| h(ev, w, cx));
    }

    col.child(add_link)
}
