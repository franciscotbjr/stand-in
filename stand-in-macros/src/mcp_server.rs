//! `#[mcp_server]` macro expansion — generates `serve()` and `serve_http()` methods
//! with inventory-based tool discovery.

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::{ItemStruct, Lit, Meta, parse2};

/// Parsed `#[mcp_server(...)]` attributes.
struct ServerAttrs {
    host: Option<String>,
    port: Option<u16>,
}

impl ServerAttrs {
    fn parse(attr: TokenStream) -> syn::Result<Self> {
        let mut host = None;
        let mut port = None;

        if attr.is_empty() {
            return Ok(Self { host, port });
        }

        let parser = syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated;
        let metas = parser.parse2(attr)?;

        for meta in metas {
            if let Meta::NameValue(nv) = meta {
                let key = nv
                    .path
                    .get_ident()
                    .map(|i| i.to_string())
                    .unwrap_or_default();
                if let syn::Expr::Lit(expr_lit) = &nv.value {
                    match (key.as_str(), &expr_lit.lit) {
                        ("host", Lit::Str(lit_str)) => host = Some(lit_str.value()),
                        ("port", Lit::Int(lit_int)) => {
                            port = Some(lit_int.base10_parse::<u16>()?);
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(Self { host, port })
    }
}

/// Main expansion entry point for `#[mcp_server]`.
pub fn expand(attr: TokenStream, item: TokenStream) -> TokenStream {
    match expand_inner(attr, item) {
        Ok(tokens) => tokens,
        Err(e) => e.to_compile_error(),
    }
}

fn expand_inner(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let attrs = ServerAttrs::parse(attr)?;
    let struc: ItemStruct = parse2(item)?;
    let struct_name = &struc.ident;
    let vis = &struc.vis;

    let serve_http = generate_serve_http(&attrs, vis);

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

                let mut prompt_registry = stand_in::prompt::PromptRegistry::new();

                for factory in inventory::iter::<stand_in::prompt::PromptFactory> {
                    prompt_registry.register(factory.0());
                }

                let server_info = stand_in::server::ServerInfo {
                    name: env!("CARGO_PKG_NAME").to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                };

                let handler = stand_in::server::RequestHandler::new(registry, prompt_registry, server_info);
                transport.run(handler).await
            }

            #serve_http
        }
    })
}

/// Generate the `serve_http()` method, feature-gated under `http`.
fn generate_serve_http(attrs: &ServerAttrs, vis: &syn::Visibility) -> TokenStream {
    let transport_expr = match (&attrs.host, &attrs.port) {
        (Some(host), Some(port)) => {
            quote! {
                stand_in::transport::HttpTransport::new(
                    std::net::SocketAddr::new(
                        #host.parse::<std::net::IpAddr>().expect("invalid host in #[mcp_server]"),
                        #port,
                    )
                )
            }
        }
        (Some(host), None) => {
            quote! {
                stand_in::transport::HttpTransport::new(
                    std::net::SocketAddr::new(
                        #host.parse::<std::net::IpAddr>().expect("invalid host in #[mcp_server]"),
                        3000,
                    )
                )
            }
        }
        (None, Some(port)) => {
            quote! {
                stand_in::transport::HttpTransport::new(
                    std::net::SocketAddr::new(
                        std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
                        #port,
                    )
                )
            }
        }
        (None, None) => {
            quote! {
                stand_in::transport::HttpTransport::default()
            }
        }
    };

    quote! {
        /// Start the MCP server over HTTP using macro-configured defaults.
        ///
        /// Uses the `host` and `port` attributes from `#[mcp_server]`, or
        /// falls back to `127.0.0.1:3000` if not specified.
        #[cfg(feature = "http")]
        #vis async fn serve_http() -> stand_in::error::Result<()> {
            let transport = #transport_expr;
            Self::serve(transport).await
        }
    }
}
