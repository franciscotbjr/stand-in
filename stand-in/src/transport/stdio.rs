//! Stdio transport — reads JSON-RPC from stdin, writes responses to stdout.

use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{error, info};

use crate::error::Result;
use crate::protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use crate::server::RequestHandler;

use super::Transport;

/// Stdio transport for local/CLI MCP servers.
///
/// Reads line-delimited JSON-RPC messages from stdin
/// and writes JSON-RPC responses to stdout.
#[derive(Debug, Default)]
pub struct StdioTransport;

#[async_trait]
impl Transport for StdioTransport {
    async fn run(&self, handler: RequestHandler) -> Result<()> {
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        info!("StdioTransport: listening on stdin");

        loop {
            line.clear();
            let bytes_read = reader.read_line(&mut line).await?;

            if bytes_read == 0 {
                info!("StdioTransport: stdin closed (EOF), shutting down");
                break;
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let request: JsonRpcRequest = match serde_json::from_str(trimmed) {
                Ok(r) => r,
                Err(e) => {
                    error!("Failed to parse JSON-RPC request: {e}");
                    let response = JsonRpcResponse::error(
                        serde_json::Value::Null,
                        JsonRpcError::parse_error(e.to_string()),
                    );
                    let json = serde_json::to_string(&response)?;
                    stdout.write_all(json.as_bytes()).await?;
                    stdout.write_all(b"\n").await?;
                    stdout.flush().await?;
                    continue;
                }
            };

            let response = handler.handle(&request).await;

            // Note: resource subscribe/unsubscribe dispatch returns success {},
            // but no SSE sender is wired here because stdio is unidirectional
            // request/response. Server-initiated notifications (SSE) are an
            // HTTP transport feature only.
            let json = serde_json::to_string(&response)?;
            stdout.write_all(json.as_bytes()).await?;
            stdout.write_all(b"\n").await?;
            stdout.flush().await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdio_default() {
        let transport = StdioTransport::default();
        assert_eq!(format!("{transport:?}"), "StdioTransport");
    }
}
