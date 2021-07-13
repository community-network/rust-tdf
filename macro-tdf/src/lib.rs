use proc_macro::{TokenStream};
use syn::{DeriveInput, Data, Fields, parse_macro_input};


mod construct;
use construct::*;

#[proc_macro_derive(Pack, attributes(rename))]
pub fn parse_macro(input: TokenStream) -> TokenStream {

    let input = parse_macro_input!(input as DeriveInput);
    
    let fields: Vec<_> = match input.data {

        // Struct
        Data::Struct(ref data_struct) => match data_struct.fields {

            Fields::Unnamed(ref _fields) => {
                panic!("Only named struct is supported!")
            }

            Fields::Named(ref fields) => {
                fields.named.iter().collect()
            }

            Fields::Unit => vec![]

        },

        // Bail on unsupported format

        Data::Enum(ref _data_enum) => {
            panic!("Cannot derive for rust enum")
        },
        
        Data::Union(_) => {
            panic!("Cannot derive for rust union")
        }

    };

    struct_map(&input, fields)

}
