use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

mod rhino;

#[proc_macro_derive(
    RhinoDeserialize,
    attributes(
        big_chunk_version,
        underlying_type,
        padding,
        table,
        table_field,
        normal_chunk
    )
)]
pub fn rhino_deserialize_derive(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident, data, attrs, ..
    }: DeriveInput = parse_macro_input!(input as DeriveInput);
    match data {
        Data::Struct(data_struct) => rhino::process_data_struct(&data_struct, &ident, &attrs),
        _ => {
            quote!()
        }
    }
    .into()
}
