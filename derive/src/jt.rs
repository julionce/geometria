use quote::quote;
use syn::Fields;

pub fn process_data_struct(
    data: &syn::DataStruct,
    ident: &syn::Ident,
    _attrs: &Vec<syn::Attribute>,
) -> proc_macro2::TokenStream {
    match &data.fields {
        Fields::Named(fields) => {
            let fields_iter = fields.named.iter().map(|named_field| {
                let field_ident = named_field.ident.as_ref().unwrap();
                let field_ty = match &named_field.ty {
                    syn::Type::Array(value) => {
                        quote!(<#value>)
                    }
                    syn::Type::Path(value) => {
                        quote!(#value)
                    }
                    _ => panic!(),
                };
                quote!(
                    #field_ident: {
                        #field_ty::deserialize(deserializer)?
                    }
                )
            });
            quote! {
                impl Deserialize for #ident
                {
                    type Error = String;

                    fn deserialize<D>(deserializer: &mut D) -> Result<Self, Self::Error>
                    where D: Deserializer
                    {
                        Ok(Self {#(#fields_iter), *})
                    }
                }
            }
        }
        _ => {
            quote!()
        }
    }
}
