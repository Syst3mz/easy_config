extern crate proc_macro;


use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Data, DataEnum, DataStruct, DeriveInput, Field, Fields, Index, Type, Variant};

fn serialized(item: TokenStream) -> TokenStream {
    quote! {#item.serialize()}
}
fn serialize_named(name: &Ident, variable_accessor: TokenStream) -> TokenStream {
    let serialized_accessor = serialized(variable_accessor);
    quote! {
        Expression::Pair(String::from(stringify!(#name)), Box::new(#serialized_accessor))
    }
}
fn serialize_unnamed(variable_accessor: TokenStream) -> TokenStream {
    serialized(variable_accessor)
}

fn serialize_self_field(ident: &Option<Ident>, index: Index) -> TokenStream {
    if let Some(ident) = ident {
        serialize_named(ident, quote! {self.#ident})
    } else {
        serialize_unnamed(quote! {self.#index})
    }
}

fn serialize_struct(strct: &DataStruct, struct_name: &Ident) -> TokenStream {
    let serializations = strct.fields.iter()
        .enumerate()
        .map(|(index, field)| serialize_self_field(&field.ident, Index::from(index)));

    match strct.fields {
        Fields::Unit => quote! {
            Expression::Presence(stringify!(#struct_name).to_string())
        },
        _ => quote! {
            Expression::Collection(vec![
                #(#serializations),*
            ])
            .minimized()
        },
    }
}

fn name_or_match_var(field: &Field, index: usize) -> Ident {
    field.ident.clone().unwrap_or(variant_match_var(index))
}

fn serialize_variant(variant: &Variant) -> TokenStream {
    let serializations = variant.fields.iter()
        .enumerate()
        .map(|(index, field)|{

            // name is the name that will be used to FIND the variable, if it exists on the field
            // it will be used in the serialized form, if it does not it will be skipped.
            let name = name_or_match_var(field, index);

            if field.ident.is_some() {
                serialize_named(
                    &name,
                    quote! {#name}
                )
            } else {
                serialize_unnamed(quote! {#name})
            }
        });

    let variant_name = &variant.ident;


    quote! {
        core::expression::Expression::Collection(vec![
            Expression::Presence(stringify!(#variant_name).to_string()),
            #(#serializations),*
        ])
        .minimized()
    }
}


fn variant_match_var(index: usize) -> Ident {
    const LETTERS: [&str; 26] = [
        "a",
        "b",
        "c",
        "d",
        "e",
        "f",
        "g",
        "h",
        "i",
        "j",
        "k",
        "l",
        "m",
        "n",
        "o",
        "p",
        "q",
        "r",
        "s",
        "t",
        "u",
        "v",
        "w",
        "x",
        "y",
        "z",
    ];
    Ident::new(LETTERS[index], Span::call_site())
}
fn serialize_variant_match(variant: &Variant) -> TokenStream {
    let variant_name = &variant.ident;
    let variant_clause = variant.fields
        .iter()
        .enumerate()
        .map(|(index, field)| name_or_match_var(field, index));
    
    match variant.fields {
        Fields::Named(_) => quote! { Self::#variant_name{#(#variant_clause),*} },
        Fields::Unnamed(_) => quote! { Self::#variant_name(#(#variant_clause),*) },
        Fields::Unit => quote! { Self::#variant_name },
    }
}

fn serialize_enum(enm: &DataEnum) -> TokenStream {
    let variant_value_serializations = enm.variants.iter().map(|x| serialize_variant(x));
    let variant_matcher = enm.variants.iter().map(|x| serialize_variant_match(x));
    let arms = variant_matcher
        .zip(variant_value_serializations)
        .map(|(matcher, serialization)| quote! {#matcher => #serialization});

    quote! {
        match self {
            #(#arms),*
        }
    }
}


fn deserialized(item: TokenStream, typ: &Type) -> TokenStream {
    quote! {<#typ>::deserialize(#item)?}
}

fn read_next_field_expecting(expected: &impl ToTokens) -> TokenStream {
    quote! {
        fields
            .next()
            .ok_or(Error::ExpectedTypeGot(
                stringify!(#expected).to_string(), "End of input".to_string())
            )?
    }
}

fn deserialize_struct_like_field(name: &Ident, to: &Type) -> TokenStream {
    let next_field = read_next_field_expecting(to);
    let to_deserialize = quote! {
        #next_field
        .get(stringify!(#name))
        .ok_or(Error::UnableToFindKey(stringify!(#name).to_string()))?
    };
    let deserialized = deserialized(to_deserialize, to);
    quote! {#name: #deserialized}
}

fn deserialize_tuple_like_field(to: &Type) -> TokenStream {
    deserialized(read_next_field_expecting(to), to)
}

fn deserialize_field(name: Option<&Ident>, ty: &Type) -> TokenStream {
    if let Some(name) = name {
        deserialize_struct_like_field(name, ty)
    } else {
        deserialize_tuple_like_field(ty)
    }
}


fn deserialize_enum_variant(variant: &Variant, enum_name: &Ident) -> TokenStream {
    let variant_name = &variant.ident;

    let deserialization = match &variant.fields {
        Fields::Named(n) => {
            let deserializations = n.named.iter().map(|x| deserialize_struct_like_field(x.ident.as_ref().unwrap(), &x.ty));
            quote! {
                #enum_name::#variant_name{
                    #(#deserializations),*
                }
            }
        },
        Fields::Unnamed(u) => {
            let deserializations = u.unnamed.iter().map(|x| deserialize_tuple_like_field(&x.ty));
            quote! {
                #enum_name::#variant_name(
                    #(#deserializations),*
                )

            }
        },
        Fields::Unit => quote! {#enum_name::#variant_name},
    };

    quote! {
        stringify!(#variant_name) => Ok(#deserialization),
    }
}

fn deserialize_enum(enm: &DataEnum, enum_name: &Ident) -> TokenStream {

    let variant_serializations = enm.variants
        .iter()
        .map(|variant| deserialize_enum_variant(variant, enum_name));

    quote! {
        let specifier = match &expr {
            Expression::Presence(s) => Some(s.clone()),
            Expression::Pair(_, _) => None,
            Expression::Collection(c) => {
                let specifier = c.get(0).map(|x| x.release().map(|x| x.clone())).flatten();

                if specifier.is_some() {
                    fields.next();
                }

                specifier
            },
        }.ok_or(Error::ExpectedTypeGot(stringify!(#enum_name).to_string(), expr.pretty()))?;

        match specifier.as_str() {
            #(#variant_serializations)*
            _ => Err(Error::ExpectedTypeGot(stringify!(#enum_name).to_string(), expr.pretty()))
        }
    }
}

fn deserialize_struct(strct: &DataStruct, struct_name: &Ident) -> TokenStream {
    match &strct.fields {
        Fields::Named(n) => {
            let deserializations = n.named
                .iter()
                .map(|field| deserialize_field(field.ident.as_ref(), &field.ty));
            quote! {
                Ok(#struct_name {
                    #(#deserializations),*
                })
            }
        },
        Fields::Unnamed(u) => {
            let deserializations = u.unnamed
                .iter()
                .map(|field| deserialize_field(field.ident.as_ref(), &field.ty));
            quote! {
                Ok(#struct_name (
                    #(#deserializations),*
                ))
            }
        },
        Fields::Unit => quote! {
            let maybe_unit = fields.next();
            if let Some(unit) = &maybe_unit {
                return match unit {
                    Expression::Presence(s) => {
                        if s == stringify!(#struct_name) {
                            Ok(#struct_name)
                        } else {
                            Err(Error::ExpectedTypeGot(stringify!(#struct_name).to_string(), maybe_unit.unwrap().pretty()))
                        }
                    }
                    _ => Err(Error::ExpectedTypeGot(stringify!(#struct_name).to_string(), maybe_unit.unwrap().pretty())) ,
                }
            }

            Ok(#struct_name)
        }
    }
}


#[proc_macro_derive(Config)]
pub fn config(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input TokenStream into a DeriveInput syntax tree
    let ast: DeriveInput = syn::parse(item).unwrap();

    // Get the name of the struct
    let name = &ast.ident;


    let serialization = match &ast.data {
        Data::Struct(s) => serialize_struct(s, name),
        Data::Enum(e) => serialize_enum(e),
        _ => unimplemented!()
    };

    let deserialization = match &ast.data {
        Data::Struct(s) => deserialize_struct(s, name),
        Data::Enum(e) => deserialize_enum(e, name),
        _ => unimplemented!(),
    };

    let gen = quote! {
        use core::serialization::Config;
        impl core::serialization::Config for #name {
            fn serialize(&self) -> core::expression::Expression {
                use core::expression::Expression;
                #serialization
            }

            fn deserialize(expr: core::expression::Expression) -> Result<Self, core::serialization::error::Error> where Self: Sized {
                use core::expression::Expression;
                use core::serialization::DeserializeExtension;
                use core::serialization::error::Error;

                let mut fields = expr
                    .clone()
                    .into_deserialization_iterator()
                    .ok_or(Error::ExpectedTypeGot(stringify!(#name).to_string(), expr.pretty()))?;

                #deserialization
            }
        }
    };

    gen.into()
}