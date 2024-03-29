use quote::quote;
use syn::{self, Fields};

enum BigChunkVersion {
    Gt(u8),
    Lt(u8),
    Eq(u8),
    Ne(u8),
    Any,
}

impl BigChunkVersion {
    fn quote_operator(&self) -> proc_macro2::TokenStream {
        match self {
            Self::Gt(_) => quote!(>).into(),
            Self::Lt(_) => quote!(<).into(),
            Self::Eq(_) => quote!(==).into(),
            Self::Ne(_) => quote!(!=).into(),
            Self::Any => quote!().into(),
        }
    }

    fn parse(version_kind: &'static str, attrs: &Vec<syn::Attribute>) -> Option<Self> {
        match attrs.iter().find(|a| a.path.is_ident("big_chunk_version")) {
            Some(attr) => {
                if attr.tokens.is_empty() {
                    Some(BigChunkVersion::Any)
                } else {
                    match attr.parse_args::<syn::ExprBinary>() {
                        Ok(expr) => match *expr.left {
                            syn::Expr::Path(path) => {
                                if !path.path.is_ident(version_kind) {
                                    panic!()
                                }
                                match *expr.right {
                                    syn::Expr::Lit(lit) => match lit.lit {
                                        syn::Lit::Int(int) => match expr.op {
                                            syn::BinOp::Gt(_) => Some(BigChunkVersion::Gt(
                                                int.base10_parse::<u8>().unwrap(),
                                            )),
                                            syn::BinOp::Lt(_) => Some(BigChunkVersion::Lt(
                                                int.base10_parse::<u8>().unwrap(),
                                            )),
                                            syn::BinOp::Eq(_) => Some(BigChunkVersion::Eq(
                                                int.base10_parse::<u8>().unwrap(),
                                            )),
                                            syn::BinOp::Ne(_) => Some(BigChunkVersion::Ne(
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

struct TableAttr {
    typecode: Option<syn::Type>,
}

struct StructAttrs {
    big_chunk_major_version: Option<BigChunkVersion>,
    table: Option<TableAttr>,
    normal_chunk: bool,
}

impl StructAttrs {
    fn new(attrs: &Vec<syn::Attribute>) -> Self {
        Self {
            big_chunk_major_version: BigChunkVersion::parse("major", attrs),
            table: Self::parse_table(attrs),
            normal_chunk: Self::parse_normal_chunk(attrs),
        }
    }

    fn parse_table(attrs: &Vec<syn::Attribute>) -> Option<TableAttr> {
        match attrs.iter().find(|a| a.path.is_ident("table")) {
            Some(attr) => {
                if attr.tokens.is_empty() {
                    Some(TableAttr { typecode: None })
                } else {
                    Some(TableAttr {
                        typecode: Some(attr.parse_args::<syn::Type>().unwrap()),
                    })
                }
            }
            None => None,
        }
    }

    fn parse_normal_chunk(attrs: &Vec<syn::Attribute>) -> bool {
        match attrs.iter().find(|a| a.path.is_ident("normal_chunk")) {
            Some(_) => true,
            None => false,
        }
    }
}

struct FieldAttrs {
    underlying_type: Option<syn::Type>,
    padding: Option<syn::Type>,
    typecode: Option<syn::Type>,
    big_chunk_minor_version: Option<BigChunkVersion>,
}

impl FieldAttrs {
    fn new(field: &syn::Field) -> Self {
        Self {
            underlying_type: Self::parse_underlying_type(&field.attrs),
            padding: Self::parse_padding(&field.attrs),
            typecode: Self::parse_typecode(&field.attrs),
            big_chunk_minor_version: BigChunkVersion::parse("minor", &field.attrs),
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

    fn parse_typecode(attrs: &Vec<syn::Attribute>) -> Option<syn::Type> {
        match attrs.iter().find(|a| a.path.is_ident("table_field")) {
            Some(attr) => Some(attr.parse_args::<syn::Type>().unwrap()),
            None => None,
        }
    }
}

pub fn process_data_struct(
    data: &syn::DataStruct,
    ident: &syn::Ident,
    attrs: &Vec<syn::Attribute>,
) -> proc_macro2::TokenStream {
    let struct_attrs = StructAttrs::new(&attrs);
    match &data.fields {
        Fields::Named(fields) => {
            let fields_iter = fields.named.iter().map(|named_field| {
                let field_attrs = FieldAttrs::new(named_field);
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
                let field_deserialize = if field_attrs.underlying_type.is_some() {
                    let underlying_ty = &field_attrs.underlying_type.as_ref().unwrap();
                    quote!(#field_ty::from(#underlying_ty::deserialize(deserializer)?))
                } else {
                    quote!(#field_ty::deserialize(deserializer)?)
                };
                let padding_deserialize = if field_attrs.padding.is_some() {
                    let padding = &field_attrs.padding.as_ref().unwrap();
                    quote!(#padding::deserialize(deserializer)?;)
                } else {
                    quote!()
                };
                if field_attrs.typecode.is_some() {
                    let typecode = &field_attrs.typecode.as_ref().unwrap();
                    match field_attrs.big_chunk_minor_version {
                        Some(version) => match version {
                            BigChunkVersion::Any => {
                                quote!(
                                    typecode::#typecode => {
                                        #padding_deserialize
                                        table.#field_ident = #field_deserialize;
                                    }
                                )
                            }
                            BigChunkVersion::Eq(value)
                            | BigChunkVersion::Gt(value)
                            | BigChunkVersion::Lt(value)
                            | BigChunkVersion::Ne(value) => {
                                let quote_operator = version.quote_operator();
                                quote!(
                                    typecode::#typecode => {
                                        if chunk_version.minor() #quote_operator #value {
                                            #padding_deserialize
                                            table.#field_ident = #field_deserialize;
                                        }
                                    }
                                )
                            }
                        },
                        None => {
                            quote!(
                                typecode::#typecode => {
                                    #padding_deserialize
                                    table.#field_ident = #field_deserialize;
                                }
                            )
                        }
                    }
                } else {
                    match field_attrs.big_chunk_minor_version {
                        Some(version) => match version {
                            BigChunkVersion::Any => {
                                quote!(
                                    #field_ident: {
                                        #padding_deserialize
                                        #field_deserialize
                                    }
                                )
                            }
                            BigChunkVersion::Eq(value)
                            | BigChunkVersion::Gt(value)
                            | BigChunkVersion::Lt(value)
                            | BigChunkVersion::Ne(value) => {
                                let quote_operator = version.quote_operator();
                                quote!(
                                    #field_ident: {
                                        if chunk_version.minor() #quote_operator #value.into() {
                                            #padding_deserialize
                                            #field_deserialize
                                        } else {
                                            #field_ty::default()
                                        }
                                    }
                                )
                            }
                        },
                        None => {
                            quote!(
                                #field_ident: {
                                    #padding_deserialize
                                    #field_deserialize
                                }
                            )
                        }
                    }
                }
            });

            let struct_deserialize = if struct_attrs.table.is_some() {
                if struct_attrs.table.as_ref().unwrap().typecode.is_some() {
                    let typecode = struct_attrs.table.unwrap().typecode.unwrap();
                    quote!(
                        let mut table = Self::default();
                        let mut properties_chunk = Chunk::deserialize(deserializer)?;
                        if typecode::#typecode == properties_chunk.chunk_begin().typecode {
                            loop {
                                let mut chunk = Chunk::deserialize(&mut properties_chunk)?;
                                let deserializer = &mut chunk;
                                match deserializer.chunk_begin().typecode {
                                    #(#fields_iter)*
                                    typecode::ENDOFTABLE => {
                                        break;
                                    }
                                    _ => {
                                    }
                                }
                                chunk.seek(SeekFrom::End(1)).unwrap();
                            }
                        }
                        properties_chunk.seek(SeekFrom::End(1)).unwrap();
                        Ok(table)
                    )
                } else {
                    quote!(
                        let mut table = Self::default();
                        loop {
                            let mut chunk = Chunk::deserialize(deserializer)?;
                            let deserializer = &mut chunk;
                            match deserializer.chunk_begin().typecode {
                                #(#fields_iter)*
                                _ => {
                                    break;
                                }
                            }
                            chunk.seek(SeekFrom::End(1)).unwrap();
                        }
                        Ok(table)
                    )
                }
            } else {
                quote!(Ok(Self {#(#fields_iter),*}))
            };

            let chunk_deserialize = if struct_attrs.normal_chunk {
                quote!(
                    let mut chunk = chunk::Chunk::deserialize(deserializer)?;
                    let deserializer = &mut chunk;
                )
            } else {
                quote!()
            };

            let chunk_version_type = if struct_attrs.normal_chunk {
                quote!(NormalVersion)
            } else {
                quote!(BigVersion)
            };

            let deserialize_body = match struct_attrs.big_chunk_major_version {
                Some(major_version) => match major_version {
                    BigChunkVersion::Any => {
                        quote!(
                            #chunk_deserialize
                            let _chunk_version = chunk::#chunk_version_type::deserialize(deserializer)?;
                            #struct_deserialize
                        )
                    }
                    BigChunkVersion::Eq(value)
                    | BigChunkVersion::Gt(value)
                    | BigChunkVersion::Lt(value)
                    | BigChunkVersion::Ne(value) => {
                        let quote_operator = major_version.quote_operator();
                        quote!(
                            #chunk_deserialize
                            let chunk_version = chunk::#chunk_version_type::deserialize(deserializer)?;
                            if chunk_version.major() #quote_operator #value.into() {
                                #struct_deserialize
                            } else {
                                Ok(Self::default())
                            }
                        )
                    }
                },
                None => {
                    quote!(
                        #chunk_deserialize
                        #struct_deserialize
                    )
                }
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
