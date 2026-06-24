//! Transport trait — abstracts the communication layer between client and server.

use async_trait::async_trait;

use crate::error::Result;
use crate::server::RequestHandler;

/// Abstracts the communication layer between an MCP client and server.
#[async_trait]
pub trait Transport: Send + Sync {
    /// Run the transport, reading requests and writing responses until shutdown.
    async fn run(&self, handler: RequestHandler) -> Result<()>;
}
