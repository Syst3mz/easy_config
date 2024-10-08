extern crate proc_macro;


use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Data, DataEnum, DataStruct, DeriveInput, Expr, Field, Fields, Index, Lit, Meta, Type, Variant};

fn serialized(item: TokenStream) -> TokenStream {
    quote! {#item.serialize()}
}

fn comment_expr(expr: TokenStream, comment: Option<String>) -> TokenStream {
    if let Some(comment) = comment{
        return quote! {#expr.with_comment(#comment.to_string())};
    }

    expr
}
fn serialize_named(name: &Ident, variable_accessor: TokenStream, comment: Option<String>) -> TokenStream {
    let serialized_accessor = serialized(variable_accessor);
    comment_expr(
        quote! {
            CstExpression::pair(String::from(stringify!(#name)), #serialized_accessor)
        },
        comment
    )
}
fn serialize_unnamed(variable_accessor: TokenStream, comment: Option<String>) -> TokenStream {
    comment_expr(serialized(variable_accessor), comment)
}

fn serialize_self_field(ident: &Option<Ident>, index: Index, comment: Option<String>) -> TokenStream {
    if let Some(ident) = ident {
        serialize_named(ident, quote! {self.#ident}, comment)
    } else {
        serialize_unnamed(quote! {self.#index}, comment)
    }
}

fn extract_comment_if_exists(field: &Field) -> Option<String> {
    for attr in &field.attrs {
        if !attr.path().is_ident("EasyConfig") {
            continue;
        }

        if let Ok(Meta::NameValue(ref name_value)) = attr.parse_args() {
            if let Expr::Lit(l) = &name_value.value {
                if let Lit::Str(text) = &l.lit {
                    return Some(text.value())
                }
            }
        }
    }

    return None;
}

fn serialize_struct(strct: &DataStruct, struct_name: &Ident) -> TokenStream {
    let serializations = strct.fields.iter()
        .enumerate()
        .map(|(index, field)| serialize_self_field(&field.ident, Index::from(index), extract_comment_if_exists(field)));

    match strct.fields {
        Fields::Unit => quote! {
            CstExpression::presence(stringify!(#struct_name).to_string())
        },
        _ => quote! {
            CstExpression::collection(vec![
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
                    quote! {#name},
                    extract_comment_if_exists(field)
                )
            } else {
                serialize_unnamed(quote! {#name}, extract_comment_if_exists(field))
            }
        });

    let variant_name = &variant.ident;


    quote! {
       CstExpression::collection(vec![
            CstExpression::presence(stringify!(#variant_name).to_string()),
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
    quote! {<#typ>::deserialize(#item)}
}

fn read_next_field_expecting(expected: &impl ToTokens) -> TokenStream {
    quote! {
        fields
            .next()
            .ok_or(
                Error::at(
                    ExpectedTypeGot(stringify!(#expected).to_string(), "End of input".to_string()),
                    expr.location
                )
            )?
    }
}

fn add_owning_information(to: TokenStream, owner: TokenStream) -> TokenStream {
    quote! {
        #to.map_err(|x| {
            if let ExpectedTypeGot(ty, at) = x.kind {
                Error::at(ExpectedTypeIn(ty, stringify!(#owner).to_string(), at), x.location)
            } else {
                x
            }
        })
    }
}

fn deserialize_struct_like_field(name: &Ident, to: &Type, owner: TokenStream) -> TokenStream {
    let next_field = read_next_field_expecting(to);
    let to_deserialize = quote! {
        #next_field
        .get(stringify!(#name))
        .ok_or(Error::at(UnableToFindKey(stringify!(#name).to_string()), expr.location))?
    };
    let deserialized = add_owning_information(deserialized(to_deserialize, to), owner);

    quote! {#name: #deserialized?}
}

fn deserialize_tuple_like_field(to: &Type, owner: TokenStream) -> TokenStream {
    let deserialized = add_owning_information(
        deserialized(read_next_field_expecting(to), to),
        owner
    );

    quote! {#deserialized?}
}

fn deserialize_field(name: Option<&Ident>, ty: &Type, owner: TokenStream) -> TokenStream {
    if let Some(name) = name {
        deserialize_struct_like_field(name, ty, owner)
    } else {
        deserialize_tuple_like_field(ty, owner)
    }
}


fn deserialize_enum_variant(variant: &Variant, enum_name: &Ident) -> TokenStream {
    let variant_name = &variant.ident;

    let deserialization = match &variant.fields {
        Fields::Named(n) => {
            let deserializations = n.named.iter()
                .map(|x| deserialize_struct_like_field(x.ident.as_ref().unwrap(), &x.ty, quote! {variant_name}));
            quote! {
                #enum_name::#variant_name{
                    #(#deserializations),*
                }
            }
        },
        Fields::Unnamed(u) => {
            let deserializations = u.unnamed.iter().map(|x| deserialize_tuple_like_field(&x.ty, quote! {variant_name}));
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
        let specifier = match &expr.data {
            CstData::Presence(s) => Some(s.clone()),
            CstData::Pair(_, _) => None,
            CstData::Collection(c) => {
                let specifier = c.get(0).map(|x| x.release().map(|x| x.clone())).flatten();

                if specifier.is_some() {
                    fields.next();
                }

                specifier
            },
        }.ok_or(Error::at(ExpectedTypeGot(stringify!(#enum_name).to_string(), expr.pretty()), expr.location))?;

        match specifier.as_str() {
            #(#variant_serializations)*
            _ => Err(Error::at(ExpectedTypeGot(stringify!(#enum_name).to_string(), expr.pretty()), expr.location))
        }
    }
}

fn deserialize_struct(strct: &DataStruct, struct_name: &Ident) -> TokenStream {
    match &strct.fields {
        Fields::Named(n) => {
            let deserializations = n.named
                .iter()
                .map(|field| {
                    let field_name = field.ident.as_ref().unwrap();
                    let field_type = &field.ty;
                    deserialize_field(field.ident.as_ref(), field_type, quote! {#struct_name.#field_name<#field_type>})
                });
            quote! {
                Ok(#struct_name {
                    #(#deserializations),*
                })
            }
        },
        Fields::Unnamed(u) => {
            let deserializations = u.unnamed
                .iter()
                .map(|field| deserialize_field(field.ident.as_ref(), &field.ty, quote! {struct_name}));
            quote! {
                Ok(#struct_name (
                    #(#deserializations),*
                ))
            }
        },
        Fields::Unit => quote! {
            let maybe_unit = fields.next();
            if let Some(unit) = &maybe_unit {
                return match &unit.data {
                    CstData::Presence(s) => {
                        if s == stringify!(#struct_name) {
                            Ok(#struct_name)
                        } else {
                            Err(Error::at(ExpectedTypeGot(stringify!(#struct_name).to_string(), unit.pretty()), unit.location))
                        }
                    }
                    _ => Err(Error::at(ExpectedTypeGot(stringify!(#struct_name).to_string(), unit.pretty()), expr.location)) ,
                }
            }

            Ok(#struct_name)
        }
    }
}


#[proc_macro_derive(Config, attributes(EasyConfig))]
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
        impl easy_config::serialization::Config for #name {
            fn serialize(&self) -> easy_config::expression::CstExpression {
                use easy_config::expression::{CstExpression, CstData};
                #serialization
            }

            fn deserialize(expr: easy_config::expression::CstExpression) -> Result<Self, easy_config::serialization::error::Error> where Self: Sized {
                use easy_config::expression::{CstExpression, CstData};
                use easy_config::serialization::DeserializeExtension;
                use easy_config::serialization::error::Error;
                use easy_config::serialization::error::Kind::*;

                let mut fields = expr
                    .clone()
                    .into_deserialization_iterator()
                    .ok_or(Error::at(ExpectedTypeGot(stringify!(#name).to_string(), expr.pretty()), expr.location))?;

                #deserialization
            }
        }
    };

    gen.into()
}