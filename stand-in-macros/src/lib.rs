//! Procedural macros for stand-in MCP server framework.
//!
//! This crate provides the following macros:
//! - `#[mcp_server]` — Wire everything together, generates initialization and dispatch
//! - `#[mcp_tool]` — Declare a tool with typed parameters
//! - `#[mcp_resource]` — Expose a resource with URI templates
//! - `#[mcp_prompt]` — Define reusable prompt templates

use proc_macro::TokenStream;

mod mcp_tool;
mod schema;

/// Marks a struct as an MCP server.
///
/// Generates initialization, capability negotiation, and dispatch logic.
///
/// # Example
///
/// ```rust,ignore
/// #[mcp_server]
/// struct MyServer;
/// ```
#[proc_macro_attribute]
pub fn mcp_server(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // TODO: Implement server macro
    item
}

/// Marks an async function as an MCP tool.
///
/// The macro infers the JSON Schema from the function signature and generates
/// the `McpTool` implementation automatically.
///
/// # Example
///
/// ```rust,ignore
/// #[mcp_tool(
///     name = "get_weather",
///     description = "Returns current weather for a given city"
/// )]
/// async fn get_weather(city: String) -> Result<String> {
///     Ok(format!("{}: sunny", city))
/// }
/// ```
#[proc_macro_attribute]
pub fn mcp_tool(attr: TokenStream, item: TokenStream) -> TokenStream {
    mcp_tool::expand(attr.into(), item.into()).into()
}

/// Marks an async function as an MCP resource.
///
/// # Example
///
/// ```rust,ignore
/// #[mcp_resource(
///     uri = "project://{project_id}/readme",
///     description = "The project README"
/// )]
/// async fn project_readme(project_id: String) -> Result<String> {
///     Ok("# README".to_string())
/// }
/// ```
#[proc_macro_attribute]
pub fn mcp_resource(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // TODO: Implement resource macro
    item
}

/// Marks an async function as an MCP prompt template.
///
/// # Example
///
/// ```rust,ignore
/// #[mcp_prompt(
///     name = "summarize_project",
///     description = "Generate a project status summary"
/// )]
/// async fn summarize_project(project_id: String) -> Result<Prompt> {
///     Ok(Prompt::user("Summarize this project"))
/// }
/// ```
#[proc_macro_attribute]
pub fn mcp_prompt(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // TODO: Implement prompt macro
    item
}
