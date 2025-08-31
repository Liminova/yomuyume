use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

/// Derive macro for implementing redb::Value for structs
///
/// Requires the struct to also be #[derive(Serialize, Deserialize)]
#[proc_macro_derive(RedbJsonValue)]
pub fn derive_redb_json_value(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let generated = quote! {
        impl redb::Value for #name {
            type SelfType<'a> = #name;
            type AsBytes<'a> = Vec<u8>;
            fn fixed_width() -> Option<usize> { None }
            fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
            where Self: 'a {
                serde_json::from_slice(data)
                    .unwrap_or_else(|e| panic!("Failed to deserialize {} from DB: {e}", stringify!(#name)))
            }
            fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
            where Self: 'b {
                serde_json::to_vec(value)
                    .unwrap_or_else(|e| panic!("Failed to serialize {} to DB: {e}", stringify!(#name)))
            }
            fn type_name() -> redb::TypeName {
                redb::TypeName::new(stringify!(#name))
            }
        }
    };
    generated.into()
}
