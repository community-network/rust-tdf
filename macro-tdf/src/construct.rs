use proc_macro::{TokenStream};
use quote::{quote};
use proc_macro2::{Ident, Span};
use syn::{Field};
use quote::ToTokens;


fn comp_ident(path: &syn::Path, name: &str) -> bool {
    path.is_ident(&Ident::new(name, Span::call_site()))
}

fn is_field_type_option(field_type: &mut syn::Type) -> bool {
    match field_type {
        syn::Type::Path(p) => {
            if p.path.segments.len() == 0 {
                return false;
            }
                       
            let is_option = &p.path.segments[0].ident.to_string() == "Option";
            if is_option {
                p.path.segments.pop();
            }

            return is_option
        },
        _ => false
    }
}

fn name_string_with_attributes(field_attrs: &Vec<syn::Attribute>, initial_name: String ) -> String {

    let mut name_string = initial_name;

    // Rename attribute
    // Like #[rename("Label")]
    for attr in field_attrs {

        if comp_ident(&attr.path, "rename") {

            let new_name: syn::LitStr = attr.parse_args().unwrap();
            name_string = new_name.value();

        }

    }

    name_string
}

pub fn struct_map(struct_type: proc_macro2::TokenStream, fields: Vec<&mut syn::Field>) -> TokenStream {

    let mut serialize_body = Vec::new();
    let mut serialize_result = Vec::new();
    let mut deserialize_body = Vec::new();

    for field in fields {

        let field_name = field.ident.clone();

        match field_name {

             // If field has name
            Some(f) => {

                let field_type = &mut field.ty;
                let is_optional = is_field_type_option(field_type);

                serialize_named_field(field, &f, &mut serialize_body, &mut serialize_result, is_optional);
                deserialize_named_field(field, &f, &mut deserialize_body, is_optional);
            },

            None => {}
        }

    }

    // Construct impl
    let out = quote! {

        #[automatically_derived]
        impl Serialize for #struct_type {
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
        impl Deserialize for #struct_type {

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



fn serialize_named_field(field: &Field, f: &Ident, serialize_body: &mut Vec<proc_macro2::TokenStream>, serialize_result: &mut Vec<proc_macro2::TokenStream>, is_optional: bool) {


    let token_type = field.ty.to_token_stream();

    let name_string = name_string_with_attributes(
        &field.attrs, 
        format!("{}", f)
    );


    let ser_body_field = if is_optional {
        quote! {
            let #f = ser.ser_field_optional::< #token_type >( TDFToken::Label( #name_string.into() ) )?;
        }
    } else {
        quote! {
            let (_, #f ) = ser.ser_field::< #token_type >()?;
        }
    };
    
    serialize_body.push(ser_body_field);

    serialize_result.push(

        quote! {
            #f,
        }

    );

}


fn deserialize_named_field(field: &Field, f: &Ident, deserialize_body: &mut Vec<proc_macro2::TokenStream>, is_optional: bool ) {

    let name_string = name_string_with_attributes(
        &field.attrs, 
        format!("{}", f)
    );

    let optional_quote = match is_optional {
        true => quote! {
            match &mut self.#f {
                Some(#f) => des.des_field( #name_string , #f )?,
                None => {}
            }
        },
        false => quote! {
            des.des_field( #name_string , &mut self.#f )?;
        }
    };

    deserialize_body.push(optional_quote);

}