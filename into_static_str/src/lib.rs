// In a new crate named `app_event_macro`

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};

#[proc_macro_derive(IntoStaticStr)]
pub fn into_static_str(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let variants = match input.data {
        Data::Enum(ref data) => &data.variants,
        _ => panic!("IntoStaticStr can only be derived for enums"),
    };

    let match_arms = variants.iter().map(|v| {
        let variant_name = &v.ident;
        match &v.fields {
            Fields::Unit => {
                quote! {
                    #name::#variant_name => stringify!(#name::#variant_name)
                }
            },
            _ => panic!("IntoStaticStr can only be derived for unit variants"),
        }
    });

    let expanded = quote! {
        impl Into<&'static str> for #name {
            fn into(self) -> &'static str {
                match self {
                    #(#match_arms),*
                }
            }
        }
    };

    TokenStream::from(expanded)
}