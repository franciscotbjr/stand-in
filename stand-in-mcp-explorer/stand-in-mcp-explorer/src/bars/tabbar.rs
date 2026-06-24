//! Tabbar — 5 tabs (Tools/Resources/Prompts/Notifications/History) with
//! icons, counts, and an oby underline on the active tab. Rendered only when
//! Connected (the caller gates with `.when()`).
//!
//! Counts follow the canon rule: Tools/Resources/Prompts always show (even 0);
//! Notifications/History only show when >0.

use crate::app::active_tab::{Tab, TabCounts, tab_count_visible};
use crate::app::i18n::{Lang, tr};
use gpui::IntoElement;
use stand_in_mcp_explorer_ds::core::button::ClickHandler;
use stand_in_mcp_explorer_ds::core::icon::IconName;
use stand_in_mcp_explorer_ds::navigation::tabbar::{TabItem, Tabbar};

pub fn render_tabbar(
    active_tab: Tab,
    counts: &TabCounts,
    lang: Lang,
    handlers: Vec<Option<ClickHandler>>,
) -> impl IntoElement {
    let items = build_tab_items(counts, lang);
    Tabbar::new("main-tabs", items, active_tab.selected_ix()).handlers(handlers)
}

fn build_tab_items(counts: &TabCounts, lang: Lang) -> Vec<TabItem> {
    let mut items = Vec::with_capacity(5);

    let tabs: [(Tab, IconName, &str); 5] = [
        (Tab::Tools, IconName::Tool, tr("tabs.tools", lang)),
        (Tab::Resources, IconName::Doc, tr("tabs.resources", lang)),
        (Tab::Prompts, IconName::Chat, tr("tabs.prompts", lang)),
        (
            Tab::Notifications,
            IconName::Bell,
            tr("tabs.notifications", lang),
        ),
        (Tab::History, IconName::History, tr("tabs.history", lang)),
    ];

    for (tab, icon, label) in tabs {
        let mut item = TabItem::new(format!("tab-{}", label), label).icon(icon);
        if let Some(n) = tab_count_visible(tab, counts) {
            item = item.count(n);
        }
        items.push(item);
    }

    items
}
