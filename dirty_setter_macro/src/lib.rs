use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, DeriveInput, Data, Fields, FieldsNamed};

#[proc_macro_derive(DirtySetter, attributes(dirty_setter))]
pub fn dirty_macro_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    impl_dirty_macro(&ast)
}

fn impl_dirty_macro(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    
    let fields = match &ast.data {
        Data::Struct(data) => {
            match &data.fields {
                Fields::Named(FieldsNamed { named, .. }) => named,
                _ => panic!("This macro only works with structs that have named fields"),
            }
        },
        _ => panic!("This macro only works with structs"),
    };

    let dirty_fields: Vec<_> = fields.iter()
        .filter(|field| field.attrs.iter().any(|attr| attr.path().is_ident("dirty_setter")))
        .collect();

    let setters = fields.iter()
        .filter(|field| field.attrs.iter().any(|attr| attr.path().is_ident("dirty_setter")))
        .map(|field| {
            let field_name = &field.ident;
            let field_type = &field.ty;
            let setter_name = format_ident!("set_{}", field_name.as_ref().unwrap());

            quote! {
                pub fn #setter_name(&mut self, value: #field_type) -> &mut Self {
                    self.#field_name = value.clone();
                    let value = serde_json::json!({
                        stringify!(#field_name): value
                    });

                    let id = self.id().value().to_owned();
                    get_render_control().add_message(UpdateMessage::Update(UpdateBody::new(
                        UpdateType::ObjectUpdate(id),
                        value
                    )));

                    self.set_dirty();
                    self
                }
            }
        });

    let field_names = dirty_fields.iter().map(|field| &field.ident);
    let field_types = dirty_fields.iter().map(|field| &field.ty);

    let batch_setter_field_names = field_names.clone();
    let batch_setter_field_types = field_types.clone();

    let dirty_field_names = field_names.clone();
    
    let batch_setter = quote! {
        pub fn set_multiple(&mut self, updates: DirtyUpdates) -> &mut Self {
            let mut update = serde_json::json!({});
            #(
                if let Some(value) = updates.#field_names {
                    self.#field_names = value.clone();
                    update[stringify!(#field_names)] = serde_json::json!(value);

                }
            )*

            if !update.as_object().unwrap().is_empty() {
                let id = self.id().value().to_owned();
                get_render_control().add_message(UpdateMessage::Update(UpdateBody::new(
                    UpdateType::ObjectUpdate(id),
                    update
                )));

                self.set_dirty();
            }
            self
        }
    };

    let update_method = quote! {
        fn update(&mut self, data: serde_json::Value) {
            let update_value: DirtyUpdates = serde_json::from_value(data).unwrap();
            #(
                if let Some(value) = update_value.#dirty_field_names {
                    self.#dirty_field_names = value;
                }
            )*
        }
    };

    
    let updates_struct = quote! {
        #[derive(Default, serde::Deserialize)]
        pub struct DirtyUpdates {
            #(pub #batch_setter_field_names: Option<#batch_setter_field_types>,)*
        }
    };

    let gen = quote! {
        #updates_struct

        impl #name {
            #(#setters)*
            #batch_setter
            #update_method
        }
    };
    
    gen.into()
}
