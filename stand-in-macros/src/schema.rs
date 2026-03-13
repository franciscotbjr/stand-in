//! Rust type → JSON Schema inference for proc macro expansion.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;

/// Generate a `quote` token stream that produces the JSON Schema type string
/// for a given Rust type. Returns `(schema_tokens, is_required)`.
///
/// `is_required` is `false` when the type is `Option<T>`.
pub fn type_to_schema(ty: &Type) -> (TokenStream, bool) {
    match ty {
        Type::Path(type_path) => {
            let segment = match type_path.path.segments.last() {
                Some(s) => s,
                None => return (quote! { serde_json::json!({"type": "string"}) }, true),
            };

            let ident = segment.ident.to_string();

            match ident.as_str() {
                "String" | "str" => (quote! { serde_json::json!({"type": "string"}) }, true),
                "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32"
                | "u64" | "u128" | "usize" => {
                    (quote! { serde_json::json!({"type": "integer"}) }, true)
                }
                "f32" | "f64" => (quote! { serde_json::json!({"type": "number"}) }, true),
                "bool" => (quote! { serde_json::json!({"type": "boolean"}) }, true),
                "Option" => {
                    // Extract inner type from Option<T>
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                            let (inner_schema, _) = type_to_schema(inner_ty);
                            return (inner_schema, false);
                        }
                    }
                    (quote! { serde_json::json!({"type": "string"}) }, false)
                }
                _ => (quote! { serde_json::json!({"type": "string"}) }, true),
            }
        }
        _ => (quote! { serde_json::json!({"type": "string"}) }, true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    fn schema_str(ty: Type) -> String {
        let (tokens, _) = type_to_schema(&ty);
        tokens.to_string()
    }

    #[test]
    fn test_string_schema() {
        let ty: Type = parse_quote!(String);
        let (_, required) = type_to_schema(&ty);
        assert!(required);
        assert!(schema_str(ty).contains("string"));
    }

    #[test]
    fn test_i32_schema() {
        let ty: Type = parse_quote!(i32);
        let (_, required) = type_to_schema(&ty);
        assert!(required);
        assert!(schema_str(ty).contains("integer"));
    }

    #[test]
    fn test_f64_schema() {
        let ty: Type = parse_quote!(f64);
        let (_, required) = type_to_schema(&ty);
        assert!(required);
        assert!(schema_str(ty).contains("number"));
    }

    #[test]
    fn test_bool_schema() {
        let ty: Type = parse_quote!(bool);
        let (_, required) = type_to_schema(&ty);
        assert!(required);
        assert!(schema_str(ty).contains("boolean"));
    }

    #[test]
    fn test_option_not_required() {
        let ty: Type = parse_quote!(Option<String>);
        let (_, required) = type_to_schema(&ty);
        assert!(!required);
        assert!(schema_str(ty).contains("string"));
    }
}
