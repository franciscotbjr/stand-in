//! Transport layer — abstracts communication between MCP client and server.

mod transport_trait;

pub use transport_trait::Transport;

#[cfg(feature = "stdio")]
mod stdio;

#[cfg(feature = "stdio")]
pub use stdio::StdioTransport;
