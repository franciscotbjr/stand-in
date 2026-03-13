//! Transport layer — abstracts communication between MCP client and server.

mod transport_trait;

pub use transport_trait::Transport;

#[cfg(feature = "stdio")]
mod stdio;

#[cfg(feature = "stdio")]
pub use stdio::StdioTransport;

#[cfg(feature = "http")]
mod http_transport;
#[cfg(feature = "http")]
pub(crate) mod session;
#[cfg(feature = "http")]
pub(crate) mod session_store;
#[cfg(feature = "http")]
pub(crate) mod sse;

#[cfg(feature = "http")]
pub use http_transport::HttpTransport;
