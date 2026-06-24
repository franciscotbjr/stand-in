//! Section registry for the Storybook gallery.
//!
//! Each milestone adds its component(s) by adding a module file and appending a
//! `Section` entry to `registry()`. M1 was empty; M2 adds Foundations; M3 adds
//! Icon; M4 adds Indicators (StatusDot + Spinner); M5 adds Actions (Button +
//! IconButton); M6 adds Badges (Badge + CopyButton + ToggleLink); M7 adds
//! Forms (Field); M8 adds Forms Advanced (KeyValueRow + SegmentedControl); M9
//! adds Select; M10 adds SectionLabel + CapChip (counted under Navigation); M11
//! adds Topbar + Tabbar; M12 adds SidebarShell; M13 adds Panel + ListItem; M14
//! adds ListSearch + PresetCard; M15 adds LogRow + EmptyState + HintBar; M16
//! adds data-extra (JsonView landed in M15 with the M16 components).
//!
//! A section identifies a component or component group, carries a stable
//! id for the sidebar index, and provides a render function that produces
//! the section's content for a given state + mode.
//!
//! ## Named states
//!
//! Every section supports `"overview"` (the default). No additional named
//! states (e.g. `"open"`, `"empty"`) exist as of M17 — component-internal
//! state (Select popup toggle, ListItem selection, Button loading) is driven
//! interactively in the live gallery and in capture mode is tested via the
//! section's overview fixture. State-dependent capture fixtures are forward-
//! looking infrastructure (the `state` parameter on the CLI exists).

mod actions;
mod badges;
mod data;
mod data_extra;
mod forms;
mod forms_advanced;
mod foundations;
mod icon;
mod indicators;
mod navigation;
mod select;
mod util;

use actions::render_actions;
use badges::render_badges;
use data::render_data;
use data_extra::render_data_extra;
use forms::render_forms;
use forms_advanced::render_forms_advanced;
use foundations::render_foundations;
use icon::render_icon;
use indicators::render_indicators;
use navigation::render_navigation;
use select::render_select;

use super::shell::GalleryShell;
use gpui::SharedString;

/// A section in the gallery sidebar index.
pub struct Section {
    pub id: SharedString,
    pub label: SharedString,
    pub render: for<'a> fn(
        state: &'a str,
        mode: &'a str,
        this: &'a GalleryShell,
        cx: &'a mut gpui::Context<GalleryShell>,
    ) -> gpui::AnyElement,
}

pub fn registry() -> Vec<Section> {
    vec![
        Section {
            id: "foundations".into(),
            label: "Foundations".into(),
            render: render_foundations,
        },
        Section {
            id: "icon".into(),
            label: "Icon".into(),
            render: render_icon,
        },
        Section {
            id: "indicators".into(),
            label: "StatusDot + Spinner".into(),
            render: render_indicators,
        },
        Section {
            id: "actions".into(),
            label: "Button + IconButton".into(),
            render: render_actions,
        },
        Section {
            id: "badges".into(),
            label: "Badge + CopyButton + ToggleLink".into(),
            render: render_badges,
        },
        Section {
            id: "forms".into(),
            label: "Field".into(),
            render: render_forms,
        },
        Section {
            id: "forms-advanced".into(),
            label: "KeyValueRow + SegmentedControl".into(),
            render: render_forms_advanced,
        },
        Section {
            id: "select".into(),
            label: "Select".into(),
            render: render_select,
        },
        Section {
            id: "navigation".into(),
            label: "SectionLabel + CapChip + Topbar + Tabbar + SidebarShell".into(),
            render: render_navigation,
        },
        Section {
            id: "data".into(),
            label: "Panel + ListItem + ListSearch + PresetCard".into(),
            render: render_data,
        },
        Section {
            id: "data-extra".into(),
            label: "LogRow + EmptyState + HintBar".into(),
            render: render_data_extra,
        },
    ]
}
