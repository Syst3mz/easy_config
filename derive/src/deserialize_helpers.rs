use proc_macro2::Ident;
use quote::quote;
use syn::{Fields, FieldsNamed, FieldsUnnamed, Variant};
use crate::shared::comma_separated_list;

fn deserialize_named_fields(fields_named: &FieldsNamed, err_name: impl AsRef<str>) -> proc_macro2::TokenStream {
    let err_name = err_name.as_ref();
    let fields = fields_named.named.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();
        let name_str = ident.to_string();
        let err_text = format!("Unable to read a {} because the mandatory field {} is not present or could not be deserialized.", err_name, name_str);
        quote! {
            #ident: ::easy_config::serialization::deserialize_field_from_map_or_error(#name_str, &mut mapping, error_span, source_text)
                .contextualize(#err_text)?
        }
    });
    comma_separated_list(fields)
}

pub fn deserialize_named_struct(fields_named: &FieldsNamed, struct_name: impl AsRef<str>) -> proc_macro2::TokenStream {
    let struct_name = struct_name.as_ref();

    let fields = deserialize_named_fields(fields_named, struct_name);
    let err_text = format!("Unable to read a {} because it is not a list of bindings.", struct_name);
    quote! {
        use ::easy_config::config_error::Contextualize;
        let (mut mapping, error_span) = exprs
            .convert_binding_list_to_hashmap_of_values(source_text)
            .contextualize(#err_text)?;
        Ok(Self {
            #fields
        })
    }
}

fn deserialize_unnamed_fields(fields_unnamed: &FieldsUnnamed, err_name: impl AsRef<str>) -> proc_macro2::TokenStream {
    let err_name = err_name.as_ref();
    // Iterate over each field by index
    let field_deserializers = fields_unnamed.unnamed.iter().enumerate().map(|(idx, _)| {
        let err_text = format!("Unable to read field {} of tuple struct {}", idx, err_name);

        quote! {
            exprs
                .deserialize_next(source_text)
                .contextualize(#err_text)?
        }
    });

    comma_separated_list(field_deserializers)
}

pub fn deserialize_unnamed_struct(
    fields_unnamed: &FieldsUnnamed,
    struct_name: impl AsRef<str>
) -> proc_macro2::TokenStream {
    let field_deserializers = deserialize_unnamed_fields(fields_unnamed, struct_name);

    quote! {
        use ::easy_config::config_error::Contextualize;

        Ok(Self(
            #field_deserializers
        ))
    }
}

pub fn deserialize_unit_struct(struct_name: &Ident) -> proc_macro2::TokenStream {
    quote! { Ok(#struct_name {}) }
}

pub fn deserialize_variant_arm(enum_name: &Ident, variant: &Variant) -> proc_macro2::TokenStream {
    let field_name = variant.ident.to_string();
    let variant_ident = &variant.ident;

    match &variant.fields {
        Fields::Named(named) => {
            // For named fields in enum variants, we need to deserialize from the fields iterator
            let field_deserializers = named.named.iter().map(|field| {
                let ident = field.ident.as_ref().unwrap();
                let name_str = ident.to_string();
                let err_text = format!("Unable to read field '{}' of enum variant '{}'", name_str, field_name);
                quote! {
                    #ident: ::easy_config::serialization::deserialize_field_from_map_or_error(#name_str, &mut mapping, error_span, source_text)
                        .contextualize(#err_text)?
                }
            });
            let field_list = comma_separated_list(field_deserializers);

            quote! {
                #field_name => {
                    use ::easy_config::config_error::Contextualize;
                    let mut fields_iter = fields.into_iter();
                    let (mut mapping, error_span) = fields_iter
                        .convert_binding_list_to_hashmap_of_values(source_text)
                        .contextualize(&format!("Unable to read enum variant '{}' because it is not a list of bindings", #field_name))?;
                    Ok(#enum_name::#variant_ident { #field_list })
                }
            }
        },
        Fields::Unnamed(unnamed) => {
            // For unnamed fields, deserialize sequentially from the fields iterator
            let field_deserializers = unnamed.unnamed.iter().enumerate().map(|(idx, _)| {
                let err_text = format!("Unable to read field {} of enum variant '{}'", idx, field_name);
                quote! {
                    fields_iter.deserialize_next(source_text)
                        .contextualize(#err_text)?
                }
            });
            let field_list = comma_separated_list(field_deserializers);

            quote! {
                #field_name => {
                    use ::easy_config::config_error::Contextualize;
                    let mut fields_iter = fields.into_iter();
                    Ok(#enum_name::#variant_ident(#field_list))
                }
            }
        },
        Fields::Unit => {
            quote! {
                #field_name => Ok(#enum_name::#variant_ident)
            }
        },
    }
}