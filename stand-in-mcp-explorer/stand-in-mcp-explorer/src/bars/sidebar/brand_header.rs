//! Brand header — returns the icon element for the brand mark slot
//! inside `SidebarShell`. The shell already provides the gradient + ring.
//! This file just exports the icon used inside the 34×34 brand mark.

use gpui::{IntoElement, px};
use stand_in_mcp_explorer_ds::core::icon::{Icon, IconName};

pub fn brand_mark_icon() -> impl IntoElement {
    Icon::new(IconName::Leaf).with_px(px(18.))
}
