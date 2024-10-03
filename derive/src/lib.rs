extern crate proc_macro;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Data, DataEnum, DataStruct, DeriveInput, Field, Fields, Index, Variant};

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

fn serialize_struct_field(ident: &Option<Ident>, index: Index) -> TokenStream {
    if let Some(ident) = ident {
        serialize_named(ident, quote! {self.#ident})
    } else {
        serialize_unnamed(quote! {self.#index})
    }
}

fn serialize_struct(strct: &DataStruct) -> TokenStream {
    let serializations = strct.fields.iter()
        .enumerate()
        .map(|(index, field)| serialize_struct_field(&field.ident, Index::from(index)));

    quote! {
        Expression::Collection(vec![
                    #(#serializations),*
        ])
        .minimized()
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
        easy_config::parser::expression::Expression::Collection(vec![
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

#[proc_macro_derive(Config)]
pub fn config(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input TokenStream into a DeriveInput syntax tree
    let ast: DeriveInput = syn::parse(item).unwrap();

    // Get the name of the struct
    let name = &ast.ident;

    // Match the data of the input to ensure it's a struct
    let serialization = match &ast.data {
        Data::Struct(s) => serialize_struct(s),
        Data::Enum(e) => serialize_enum(e),
        _ => unimplemented!()
    };

    let gen = quote! {
        impl Config for #name {
            fn serialize(&self) -> easy_config::parser::expression::Expression {
                use easy_config::parser::expression::Expression;
                #serialization
            }

            fn deserialize(expr: easy_config::parser::expression::Expression) -> Result<Self, easy_config::serialization::error::Error> where Self: Sized {
                unimplemented!()
            }
        }
    };

    gen.into()
}
