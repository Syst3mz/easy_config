mod helpers;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DataEnum, DataStruct, DeriveInput};
use crate::helpers::{serialize_named_fields, serialize_unnamed_fields, serialize_variant_arm};

fn serialize_unit_field() -> proc_macro2::TokenStream {
    quote! { Expression::list(vec![]) }
}

fn generate_config_for_struct(input: &DeriveInput, data: &DataStruct) -> TokenStream {
    let struct_name = &input.ident;

    let _self = quote! { self. };

    let serialize_body = match &data.fields {
        syn::Fields::Named(fields_named) => serialize_named_fields(_self, fields_named),
        syn::Fields::Unnamed(fields_unnamed) => serialize_unnamed_fields(_self, fields_unnamed),
        syn::Fields::Unit => serialize_unit_field(),
    };

    let name_as_string = struct_name.to_string();

    quote! {
        use easy_config::serialization::EasyConfig;
        impl EasyConfig for #struct_name {
            fn serialize(&self) -> ::easy_config::expression::Expression {
                let mut body = #serialize_body;
                body.prepend_into_list(::easy_config::expression::Expression::presence(#name_as_string));
                body
            }

            fn deserialize(exprs: &mut ::easy_config::expression_iterator::ExpressionIterator, source_text: impl AsRef<str>) -> Result<Self, ::easy_config::serialization::serialization_error::SerializationError> {
                todo!("deserialize for struct {}", stringify!(#struct_name));
            }
        }
    }.into()
}

fn generate_config_for_enum(input: &DeriveInput, data: &DataEnum) -> TokenStream {
    let enum_name = &input.ident;
    let arms = data.variants.iter().map(|variant| {
        serialize_variant_arm(&enum_name, &variant)
    });
    quote! {
        use easy_config::serialization::EasyConfig;
        impl EasyConfig for #enum_name {
            fn serialize(&self) -> ::easy_config::expression::Expression {
                match self {
                    #(#arms),*
                }
            }

            fn deserialize(exprs: &mut ::easy_config::expression_iterator::ExpressionIterator, source_text: impl AsRef<str>) -> Result<Self, ::easy_config::serialization::serialization_error::SerializationError> {
                todo!("deserialize for enum {}", stringify!(#enum_name));
            }
        }
    }.into()
}

#[proc_macro_derive(EasyConfig, attributes(comment))]
pub fn derive_config(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match &input.data {
        syn::Data::Struct(data_struct) => generate_config_for_struct(&input, data_struct),
        syn::Data::Enum(data_enum) => generate_config_for_enum(&input, data_enum),
        syn::Data::Union(_) => {
            syn::Error::new_spanned(
                input,
                "#[derive(Config)] is not supported for unions",
            )
                .to_compile_error()
                .into()
        },
    }
}