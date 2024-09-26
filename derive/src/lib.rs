use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn;
use syn::{Data, DeriveInput, Field, Fields, FieldsNamed, FieldsUnnamed, Type, Variant};
use syn::punctuated::Punctuated;
use syn::token::Comma;


type SerializeDeserialize = (Box<dyn ToTokens>, Box<dyn ToTokens>);

fn serialize_numbered_field(number: usize) -> impl ToTokens {
    let number = syn::Index::from(number);

    quote! {
        self.#number.serialize()
    }
}
fn deserialize_numbered_field(ty: &Type, arg_number: usize, expected_field_count: usize) -> impl ToTokens {
    quote! {
        <#ty>::deserialize(fields.next().ok_or(Error::WrongNumberOfElements(#expected_field_count, #arg_number))?)?
    }
}

fn serialize_unnamed_struct(type_name: &Ident, fields: &FieldsUnnamed) -> SerializeDeserialize {

    let field_serializations = fields.unnamed.iter()
        .enumerate()
        .map(|(number, _)| serialize_numbered_field(number));

    let length_of_fields = fields.unnamed.len();
    let field_deserializations = fields.unnamed.iter()
        .enumerate()
        .map(|(number, f)| deserialize_numbered_field(&f.ty, number + 1, length_of_fields));

    (
        Box::new(quote! {
            Expression::Collection(vec![
                #(#field_serializations),*
            ])
        }),
        Box::new(quote! {
            let mut fields = expr
                .clone()
                .into_deserialization_iterator()
                .ok_or(Error::ExpectedTypeGot(type_name::<#type_name>().to_string(), expr.pretty()))?;

            Ok(Self(
                #(#field_deserializations),*
            ))
        })
    )
}

fn serialize_named_field(field: &Field) -> impl ToTokens {
    let field_name = field.ident.as_ref().unwrap(); // Get the field name
    let field_name_str = field_name.to_string(); // Convert field name to string
    quote! {
        easy_config::parser::Expression::Pair(#field_name_str.to_string(), Box::new(self.#field_name.serialize()))
    }
}
fn deserialize_named_field(field: &Field) -> impl ToTokens {
    let field_name = field.ident.as_ref().unwrap(); // Get the field name
    let ty = &field.ty;

    quote! {
        #field_name: <#ty>::deserialize(expr.deserialize_get(stringify!(#field_name))?)?
    }
}

fn serialize_named_fields(fields: &FieldsNamed) -> SerializeDeserialize {
    // Iterate over each field and generate Expression::Pair for each field
    let field_serializations = fields.named.iter().map(|f| serialize_named_field(f));
    let field_deserializations = fields.named.iter().map(|f| deserialize_named_field(f));

    (
        Box::new(quote! {
            Expression::Collection(vec![
                #(#field_serializations),*
            ])
        }),
        Box::new(quote! {
            Ok(Self {
                #(#field_deserializations),*
            })
        })
    )
}

fn serialize_unit_struct() -> SerializeDeserialize {
    (
        Box::new(quote! {
            easy_config::parser::expression::Expression::Collection(vec![])
        }),
        Box::new(quote! {
            match expr {
            easy_config::parser::expression::Expression::Presence(_) => Err(easy_config::serialization::error::Error::ExpectedCollectionGot(expr.pretty())),
            easy_config::parser::expression::Expression::Pair(_, _) => Err(easy_config::serialization::error::Error::ExpectedCollectionGot(expr.pretty())),
            easy_config::parser::expression::Expression::Collection(c) => {
                if !c.is_empty() {
                    Err(easy_config::serialization::error::Error::WrongNumberOfElements(0, c.len()))
                } else {
                    Ok(Self)
                }
            }
        }
        })
    )
}

fn serialize_struct_fields(type_name: &Ident, fields: &Fields) -> SerializeDeserialize {
    match fields {
        Fields::Named(fields) => serialize_named_fields(fields),
        Fields::Unnamed(fields) => serialize_unnamed_struct(type_name, fields),
        Fields::Unit => serialize_unit_struct()
    }
}

fn impl_config(name: &Ident, serialize_body: impl ToTokens, deserialize_body: impl ToTokens) -> TokenStream {
    // Generate the implementation of the Serialize trait
    let gen = quote! {
        use easy_config::serialization::DeserializeExtension;
        use easy_config::serialization::error::Error;
        use std::any::type_name;

        impl Config for #name {
            fn serialize(&self) -> Expression {
                #serialize_body
            }

            fn deserialize(expr: Expression) -> Result<Self, easy_config::serialization::error::Error> where Self: Sized {
                #deserialize_body
            }
        }
    };

    // Return the generated code as a TokenStream
    gen.into()
}


fn serialize_named_unit(enum_name: &Ident, variant_name: &Ident) -> SerializeDeserialize {
    (Box::new(quote!{#enum_name::#variant_name => Expression::Presence(stringify!(#variant_name).to_string())}),
     Box::new(quote! {
         if let Some(name) = current.clone() {
             if let Some(name) = name.release() {
                 if name == stringify!(#variant_name) {
                     current = fields.next();
                     return Ok(#enum_name::#variant_name);
                }
             }
         }
     }))
}

fn name_from_index(index: usize) -> String {
    match index {
        0  => "a".to_string(),
        1  => "b".to_string(),
        2  => "c".to_string(),
        3  => "d".to_string(),
        4  => "e".to_string(),
        5  => "f".to_string(),
        6  => "g".to_string(),
        7  => "h".to_string(),
        8  => "i".to_string(),
        9  => "j".to_string(),
        10 => "k".to_string(),
        11 => "l".to_string(),
        12 => "m".to_string(),
        13 => "n".to_string(),
        14 => "o".to_string(),
        15 => "p".to_string(),
        16 => "q".to_string(),
        17 => "r".to_string(),
        18 => "s".to_string(),
        19 => "t".to_string(),
        20 => "u".to_string(),
        21 => "v".to_string(),
        22 => "w".to_string(),
        23 => "x".to_string(),
        24 => "y".to_string(),
        25 => "z".to_string(),
        _ => panic!("You have a enum variant with 26 unnamed fields...make your code better.")
    }
}

fn serialize_unnamed_enum_fields(enum_name: &Ident, variant_name: &Ident, fields: &FieldsUnnamed) -> SerializeDeserialize {
    let serialize_field_types = fields.unnamed.iter()
        .enumerate()
        .map(|(n, _)| Ident::new(name_from_index(n).as_str(), Span::call_site()));
    let serialize_field_types_clone = serialize_field_types.clone();
    (
        Box::new(quote!{#enum_name::#variant_name(#(#serialize_field_types),*) => Expression::Collection(vec![
            Expression::Presence(stringify!(#variant_name).to_string()),
            #(#serialize_field_types_clone.serialize()),*
        ])}),
        Box::new(quote! {#variant_name})
    )
}

fn serialize_named_enum_fields(enum_name: &Ident, variant_name: &Ident, fields: &FieldsNamed) -> SerializeDeserialize {
    let serialized_names: Vec<&Ident> = fields.named.iter()
        .map(|x| x.ident.as_ref().unwrap())
        .collect();
    let serialize_fields = serialized_names.iter().map(|x| quote! {
        Expression::Pair(stringify!(#x).to_string(), Box::new(#x.serialize()))
    });
    (
        Box::new(quote!{#enum_name::#variant_name{#(#serialized_names),*} => Expression::Collection(vec![
            Expression::Presence(stringify!(#variant_name).to_string()),
            #(#serialize_fields),*
        ])}),
        Box::new(quote! {#variant_name})
    )
}

fn serialize_enum(enum_name: &Ident, variants: &Punctuated<Variant, Comma>) -> SerializeDeserialize {
    let (serializations, deserializations): (Vec<_>, Vec<_>)  = variants.iter().map(|x| {
        let variant_name = &x.ident;
        let (serialize_fields, deserialize_fields) = match &x.fields {
            Fields::Named(n) => serialize_named_enum_fields(enum_name, variant_name, n),
            Fields::Unnamed(u) => serialize_unnamed_enum_fields(enum_name, variant_name, u),
            Fields::Unit => serialize_named_unit(enum_name, variant_name),
        };

        (serialize_fields, deserialize_fields)
    }).unzip();

    let serializations = serializations.into_iter();
    let deserializations = deserializations.into_iter();

    (
        Box::new(quote! {
            match self {
                #(#serializations),*
            }
        }),
        Box::new(quote! {
            let mut fields = expr
                .clone()
                .into_deserialization_iterator()
                .ok_or(Error::ExpectedTypeGot(type_name::<#enum_name>().to_string(), expr.pretty()))?;
            let mut current = fields.next();

            #(#deserializations)*

            Err(Error::WrongNumberOfElements(0, 0))
        })
    )
}

#[proc_macro_derive(Config)]
pub fn config(item: TokenStream) -> TokenStream {
    // Parse the input TokenStream into a DeriveInput syntax tree
    let ast: DeriveInput = syn::parse(item).unwrap();

    // Get the name of the struct
    let name = &ast.ident;

    // Match the data of the input to ensure it's a struct
    match ast.data {
        Data::Struct(s) => {
            let (s, d) =  serialize_struct_fields(name, &s.fields);
            impl_config(name, s, d)
        },
        Data::Enum(e) => {
            let (s, d) =  serialize_enum(name, &e.variants);
            impl_config(name, s, d)
        },
        Data::Union(_) => unimplemented!()
    }
}
