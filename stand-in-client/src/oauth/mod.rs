//! OAuth 2.0 Authorization Code + PKCE flow (RFC 7636).
//!
//! Gated behind the `oauth` feature — provides UI-agnostic OAuth 2.0
//! authentication for the Streamable HTTP transport (D0: auth is HTTP-only).
//!
//! # Architecture
//!
//! - [`OAuthConfig`] — client registration and endpoint configuration.
//! - `Pkce` — Proof Key for Code Exchange (code_verifier + code_challenge).
//! - [`OAuthTokens`] — access/refresh token pair with expiry.
//! - [`OAuthFlow`] — runs the full Authorization Code + PKCE flow and
//!   token refresh, both UI-agnostic (the app provides a browser-opener).

mod config;
mod flow;
mod loopback;
mod pkce;
mod tokens;

pub use config::OAuthConfig;
pub use flow::OAuthFlow;
pub use tokens::OAuthTokens;
