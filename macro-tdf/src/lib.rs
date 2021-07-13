use proc_macro::{TokenStream};
use syn::{DeriveInput, Data, Fields, parse_macro_input, Ident};
use quote::{quote};

mod construct;
use construct::*;

use itertools::Itertools;

#[proc_macro_derive(Pack, attributes(rename))]
pub fn parse_macro(input: TokenStream) -> TokenStream {

    let mut input = parse_macro_input!(input as DeriveInput);

    let input_type = &input.ident;
    
    let struct_type = quote! {
        #input_type
    };
    
    let data_struct = match &mut input.data {

        // Struct
        Data::Struct(data_struct) => data_struct,

        // Bail on unsupported format

        Data::Enum(_) => {
            panic!("Cannot derive for rust enum")
        },
        
        Data::Union(_) => {
            panic!("Cannot derive for rust union")
        }

    };

    let data_struct_fields = &mut data_struct.fields;

    let mut fields = match data_struct_fields {

        Fields::Unnamed(_) => {
            panic!("Only named struct is supported!")
        }

        Fields::Named(fields) => {
            fields.named.iter_mut().collect()
        }

        Fields::Unit => vec![]

    };

    fields = fields.into_iter().sorted_by(|a, b| Ord::cmp(&ind_to_string(&a.ident), &ind_to_string(&b.ident))).collect();

    struct_map(struct_type, fields)

}


fn ind_to_string(optional_ident: &Option<Ident>) -> String {
    match optional_ident{
        Some(i) => i.to_string(),
        None => String::new()
    }
}