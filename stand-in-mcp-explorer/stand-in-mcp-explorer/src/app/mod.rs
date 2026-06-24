//! App shell — the top-level `StudioApp` component, the async bridge
//! (tokio engine ↔ gpui UI), the connection state machine, and event types.
//!
//! - `events` — message types flowing across the bridge (`UiCommand`, `EngineEvent`).
//! - `conn_state` — connection state enum + pure reducer + capture fixtures.
//! - `engine_loop` — dedicated tokio thread hosting the `stand-in-client` SDK.
//! - `studio_app` — gpui `Render` impl, driven entirely by `ConnState`.
//! - `active_tab` — tab routing enum + pure `work_mode` selector (M8).
//! - `perf` — list-rendering instrumentation behind the dev `perf` feature (037).

pub mod active_tab;
pub mod conn_state;
pub mod engine_loop;
pub mod events;
pub mod history;
pub mod i18n;
pub mod log;
#[cfg(feature = "perf")]
pub mod perf;
pub mod secrets;
pub mod servers;
pub mod settings;
pub mod studio_app;
