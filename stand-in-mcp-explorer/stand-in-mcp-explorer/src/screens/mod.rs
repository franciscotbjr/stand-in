//! Screens region — Onboarding, Error state, Settings, Auth panel, Env panel.
//!
//! - `onboarding` (M8): disconnected "ready" EmptyState + CTA.
//! - `error_state` (M8): disconnected "error" EmptyState + CTA (retry).
//! - `settings` (M15): overlay with scrim + elevated card for theme, density,
//!   primary colour, and guided mode (BUG-3).
//! - `auth_panel` (035/M3): floating panel for HTTP authorization.
//! - `env_panel` (036/M1): floating panel for STDIO environment variables.

pub mod auth_panel;
pub mod env_panel;
pub mod error_state;
pub mod onboarding;
pub mod settings;
