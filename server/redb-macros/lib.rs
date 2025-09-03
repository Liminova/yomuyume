use heck::ToSnakeCase;
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, ItemType, Type, TypeTuple, parse_macro_input, spanned::Spanned};

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
                    .unwrap_or_else(|e| panic!("can't deserialize {} from DB: {e}", stringify!(#name)))
            }
            fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
            where Self: 'b {
                serde_json::to_vec(value)
                    .unwrap_or_else(|e| panic!("can't serialize {} to DB: {e}", stringify!(#name)))
            }
            fn type_name() -> redb::TypeName {
                redb::TypeName::new(stringify!(#name))
            }
        }
    };
    generated.into()
}

#[proc_macro_attribute]
pub fn key_function(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // parse the alias
    let alias: ItemType = parse_macro_input!(item as ItemType);
    let alias_ident = &alias.ident;
    let alias_ty = *alias.ty.clone();
    // must be a tuple
    let elems = match alias_ty {
        Type::Tuple(TypeTuple { elems, .. }) => elems.into_iter().collect::<Vec<_>>(),
        _ => {
            return syn::Error::new_spanned(alias.ty, "expected a tuple type")
                .to_compile_error()
                .into();
        }
    };
    // build fn name: FooBar -> foo_bar
    let fn_name_str = alias_ident.to_string().to_snake_case();
    let fn_ident = syn::Ident::new(&fn_name_str, alias_ident.span());
    // for each tuple element extract its type and a snake-cased param name
    let params = elems
        .iter()
        .map(|ty| {
            let ty_str = quote!(#ty).to_string();
            let mut name = ty_str.to_snake_case();
            if let Some(stripped) = name.strip_prefix("option_") {
                name = stripped.to_string();
            }
            let param_ident = syn::Ident::new(&name, ty.span());
            (param_ident, ty)
        })
        .collect::<Vec<_>>();
    let param_idents_vec: Vec<_> = params.iter().map(|(id, _)| id.clone()).collect();
    let param_tys_vec: Vec<_> = params.iter().map(|(_, ty)| (*ty).clone()).collect();
    let param_idents = &param_idents_vec;
    let param_tys = &param_tys_vec;
    // generate
    let expanded = quote! {
        #alias

        pub const fn #fn_ident( #(#param_idents: #param_tys),* ) -> #alias_ident {
            ( #(#param_idents),* )
        }
    };
    TokenStream::from(expanded)
}
