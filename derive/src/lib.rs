use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse_macro_input, Data, DeriveInput, Fields};

enum MajorChunkVersion {
    Gt(u8),
    Lt(u8),
    Eq(u8),
    Ne(u8),
    Any,
}

impl MajorChunkVersion {
    fn quote_operator(&self) -> proc_macro2::TokenStream {
        match self {
            Self::Gt(_) => quote!(>).into(),
            Self::Lt(_) => quote!(<).into(),
            Self::Eq(_) => quote!(==).into(),
            Self::Ne(_) => quote!(!=).into(),
            Self::Any => quote!().into(),
        }
    }
}

struct StructAttrs {
    major_chunk_version: Option<MajorChunkVersion>,
}

impl StructAttrs {
    fn new(attrs: &Vec<syn::Attribute>) -> Self {
        Self {
            major_chunk_version: Self::parse_major_chunk_version(attrs),
        }
    }

    fn parse_major_chunk_version(attrs: &Vec<syn::Attribute>) -> Option<MajorChunkVersion> {
        match attrs.iter().find(|a| a.path.is_ident("chunk_version")) {
            Some(attr) => {
                if attr.tokens.is_empty() {
                    Some(MajorChunkVersion::Any)
                } else {
                    match attr.parse_args::<syn::ExprBinary>() {
                        Ok(expr) => match *expr.left {
                            syn::Expr::Path(path) => {
                                if !path.path.is_ident("major") {
                                    panic!()
                                }
                                match *expr.right {
                                    syn::Expr::Lit(lit) => match lit.lit {
                                        syn::Lit::Int(int) => match expr.op {
                                            syn::BinOp::Gt(_) => Some(MajorChunkVersion::Gt(
                                                int.base10_parse::<u8>().unwrap(),
                                            )),
                                            syn::BinOp::Lt(_) => Some(MajorChunkVersion::Lt(
                                                int.base10_parse::<u8>().unwrap(),
                                            )),
                                            syn::BinOp::Eq(_) => Some(MajorChunkVersion::Eq(
                                                int.base10_parse::<u8>().unwrap(),
                                            )),
                                            syn::BinOp::Ne(_) => Some(MajorChunkVersion::Ne(
                                                int.base10_parse::<u8>().unwrap(),
                                            )),
                                            _ => panic!(),
                                        },
                                        _ => panic!(),
                                    },
                                    _ => panic!(),
                                }
                            }
                            _ => panic!(),
                        },
                        _ => panic!(),
                    }
                }
            }
            None => None,
        }
    }
}

struct FieldAttrs {
    underlying_type: Option<syn::Type>,
    padding: Option<syn::Type>,
}

impl FieldAttrs {
    fn new(field: &syn::Field) -> Self {
        Self {
            underlying_type: Self::parse_underlying_type(&field.attrs),
            padding: Self::parse_padding(&field.attrs),
        }
    }

    fn parse_underlying_type(attrs: &Vec<syn::Attribute>) -> Option<syn::Type> {
        match attrs.iter().find(|a| a.path.is_ident("underlying_type")) {
            Some(attr) => Some(attr.parse_args::<syn::Type>().unwrap()),
            None => None,
        }
    }

    fn parse_padding(attrs: &Vec<syn::Attribute>) -> Option<syn::Type> {
        match attrs.iter().find(|a| a.path.is_ident("padding")) {
            Some(attr) => Some(attr.parse_args::<syn::Type>().unwrap()),
            None => None,
        }
    }
}

#[proc_macro_derive(Deserialize, attributes(chunk_version, underlying_type, padding))]
pub fn deserialize_derive(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident, data, attrs, ..
    }: DeriveInput = parse_macro_input!(input as DeriveInput);
    match data {
        Data::Struct(data_struct) => {
            let struct_attrs = StructAttrs::new(&attrs);
            match data_struct.fields {
                Fields::Named(fields) => {
                    let fields_iter = fields.named.iter().map(|named_field| {
                        let field_attrs = FieldAttrs::new(named_field);
                        let field_ident = named_field.ident.as_ref().unwrap();
                        let field_ty = &named_field.ty;
                        let field_deserialize = if field_attrs.underlying_type.is_some() {
                            let underlying_ty = &field_attrs.underlying_type.as_ref().unwrap();
                            quote!(#field_ty::from(#underlying_ty::deserialize(deserializer)?))
                        } else {
                            quote!(#field_ty::deserialize(deserializer)?)
                        };
                        if field_attrs.padding.is_some() {
                            let padding = &field_attrs.padding.as_ref().unwrap();
                            quote!(
                                #field_ident: {
                                    #padding::deserialize(deserializer)?;
                                    #field_deserialize
                                }
                            )
                        } else {
                            quote!(
                                #field_ident: #field_deserialize
                            )
                        }
                    });
                    let fields_deserialize = quote!(#(#fields_iter),*);
                    let deserialize_body = match struct_attrs.major_chunk_version {
                        Some(major_version) => match major_version {
                            MajorChunkVersion::Any => {
                                quote!(
                                    let _chunk_version = chunk::Version::deserialize(deserializer)?;
                                    Ok(Self {#fields_deserialize})
                                )
                            }
                            MajorChunkVersion::Eq(value)
                            | MajorChunkVersion::Gt(value)
                            | MajorChunkVersion::Lt(value)
                            | MajorChunkVersion::Ne(value) => {
                                let quote_operator = major_version.quote_operator();
                                quote!(
                                    let chunk_version = chunk::Version::deserialize(deserializer)?;
                                    if chunk_version.major() #quote_operator #value {
                                        Ok(Self {#fields_deserialize})
                                    } else {
                                        Ok(Self::default())
                                    }
                                )
                            }
                        },
                        None => quote!(
                            Ok(Self {#fields_deserialize})
                        ),
                    };
                    quote! {
                        impl<'de, D> Deserialize<'de, D> for #ident where D: Deserializer,
                        {
                            type Error = String;

                            fn deserialize(deserializer: &mut D) -> Result<Self, Self::Error> {
                                #deserialize_body
                            }
                        }
                    }
                }
                _ => {
                    quote!()
                }
            }
        }
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
