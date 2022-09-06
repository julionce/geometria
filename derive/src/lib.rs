use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Deserialize)]
pub fn deserialize_derive(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. }: DeriveInput = parse_macro_input!(input as DeriveInput);
    match data {
        Data::Struct(data_struct) => match data_struct.fields {
            Fields::Named(fields) => {
                let fields_iter = fields.named.iter().map(|named_field| {
                    let field_ident = named_field.ident.as_ref().unwrap();
                    let field_ty = &named_field.ty;
                    quote!(
                        #field_ident: #field_ty::deserialize(deserializer)?,
                    )
                });
                quote! {
                    impl<'de, D> Deserialize<'de, D> for #ident where D: Deserializer,
                    {
                        type Error = String;

                        fn deserialize(deserializer: &mut D) -> Result<Self, Self::Error> {
                            Ok(Self { #(#fields_iter)*})
                        }
                    }
                }
            }
            _ => {
                quote!()
            }
        },
        _ => {
            quote!()
        }
    }
    .into()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
