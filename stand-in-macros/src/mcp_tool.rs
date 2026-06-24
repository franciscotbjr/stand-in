//! `#[mcp_tool]` macro expansion — generates a struct + McpTool impl from an async function.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::Parser;
use syn::{FnArg, ItemFn, Lit, Meta, Pat, PatType, parse2};

use crate::schema::type_to_schema;

/// Parse macro attributes: `name = "...", description = "..."`
struct ToolAttrs {
    name: String,
    description: String,
}

impl ToolAttrs {
    fn parse(attr: TokenStream) -> syn::Result<Self> {
        let mut name = None;
        let mut description = None;

        let parser = syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated;
        let metas = parser.parse2(attr)?;

        for meta in metas {
            if let Meta::NameValue(nv) = meta {
                let key = nv
                    .path
                    .get_ident()
                    .map(|i| i.to_string())
                    .unwrap_or_default();
                if let syn::Expr::Lit(expr_lit) = &nv.value
                    && let Lit::Str(lit_str) = &expr_lit.lit
                {
                    match key.as_str() {
                        "name" => name = Some(lit_str.value()),
                        "description" => description = Some(lit_str.value()),
                        _ => {}
                    }
                }
            }
        }

        Ok(Self {
            name: name.ok_or_else(|| {
                syn::Error::new(proc_macro2::Span::call_site(), "missing `name` attribute")
            })?,
            description: description.ok_or_else(|| {
                syn::Error::new(
                    proc_macro2::Span::call_site(),
                    "missing `description` attribute",
                )
            })?,
        })
    }
}

/// Main expansion entry point for `#[mcp_tool(...)]`.
pub fn expand(attr: TokenStream, item: TokenStream) -> TokenStream {
    match expand_inner(attr, item) {
        Ok(tokens) => tokens,
        Err(e) => e.to_compile_error(),
    }
}

fn expand_inner(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let attrs = ToolAttrs::parse(attr)?;
    let func: ItemFn = parse2(item)?;

    let func_name = &func.sig.ident;
    let struct_name = format_ident!("{}Tool", to_pascal_case(&func_name.to_string()));
    let tool_name = &attrs.name;
    let tool_description = &attrs.description;

    // Collect parameters (skip &self if present)
    let mut param_names: Vec<syn::Ident> = Vec::new();
    let mut param_types: Vec<syn::Type> = Vec::new();
    let mut property_tokens: Vec<TokenStream> = Vec::new();
    let mut required_names: Vec<String> = Vec::new();

    for arg in &func.sig.inputs {
        if let FnArg::Typed(PatType { pat, ty, .. }) = arg
            && let Pat::Ident(pat_ident) = pat.as_ref()
        {
            let name = &pat_ident.ident;
            let name_str = name.to_string();
            let (schema_tokens, is_required) = type_to_schema(ty);

            param_names.push(name.clone());
            param_types.push(*ty.clone());
            property_tokens.push(quote! {
                properties.insert(#name_str.to_string(), #schema_tokens);
            });

            if is_required {
                required_names.push(name_str);
            }
        }
    }

    // Generate argument deserialization
    let deserialize_args: Vec<TokenStream> = param_names
        .iter()
        .zip(param_types.iter())
        .map(|(name, ty)| {
            let name_str = name.to_string();
            quote! {
                let #name: #ty = serde_json::from_value(
                    arguments.get(#name_str)
                        .cloned()
                        .unwrap_or(serde_json::Value::Null)
                ).map_err(|e| stand_in::error::Error::ToolError(
                    format!("Failed to deserialize parameter '{}': {}", #name_str, e)
                ))?;
            }
        })
        .collect();

    let call_args: Vec<&syn::Ident> = param_names.iter().collect();

    let required_tokens = if required_names.is_empty() {
        quote! { None }
    } else {
        quote! { Some(vec![#(#required_names.to_string()),*]) }
    };

    Ok(quote! {
        // Keep the original function
        #func

        /// Auto-generated tool struct for [`#func_name`].
        #[derive(Debug)]
        pub struct #struct_name;

        #[async_trait::async_trait]
        impl stand_in::tool::McpTool for #struct_name {
            fn name(&self) -> &str {
                #tool_name
            }

            fn description(&self) -> &str {
                #tool_description
            }

            fn input_schema(&self) -> serde_json::Value {
                let mut properties = serde_json::Map::new();
                #(#property_tokens)*
                let mut schema = serde_json::json!({
                    "type": "object",
                    "properties": serde_json::Value::Object(properties),
                });
                let required: Option<Vec<String>> = #required_tokens;
                if let Some(req) = required {
                    schema["required"] = serde_json::json!(req);
                }
                schema
            }

            async fn execute(
                &self,
                arguments: serde_json::Value,
            ) -> stand_in::error::Result<stand_in::tool::CallToolResult> {
                #(#deserialize_args)*
                let result = #func_name(#(#call_args),*).await?;
                Ok(stand_in::tool::CallToolResult::text(result))
            }
        }

        inventory::submit! {
            stand_in::tool::ToolFactory(|| -> Box<dyn stand_in::tool::McpTool> {
                Box::new(#struct_name)
            })
        }
    })
}

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}
