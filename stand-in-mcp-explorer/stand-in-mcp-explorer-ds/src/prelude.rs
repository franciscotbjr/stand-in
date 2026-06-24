//! Prelude — re-exports the essential gpui and gpui-component items
//! so the gallery (and later the app) can compose without importing
//! gpui directly all over the place.
//!
//! Every import here is current against the pinned revisions:
//! `gpui` 0.2.2 @ `3f5705b9` / `gpui-component` 0.5.2 @ `70d2c44b`.

pub use gpui::{
    Bounds, Context, IntoElement, ParentElement, Render, SharedString, Styled, Window,
    WindowBounds, WindowOptions, div, px, size,
};
pub use gpui_component::{Root, h_flex, v_flex};
