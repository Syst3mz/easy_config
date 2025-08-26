mod serialize_helpers;
mod deserialize_helpers;
mod shared;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DataEnum, DataStruct, DeriveInput};
use crate::deserialize_helpers::{deserialize_variant_arm, deserialize_unit_struct, deserialize_named_struct, deserialize_unnamed_struct};
use crate::serialize_helpers::{serialize_named_fields, serialize_unnamed_fields, serialize_variant_arm};
use crate::shared::comma_separated_list;

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

    let struct_name_str = struct_name.to_string();

    let deserialize_body = match &data.fields {
        syn::Fields::Named(fields_named) => deserialize_named_struct(fields_named, &struct_name_str),
        syn::Fields::Unnamed(fields_unnamed) => deserialize_unnamed_struct(fields_unnamed, &struct_name_str),
        syn::Fields::Unit => deserialize_unit_struct(struct_name)
    };

    quote! {
        use easy_config::serialization::EasyConfig;
        impl EasyConfig for #struct_name {
            fn serialize(&self) -> ::easy_config::expression::Expression {
                let mut body = #serialize_body;
                body.prepend_into_list(::easy_config::expression::Expression::presence(#struct_name_str));
                body
            }

            fn deserialize(exprs: &mut ::easy_config::expression_iterator::ExpressionIterator, source_text: impl AsRef<str>) -> Result<Self, ::easy_config::serialization::serialization_error::SerializationError> {
                let source_text = source_text.as_ref();
                exprs.eat_presence_if_present_and_matching(#struct_name_str);
                #deserialize_body
            }
        }
    }.into()
}

fn generate_config_for_enum(input: &DeriveInput, data: &DataEnum) -> TokenStream {
    let enum_name = &input.ident;
    let enum_name_str = enum_name.to_string();

    let options = comma_separated_list(data.variants.iter().map(|x| {
        let x = x.ident.to_string();
        quote! { #x }
    }));

    let serialize_arms = data.variants.iter().map(|variant| {
        serialize_variant_arm(enum_name, variant)
    });

    let deserialize_arms = data.variants.iter().map(|variant| {
        deserialize_variant_arm(enum_name, variant)
    });

    // Fixed error message to use actual enum name
    let enum_error_msg = format!(
        "Unable to deserialize enum {} since we can't extract a discriminant and an argument list",
        enum_name_str
    );

    quote! {
        use easy_config::serialization::EasyConfig;
        impl EasyConfig for #enum_name {
            fn serialize(&self) -> ::easy_config::expression::Expression {
                match self {
                    #(#serialize_arms,)*
                }
            }

            fn deserialize(exprs: &mut ::easy_config::expression_iterator::ExpressionIterator, source_text: impl AsRef<str>) -> Result<Self, ::easy_config::serialization::serialization_error::SerializationError> {
                use ::easy_config::config_error::Contextualize;
                let source_text = source_text.as_ref();
                let (discriminant, mut fields) = exprs
                    .extract_enum(source_text)
                    .contextualize(#enum_error_msg)?;
                let span = exprs.span().unwrap();
                const OPTIONS: &'static [&'static str] = &[#options];
                match discriminant.as_str() {
                    #(#deserialize_arms,)*
                    _ => Err(
                        ::easy_config::serialization::serialization_error::SerializationError::on_span(
                            ::easy_config::serialization::serialization_error::Kind::ExpectedDiscriminant(discriminant, OPTIONS),
                            span,
                            source_text
                        )
                    ),
                }
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
                "#[derive(EasyConfig)] is not supported for unions",
            )
                .to_compile_error()
                .into()
        },
    }
}