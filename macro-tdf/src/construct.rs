use proc_macro::{TokenStream};
use quote::{quote};
use proc_macro2::{Ident, Span};
use syn::{DeriveInput, Field};
use quote::ToTokens;


fn comp_ident(path: &syn::Path, name: &str) -> bool {
    path.is_ident(&Ident::new(name, Span::call_site()))
}


pub fn struct_map(input: &DeriveInput, fields: Vec<&syn::Field>) -> TokenStream {

    let input_type = &input.ident;

    let mut serialize_body = Vec::new();
    let mut serialize_result = Vec::new();
    let mut deserialize_body = Vec::new();

    for (_, field) in fields.iter().enumerate() {

        let field_name = &field.ident;

        match field_name {

             // If field has name
            Some(f) => {
                serialize_named_field(field, f, &mut serialize_body, &mut serialize_result);
                deserialize_named_field(field, f, &mut deserialize_body);
            },

            None => {}
        }

    }

    // Construct impl
    let out = quote! {

        #[automatically_derived]
        impl Serialize for #input_type {
            fn serialize(ser: &mut RTDFSerializer) -> Result<Self> {

                ser.map_start()?;

                #( #serialize_body )*

                ser.map_end()?;

                Ok(
                    Self {
                        #( #serialize_result )*
                    }
                )

            }
        }

        #[automatically_derived]
        impl Deserialize for #input_type {

            const TYPE: TDFToken = TDFToken::MapType;
        
            fn deserialize(&mut self, des: &mut RTDFDeserializer) -> Result<()> {
        
                des.stream.push(TDFToken::MapStart);

                #( #deserialize_body )*
        
                des.stream.push(TDFToken::MapEnd);
        
                Ok(())
            }
        
        }
        
    };

    out.into()

}



fn serialize_named_field(field: &Field, f: &Ident, serialize_body: &mut Vec<proc_macro2::TokenStream>, serialize_result: &mut Vec<proc_macro2::TokenStream>) {

    // Get references for field params
    let field_type = &field.ty;

    let token_type = field_type.to_token_stream();

    serialize_body.push(quote! {
        let (_, #f ) = ser.ser_field::< #token_type >()?;
    });

    serialize_result.push(quote! {
        #f,
    });

}


fn deserialize_named_field(field: &Field, f: &Ident, deserialize_body: &mut Vec<proc_macro2::TokenStream>) {

    let field_attrs = &field.attrs;

    let mut name_string: String = format!("{}", f);

    // Rename attribute
    // Like #[rename("Label")]
    for attr in field_attrs {
        if comp_ident(&attr.path, "rename") {
            let new_name: syn::LitStr = attr.parse_args().unwrap();
            name_string = new_name.value();
        }
    }

    deserialize_body.push(quote! {

        des.des_field( #name_string , &mut self.#f )?;

    });

}