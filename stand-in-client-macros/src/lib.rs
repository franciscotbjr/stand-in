//! Procedural macros for stand-in MCP client framework.
//!
//! This crate provides the `#[mcp_client]` attribute macro that generates
//! typed stubs for a known MCP server from a trait definition.

mod mcp_client;

use proc_macro::TokenStream;

/// Marks a trait as an MCP client stub.
///
/// Generates a typed client struct with async methods that serialize their
/// arguments and delegate to `stand_in_client::Client::call_tool`.
///
/// ## Tool name
///
/// By default, the Rust method name is used as the MCP tool name. Use
/// `#[tool(name = "...")]` on individual methods to override:
///
/// ```rust,ignore
/// use stand_in_client::prelude::*;
///
/// #[mcp_client]
/// trait Weather {
///     /// Calls the tool "get_weather" (derived from method name).
///     async fn get_weather(&self, city: String) -> Result<String>;
///
///     #[tool(name = "add")]
///     async fn sum(&self, a: i64, b: i64) -> Result<i64>;
/// }
///
/// let client = Client::builder()
///     .transport(StdioTransport::command("my-server", &[] as &[&str]))
///     .connect()
///     .await?;
///
/// // Typed call — args serialized, response deserialized.
/// let api = WeatherClient::new(client);
/// let result = api.get_weather("São Paulo".into()).await?;
/// ```
///
/// ## Return type detection
///
/// - `Result<String>` — returns the text content of the first `Content::Text`
///   directly (no additional deserialization).
/// - `Result<i64>`, `Result<MyStruct>`, etc. — parses the text content with
///   `serde_json::from_str`.
///
/// ## Error handling
///
/// The typed layer **collapses** the two error planes: both protocol errors
/// and tool execution errors (`isError: true`) become `Err(...)`. This differs
/// from the dynamic `call_tool()` which returns tool errors as `Ok(CallToolResult)`
/// with `isError` as data.
#[proc_macro_attribute]
pub fn mcp_client(attr: TokenStream, item: TokenStream) -> TokenStream {
    mcp_client::expand(attr.into(), item.into()).into()
}
