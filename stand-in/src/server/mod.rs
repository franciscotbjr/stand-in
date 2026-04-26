//! MCP server types — capabilities, handshake, and request dispatch.

mod client_info;
mod handler;
mod initialize_params;
mod initialize_result;
mod prompts_capability;
mod resources_capability;
mod server_capabilities;
mod server_info;
mod tools_capability;

pub use client_info::ClientInfo;
pub use handler::RequestHandler;
pub use initialize_params::InitializeParams;
pub use initialize_result::InitializeResult;
pub use prompts_capability::PromptsCapability;
pub use resources_capability::ResourcesCapability;
pub use server_capabilities::ServerCapabilities;
pub use server_info::ServerInfo;
pub use tools_capability::ToolsCapability;
