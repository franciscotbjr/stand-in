//! `#[mcp_client]` macro expansion — generates a typed client stub from a trait.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    FnArg, GenericArgument, ItemTrait, Lit, Meta, Pat, PatType, PathArguments, ReturnType,
    TraitItem, TraitItemFn, Type, TypePath, parse2,
};

/// Entry point for `#[mcp_client(...)]` proc-macro attribute.
///
/// Parses a trait, generates a typed `{Trait}Client` struct with inherent
/// async methods that serialize arguments and delegate to `Client::call_tool`.
///
/// Errors are surfaced to the user as compile diagnostics, never as panics.
pub fn expand(attr: TokenStream, item: TokenStream) -> TokenStream {
    match expand_inner(attr, item) {
        Ok(tokens) => tokens,
        Err(e) => e.to_compile_error(),
    }
}

fn expand_inner(_attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let input_trait: ItemTrait = parse2(item)?;

    let trait_name = &input_trait.ident;
    let client_name = format_ident!("{}Client", to_pascal_case(&trait_name.to_string()));
    let trait_vis = &input_trait.vis;

    // Collect trait attributes (doc comments, derives etc.) for re-emission.
    let trait_attrs = &input_trait.attrs;

    // Clean trait items: strip #[tool(...)] attrs before re-emitting.
    let cleaned_items = clean_trait_items(&input_trait.items);

    // Build the typed methods from the trait's method signatures.
    let mut typed_methods: Vec<TokenStream> = Vec::new();

    for item in &input_trait.items {
        if let TraitItem::Fn(method) = item {
            let method_tokens = build_typed_method(&client_name, method)?;
            typed_methods.push(method_tokens);
        }
    }

    let doc = format!(
        "Typed client generated for [`{trait_name}`].\n\n\
         Each trait method becomes a typed async method that serializes the Rust\n\
         arguments to JSON, calls the server tool, and deserializes the response\n\
         back to Rust.\n\n\
         # Tool execution errors\n\n\
         When the server returns `isError: true`, the typed method returns\n\
         `Err(Error::ToolError(...))`. This **collapses** the two error planes\n\
         into one `Result` — unlike the dynamic `call_tool()` which returns\n\
         `Ok(CallToolResult)` with `isError` as data."
    );

    Ok(quote! {
        // Re-emit the trait as IDL / documentation
        #[allow(async_fn_in_trait)]
        #(#trait_attrs)*
        #trait_vis trait #trait_name {
            #(#cleaned_items)*
        }

        #[doc = #doc]
        #trait_vis struct #client_name {
            client: ::stand_in_client::Client,
        }

        impl #client_name {
            /// Create a new typed client wrapping an existing [`stand_in_client::Client`].
            pub fn new(client: ::stand_in_client::Client) -> Self {
                Self { client }
            }

            /// Return a shared reference to the inner [`stand_in_client::Client`].
            pub fn client(&self) -> &::stand_in_client::Client {
                &self.client
            }

            /// Consume the typed wrapper and return the inner [`stand_in_client::Client`].
            pub fn into_inner(self) -> ::stand_in_client::Client {
                self.client
            }

            #(#typed_methods)*
        }
    })
}

/// Parse the `#[tool(name = "...")]` helper attribute from a method's attrs.
///
/// Returns `Some(name)` if found, `None` otherwise (method name is used as tool name).
fn parse_tool_name(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if !attr.path().is_ident("tool") {
            continue;
        }
        if let Ok(Meta::NameValue(nv)) = attr.parse_args::<Meta>()
            && let syn::Expr::Lit(syn::ExprLit {
                lit: Lit::Str(s), ..
            }) = &nv.value
            && nv.path.is_ident("name")
        {
            return Some(s.value());
        }
    }
    None
}

/// Remove `#[tool(...)]` attributes from a list — they are consumed by the
/// macro and would cause "unknown attribute" errors if re-emitted on the trait.
fn strip_tool_attrs(attrs: &[syn::Attribute]) -> Vec<syn::Attribute> {
    attrs
        .iter()
        .filter(|a| !a.path().is_ident("tool"))
        .cloned()
        .collect()
}

/// Build a typed async method for one trait method.
fn build_typed_method(_client_name: &syn::Ident, method: &TraitItemFn) -> syn::Result<TokenStream> {
    let method_sig = &method.sig;
    let method_name = &method_sig.ident;
    let method_attrs = &method.attrs;

    // Tool name: `#[tool(name = "...")]` or the method name itself.
    let tool_name_str = parse_tool_name(method_attrs).unwrap_or_else(|| method_name.to_string());

    // Validate: method must be async.
    if method_sig.asyncness.is_none() {
        return Err(syn::Error::new_spanned(
            method_sig,
            "methods in #[mcp_client] traits must be async",
        ));
    }

    // Strip the #[tool(...)] attrs that the macro consumed.
    let clean_attrs = strip_tool_attrs(method_attrs);

    // Collect typed parameters (skip &self).
    let mut param_names: Vec<syn::Ident> = Vec::new();
    let mut param_types: Vec<syn::Type> = Vec::new();

    for input in &method_sig.inputs {
        if let FnArg::Typed(PatType { pat, ty, .. }) = input
            && let Pat::Ident(pat_ident) = pat.as_ref()
        {
            let ident = &pat_ident.ident;
            // Skip &self
            if ident == "self" {
                continue;
            }
            param_names.push(ident.clone());
            param_types.push((**ty).clone());
        } else if let FnArg::Receiver(_) = input {
            // &self — skip
            continue;
        }
    }

    // Build the JSON argument construction.
    let json_args = if param_names.is_empty() {
        quote! { ::stand_in_client::__macros::serde_json::json!({}) }
    } else {
        let pairs: Vec<TokenStream> = param_names
            .iter()
            .map(|name| {
                let name_str = name.to_string();
                quote! { #name_str: #name }
            })
            .collect();
        quote! { ::stand_in_client::__macros::serde_json::json!({ #(#pairs),* }) }
    };

    // Determine return type handling: String → text extraction; other → from_str.
    let return_type = &method_sig.output;
    let return_ty = extract_ok_type(return_type).map_err(|e| {
        syn::Error::new_spanned(return_type, format!("return type must be `Result<T>`: {e}"))
    })?;

    let is_string_return = is_path_string(&return_ty);

    let content_extract = quote! {
        call_result.content.first()
            .and_then(|c| {
                if let ::stand_in_client::Content::Text { text } = c {
                    Some(text.clone())
                } else {
                    None
                }
            })
    };

    let call_and_extract = if is_string_return {
        quote! {
            let call_result = self.client.call_tool(#tool_name_str, args).await?;
            if call_result.is_error == Some(true) {
                let msg = #content_extract
                    .unwrap_or_else(|| "unknown tool error".to_string());
                return ::std::result::Result::Err(
                    ::stand_in_client::Error::ToolError(msg)
                );
            }
            let text = #content_extract
                .ok_or_else(|| ::stand_in_client::Error::ToolError(
                    format!("tool '{}' returned empty or non-text content", #tool_name_str)
                ))?;
            ::std::result::Result::Ok(text)
        }
    } else {
        quote! {
            let call_result = self.client.call_tool(#tool_name_str, args).await?;
            if call_result.is_error == Some(true) {
                let msg = #content_extract
                    .unwrap_or_else(|| "unknown tool error".to_string());
                return ::std::result::Result::Err(
                    ::stand_in_client::Error::ToolError(msg)
                );
            }
            let text = #content_extract
                .ok_or_else(|| ::stand_in_client::Error::ToolError(
                    format!("tool '{}' returned empty or non-text content", #tool_name_str)
                ))?;
            let value: #return_ty = ::stand_in_client::__macros::serde_json::from_str(&text)
                .map_err(|e| ::stand_in_client::Error::ToolError(
                    format!("failed to deserialize tool '{}' response: {}", #tool_name_str, e)
                ))?;
            ::std::result::Result::Ok(value)
        }
    };

    let doc = format!(
        "Call the server tool `{tool_name_str}` with typed arguments.\n\n\
         Generated from the `{method_name}` method in `#[mcp_client]`."
    );

    Ok(quote! {
        #(#clean_attrs)*
        #[doc = #doc]
        pub async fn #method_name(&self, #(
            #param_names: #param_types
        ),*) -> ::stand_in_client::error::Result<#return_ty> {
            let args = #json_args;
            #call_and_extract
        }
    })
}

/// Extract the `Ok` type from `Result<T, E>` in a function signature's return type.
///
/// Handles both `Result<String>` and `stand_in_client::Result<String>` (which
/// is `std::result::Result<T, stand_in_client::Error>`).
fn extract_ok_type(return_type: &ReturnType) -> syn::Result<Type> {
    let ty = match return_type {
        ReturnType::Default => {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "missing return type",
            ));
        }
        ReturnType::Type(_, ty) => ty,
    };

    let type_path = match ty.as_ref() {
        Type::Path(tp) => tp,
        _ => {
            return Err(syn::Error::new_spanned(
                ty,
                "return type must be a path like Result<T>",
            ));
        }
    };

    let last_seg = type_path
        .path
        .segments
        .last()
        .ok_or_else(|| syn::Error::new_spanned(ty, "empty return type path"))?;

    if last_seg.ident != "Result" {
        return Err(syn::Error::new_spanned(
            &last_seg.ident,
            format!("expected `Result<...>`, found `{}`", last_seg.ident),
        ));
    }

    let args = match &last_seg.arguments {
        PathArguments::AngleBracketed(ab) => &ab.args,
        _ => {
            return Err(syn::Error::new_spanned(
                last_seg,
                "Result must have generic arguments: Result<T>",
            ));
        }
    };

    let ok_type = args.first().ok_or_else(|| {
        syn::Error::new_spanned(last_seg, "Result must have at least one type argument")
    })?;

    match ok_type {
        GenericArgument::Type(ty) => Ok(ty.clone()),
        _ => Err(syn::Error::new_spanned(
            ok_type,
            "Result's first argument must be a type",
        )),
    }
}

/// Check if a syn `Type` is the `String` path (handles `String`, `std::string::String`, etc.).
fn is_path_string(ty: &Type) -> bool {
    let Type::Path(TypePath { qself: None, path }) = ty else {
        return false;
    };
    let Some(last_seg) = path.segments.last() else {
        return false;
    };
    last_seg.ident == "String"
}

/// Convert a snake_case string to PascalCase.
///
/// Used to derive the client struct name from the trait name
/// (e.g. `weather_api` → `WeatherApiClient`).
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

/// Strip `#[tool(...)]` from the trait items before re-emitting.
fn clean_trait_item(item: &TraitItem) -> TraitItem {
    match item {
        TraitItem::Fn(m) => {
            let mut m = m.clone();
            m.attrs = strip_tool_attrs(&m.attrs);
            TraitItem::Fn(m)
        }
        other => other.clone(),
    }
}

fn clean_trait_items(items: &[TraitItem]) -> Vec<TraitItem> {
    items.iter().map(clean_trait_item).collect()
}
