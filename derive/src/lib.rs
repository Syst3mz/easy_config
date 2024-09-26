use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn;
use syn::{Data, DeriveInput, Fields, Variant};
use syn::punctuated::Punctuated;
use syn::token::Comma;

fn struct_serialize(struct_name: &Ident, fields: &Fields) -> TokenStream {
    let Fields::Named(fields) = fields else {
        panic!("Unnamed fields are not yet supported.")};

    // Iterate over each field and generate Expression::Pair for each field
    let field_serializations = fields.named.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap(); // Get the field name
        let field_name_str = field_name.to_string(); // Convert field name to string
        quote! {
            Expression::Pair(#field_name_str.to_string(), Box::new(self.#field_name.serialize()))
        }
    });

    let field_deserializations = fields.named.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap(); // Get the field name
        let ty = f.ty.clone();

        quote! {
            #field_name: <#ty>::deserialize(expr.deserialize_get(stringify!(#field_name))?)?
        }
    });

    // Generate the implementation of the Serialize trait
    let gen = quote! {

        impl Config for #struct_name {
            fn serialize(&self) -> Expression {
                Expression::Collection(vec![
                    #(#field_serializations),*
                ])
            }

            fn deserialize(expr: Expression) -> Result<Self, easy_config::serialization::error::Error> where Self: Sized {
                Ok(Self {
                    #(#field_deserializations),*
                })
            }
        }
    };

    // Return the generated code as a TokenStream
    gen.into()
}

// fn enum_serialize(enum_name: &Ident, variants: Punctuated<Variant, Comma>) -> TokenStream {
//     for variant in variants {
//         variant.ident
//     }
// }

#[proc_macro_derive(Config)]
pub fn config(item: TokenStream) -> TokenStream {
    // Parse the input TokenStream into a DeriveInput syntax tree
    let ast: DeriveInput = syn::parse(item).unwrap();

    // Get the name of the struct
    let name = &ast.ident;

    // Match the data of the input to ensure it's a struct
    match ast.data {
        Data::Struct(s) => struct_serialize(name, &s.fields),
        Data::Enum(e) => unimplemented!(),
        Data::Union(_) => unimplemented!()
    }
}
