//! `#[mcp_resource]` macro expansion — generates a struct + McpResource impl from an async function.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::Parser;
use syn::{FnArg, ItemFn, Lit, Meta, Pat, PatType, ReturnType, Type, parse2};

/// Parse macro attributes: `uri = "...", name = "...", description = "...", mime_type = "..."`
struct ResourceAttrs {
    uri: String,
    name: Option<String>,
    description: Option<String>,
    mime_type: Option<String>,
}

impl ResourceAttrs {
    fn parse(attr: TokenStream) -> syn::Result<Self> {
        let mut uri = None;
        let mut name = None;
        let mut description = None;
        let mut mime_type = None;

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
                        "uri" => uri = Some(lit_str.value()),
                        "name" => name = Some(lit_str.value()),
                        "description" => description = Some(lit_str.value()),
                        "mime_type" => mime_type = Some(lit_str.value()),
                        _ => {}
                    }
                }
            }
        }

        Ok(Self {
            uri: uri.ok_or_else(|| {
                syn::Error::new(proc_macro2::Span::call_site(), "missing `uri` attribute")
            })?,
            name,
            description,
            mime_type,
        })
    }
}

/// Main expansion entry point for `#[mcp_resource(...)]`.
pub fn expand(attr: TokenStream, item: TokenStream) -> TokenStream {
    match expand_inner(attr, item) {
        Ok(tokens) => tokens,
        Err(e) => e.to_compile_error(),
    }
}

fn expand_inner(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let attrs = ResourceAttrs::parse(attr)?;
    let func: ItemFn = parse2(item)?;

    let func_name = &func.sig.ident;
    let struct_name = format_ident!("{}Resource", to_pascal_case(&func_name.to_string()));
    let resource_uri = &attrs.uri;
    let is_template = resource_uri.contains('{') && resource_uri.contains('}');

    // Name defaults to the function name in PascalCase
    let resource_name = attrs
        .name
        .unwrap_or_else(|| to_pascal_case(&func_name.to_string()));

    // Collect parameters
    let mut param_names: Vec<syn::Ident> = Vec::new();
    let mut param_types: Vec<syn::Type> = Vec::new();

    for arg in &func.sig.inputs {
        if let FnArg::Typed(PatType { pat, ty, .. }) = arg
            && let Pat::Ident(pat_ident) = pat.as_ref()
        {
            param_names.push(pat_ident.ident.clone());
            param_types.push(*ty.clone());
        }
    }

    // Detect return type: String → TextResourceContents, Vec<u8> → BlobResourceContents
    let is_blob = is_return_type_vec_u8(&func.sig.output);

    let (_, read_body) = if is_template && !param_names.is_empty() {
        let wrap = if is_blob {
            quote! {
                stand_in::resource::ReadResourceResult::from_blob(uri, result)
            }
        } else {
            quote! {
                match &self._mime_type {
                    Some(mt) => stand_in::resource::ReadResourceResult::text_with_mime(uri, result, mt),
                    None => stand_in::resource::ReadResourceResult::text(uri, result),
                }
            }
        };
        let body = {
            let param_deserialize: Vec<TokenStream> = param_names
                .iter()
                .zip(param_types.iter())
                .map(|(name, ty)| {
                    let name_str = name.to_string();
                    quote! {
                        let #name: #ty = serde_json::from_value(
                            arguments.get(#name_str)
                                .cloned()
                                .unwrap_or(serde_json::Value::Null)
                        ).map_err(|e| stand_in::error::Error::ResourceError(
                            format!("Failed to deserialize parameter '{}': {}", #name_str, e)
                        ))?;
                    }
                })
                .collect();
            let call_args: Vec<&syn::Ident> = param_names.iter().collect();
            quote! {
                let arguments = stand_in::resource::match_template_params(#resource_uri, uri)
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
                #(#param_deserialize)*
                let result = #func_name(#(#call_args),*).await?;
                Ok(#wrap)
            }
        };
        (wrap, body)
    } else {
        let call_args: Vec<&syn::Ident> = param_names.iter().collect();
        let wrap = if is_blob {
            quote! {
                stand_in::resource::ReadResourceResult::from_blob(uri, result)
            }
        } else {
            quote! {
                match &self._mime_type {
                    Some(mt) => stand_in::resource::ReadResourceResult::text_with_mime(uri, result, mt),
                    None => stand_in::resource::ReadResourceResult::text(uri, result),
                }
            }
        };
        let body = if call_args.is_empty() {
            quote! {
                let result = #func_name().await?;
                Ok(#wrap)
            }
        } else {
            quote! {
                let arguments = stand_in::resource::match_template_params(#resource_uri, uri)
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
                #(
                    let #param_names: #param_types = serde_json::from_value(
                        arguments.get(stringify!(#param_names))
                            .cloned()
                            .unwrap_or(serde_json::Value::Null)
                    ).map_err(|e| stand_in::error::Error::ResourceError(
                        format!("Failed to deserialize parameter '{}': {}", stringify!(#param_names), e)
                    ))?;
                )*
                let result = #func_name(#(#param_names),*).await?;
                Ok(#wrap)
            }
        };
        (wrap, body)
    };

    // Prepare attribute strings for to_resource/to_template
    let desc_init = attrs
        .description
        .as_ref()
        .map(|d| quote! { Some(#d.to_string()) })
        .unwrap_or_else(|| quote! { None });
    let mime_init = attrs
        .mime_type
        .as_ref()
        .map(|m| quote! { Some(#m.to_string()) })
        .unwrap_or_else(|| quote! { None });

    let desc_clone = attrs
        .description
        .as_ref()
        .map(|_| quote! { self._description.clone() })
        .unwrap_or_else(|| quote! { None });
    let mime_clone = attrs
        .mime_type
        .as_ref()
        .map(|_| quote! { self._mime_type.clone() })
        .unwrap_or_else(|| quote! { None });

    let (to_resource_body, to_template_body) = if is_template {
        let template_code = quote! {
            fn to_template(&self) -> Option<stand_in::resource::ResourceTemplate> {
                Some(stand_in::resource::ResourceTemplate {
                    uri_template: #resource_uri.to_string(),
                    name: #resource_name.to_string(),
                    description: #desc_clone,
                    mime_type: #mime_clone,
                })
            }
        };
        (TokenStream::new(), template_code)
    } else {
        let resource_code = quote! {
            fn to_resource(&self) -> Option<stand_in::resource::Resource> {
                Some(stand_in::resource::Resource {
                    uri: #resource_uri.to_string(),
                    name: #resource_name.to_string(),
                    description: #desc_clone,
                    mime_type: #mime_clone,
                    size: None,
                    annotations: None,
                })
            }
        };
        (resource_code, TokenStream::new())
    };

    Ok(quote! {
        // Keep the original function
        #func

        /// Auto-generated resource struct for [`#func_name`].
        #[derive(Debug)]
        pub struct #struct_name {
            _description: Option<String>,
            _mime_type: Option<String>,
        }

        #[async_trait::async_trait]
        impl stand_in::resource::McpResource for #struct_name {
            fn uri(&self) -> &str {
                #resource_uri
            }

            fn name(&self) -> &str {
                #resource_name
            }

            fn description(&self) -> Option<&str> {
                self._description.as_deref()
            }

            fn mime_type(&self) -> Option<&str> {
                self._mime_type.as_deref()
            }

            fn is_template(&self) -> bool {
                #is_template
            }

            async fn read(
                &self,
                uri: &str,
            ) -> stand_in::error::Result<stand_in::resource::ReadResourceResult> {
                #read_body
            }

            #to_resource_body
            #to_template_body
        }

        inventory::submit! {
            stand_in::resource::ResourceFactory(|| -> Box<dyn stand_in::resource::McpResource> {
                Box::new(#struct_name {
                    _description: #desc_init,
                    _mime_type: #mime_init,
                })
            })
        }
    })
}

/// Inspect the return type to detect `Result<Vec<u8>, ...>` vs `Result<String, ...>`.
///
/// Returns `true` if the success type is `Vec<u8>`, indicating the resource
/// should produce `BlobResourceContents` (base64-encoded binary data).
fn is_return_type_vec_u8(output: &ReturnType) -> bool {
    if let ReturnType::Type(_, ty) = output
        && let Type::Path(type_path) = ty.as_ref()
        && let Some(seg) = type_path.path.segments.last()
        && seg.ident == "Result"
        && let syn::PathArguments::AngleBracketed(args) = &seg.arguments
        && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
        && let Type::Path(inner_path) = inner_ty
        && let Some(inner_seg) = inner_path.path.segments.last()
        && inner_seg.ident == "Vec"
        && let syn::PathArguments::AngleBracketed(vec_args) = &inner_seg.arguments
        && let Some(syn::GenericArgument::Type(elem_ty)) = vec_args.args.first()
        && let Type::Path(elem_path) = elem_ty
        && let Some(elem_seg) = elem_path.path.segments.last()
    {
        return elem_seg.ident == "u8";
    }
    false
}

/// Convert snake_case to PascalCase.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("server_version"), "ServerVersion");
        assert_eq!(to_pascal_case("readme"), "Readme");
        assert_eq!(to_pascal_case("docs_readme"), "DocsReadme");
    }

    #[test]
    fn test_is_return_type_vec_u8() {
        let ret_blob: ReturnType = syn::parse_quote!(-> Result<Vec<u8>, Error>);
        assert!(is_return_type_vec_u8(&ret_blob));

        let ret_str: ReturnType = syn::parse_quote!(-> Result<String, Error>);
        assert!(!is_return_type_vec_u8(&ret_str));

        let ret_other: ReturnType = syn::parse_quote!(-> Result<i32, Error>);
        assert!(!is_return_type_vec_u8(&ret_other));
    }

    #[test]
    fn test_resource_attrs_parse() {
        let attrs: TokenStream = syn::parse_quote!(
            uri = "file:///readme.md",
            name = "README",
            description = "Project readme"
        );
        let parsed = ResourceAttrs::parse(attrs).unwrap();
        assert_eq!(parsed.uri, "file:///readme.md");
        assert_eq!(parsed.name.unwrap(), "README");
        assert_eq!(parsed.description.unwrap(), "Project readme");
        assert!(parsed.mime_type.is_none());
    }

    #[test]
    fn test_resource_attrs_parse_minimal() {
        let attrs: TokenStream = syn::parse_quote!(uri = "file:///readme.md");
        let parsed = ResourceAttrs::parse(attrs).unwrap();
        assert_eq!(parsed.uri, "file:///readme.md");
        assert!(parsed.name.is_none());
    }

    #[test]
    fn test_resource_attrs_missing_uri_errors() {
        let attrs: TokenStream = syn::parse_quote!(name = "README");
        let result = ResourceAttrs::parse(attrs);
        assert!(result.is_err());
    }

    #[test]
    fn test_template_detection() {
        let uri_with_template = "docs://{topic}/readme";
        let uri_concrete = "file:///readme.md";
        assert!(uri_with_template.contains('{') && uri_with_template.contains('}'));
        assert!(!(uri_concrete.contains('{') && uri_concrete.contains('}')));
    }
}
