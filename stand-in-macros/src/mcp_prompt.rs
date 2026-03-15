//! `#[mcp_prompt]` macro expansion — generates a struct + McpPrompt impl from an async function.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::Parser;
use syn::{FnArg, ItemFn, Lit, Meta, Pat, PatType, parse2};

/// Parse macro attributes: `name = "...", description = "..."`
struct PromptAttrs {
    name: String,
    description: String,
}

impl PromptAttrs {
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

/// Main expansion entry point for `#[mcp_prompt(...)]`.
pub fn expand(attr: TokenStream, item: TokenStream) -> TokenStream {
    match expand_inner(attr, item) {
        Ok(tokens) => tokens,
        Err(e) => e.to_compile_error(),
    }
}

fn expand_inner(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let attrs = PromptAttrs::parse(attr)?;
    let func: ItemFn = parse2(item)?;

    let func_name = &func.sig.ident;
    let struct_name = format_ident!("{}Prompt", to_pascal_case(&func_name.to_string()));
    let prompt_name = &attrs.name;
    let prompt_description = &attrs.description;

    // Collect parameters to build PromptArgument list and deserialization code.
    let mut param_names: Vec<syn::Ident> = Vec::new();
    let mut param_types: Vec<syn::Type> = Vec::new();
    let mut argument_tokens: Vec<TokenStream> = Vec::new();

    for arg in &func.sig.inputs {
        if let FnArg::Typed(PatType { pat, ty, .. }) = arg
            && let Pat::Ident(pat_ident) = pat.as_ref()
        {
            let name = &pat_ident.ident;
            let name_str = name.to_string();
            let is_required = !is_option(ty);

            param_names.push(name.clone());
            param_types.push(*ty.clone());

            argument_tokens.push(quote! {
                stand_in::prompt::PromptArgument {
                    name: #name_str.to_string(),
                    description: None,
                    required: Some(#is_required),
                }
            });
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
                ).map_err(|e| stand_in::error::Error::PromptError(
                    format!("Failed to deserialize parameter '{}': {}", #name_str, e)
                ))?;
            }
        })
        .collect();

    let call_args: Vec<&syn::Ident> = param_names.iter().collect();

    Ok(quote! {
        // Keep the original function
        #func

        /// Auto-generated prompt struct for [`#func_name`].
        #[derive(Debug)]
        pub struct #struct_name;

        #[async_trait::async_trait]
        impl stand_in::prompt::McpPrompt for #struct_name {
            fn name(&self) -> &str {
                #prompt_name
            }

            fn description(&self) -> &str {
                #prompt_description
            }

            fn arguments(&self) -> Vec<stand_in::prompt::PromptArgument> {
                vec![#(#argument_tokens),*]
            }

            async fn execute(
                &self,
                arguments: serde_json::Value,
            ) -> stand_in::error::Result<stand_in::prompt::Prompt> {
                #(#deserialize_args)*
                #func_name(#(#call_args),*).await
            }
        }

        inventory::submit! {
            stand_in::prompt::PromptFactory(|| -> Box<dyn stand_in::prompt::McpPrompt> {
                Box::new(#struct_name)
            })
        }
    })
}

/// Returns true if the type is `Option<T>`.
fn is_option(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "Option";
    }
    false
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
