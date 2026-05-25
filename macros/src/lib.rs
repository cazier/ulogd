use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(New)]
pub fn derive_new(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    if let Data::Struct(data) = input.data {
        let mut attrs = vec![];
        let mut params = vec![];

        if let Fields::Named(fields) = data.fields {
            for field in fields.named {
                let field_name = field.ident.unwrap();
                let field_type = field.ty;

                attrs.push(quote! {
                    #field_name
                });
                params.push(quote! {
                    #field_name: #field_type
                })
            }
        }

        let method = quote! {
            impl #name {
                pub fn new(#(#params), *) -> Self {
                    Self {
                        #(#attrs),*
                    }
                }
            }
        };

        return TokenStream::from(method);
    }
    return TokenStream::from(quote! {});
}
