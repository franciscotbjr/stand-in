//! `#[mcp_server]` macro expansion — generates `serve()` method with inventory-based tool discovery.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, ItemStruct};

/// Main expansion entry point for `#[mcp_server]`.
pub fn expand(_attr: TokenStream, item: TokenStream) -> TokenStream {
    match expand_inner(item) {
        Ok(tokens) => tokens,
        Err(e) => e.to_compile_error(),
    }
}

fn expand_inner(item: TokenStream) -> syn::Result<TokenStream> {
    let struc: ItemStruct = parse2(item)?;
    let struct_name = &struc.ident;
    let vis = &struc.vis;

    Ok(quote! {
        #struc

        impl #struct_name {
            /// Start the MCP server using the given transport.
            ///
            /// Discovers all tools registered via `#[mcp_tool]` using `inventory`,
            /// builds the request handler, and runs the transport loop.
            #vis async fn serve(transport: impl stand_in::transport::Transport) -> stand_in::error::Result<()> {
                let mut registry = stand_in::tool::ToolRegistry::new();

                for factory in inventory::iter::<stand_in::tool::ToolFactory> {
                    registry.register(factory.0());
                }

                let server_info = stand_in::server::ServerInfo {
                    name: env!("CARGO_PKG_NAME").to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                };

                let handler = stand_in::server::RequestHandler::new(registry, server_info);
                transport.run(handler).await
            }
        }
    })
}
