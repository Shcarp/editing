use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, DeriveInput, Data, Fields, FieldsNamed};

#[proc_macro_derive(Dirty, attributes(dirty_setter))]
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

    let setters = fields.iter()
        .filter(|field| field.attrs.iter().any(|attr| attr.path().is_ident("dirty_setter")))
        .map(|field| {
            let field_name = &field.ident;
            let field_type = &field.ty;
            let setter_name = format_ident!("set_{}", field_name.as_ref().unwrap());

            quote! {
                pub fn #setter_name(&mut self, value: #field_type) -> &mut Self {
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

    let gen = quote! {
        impl #name {
            #(#setters)*
        }
    };
    
    gen.into()
}
