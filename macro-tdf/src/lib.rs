use proc_macro::{TokenStream};
use syn::{DeriveInput, Data, Fields, parse_macro_input, Ident, Field};
use quote::{quote};
use proc_macro2::{Span};

mod construct;
use construct::*;

use itertools::Itertools;

fn comp_ident(path: &syn::Path, name: &str) -> bool {
    path.is_ident(&Ident::new(name, Span::call_site()))
}


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

    //fields = fields.into_iter().sorted_by(|a, b| Ord::cmp(&ind_to_string(&a), &ind_to_string(&b))).collect();

    struct_map(struct_type, fields)

}


fn ind_to_string(field: &Field) -> String {


    let mut name_string = match &field.ident {
        Some(i) => i.to_string(),
        None => String::new()
    };

    // Rename attribute
    // Like #[rename("Label")]
    for attr in &field.attrs {

        if comp_ident(&attr.path, "rename") {

            let new_name: syn::LitStr = attr.parse_args().unwrap();
            name_string = new_name.value();

        }

    }

    name_string

}