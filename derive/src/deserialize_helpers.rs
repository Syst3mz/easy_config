use proc_macro2::Ident;
use quote::quote;
use syn::{Fields, FieldsNamed, FieldsUnnamed, Variant};
use crate::shared::comma_separated_list;

fn deserialize_named_fields(fields_named: &FieldsNamed, err_name: impl AsRef<str>) -> proc_macro2::TokenStream {
    let err_name = err_name.as_ref();
    let fields = fields_named.named.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();
        let name_str = ident.to_string();
        let err_text = format!(
            "Unable to read a {} because the mandatory field '{}' is not present or could not be deserialized.",
            err_name, name_str
        );

        quote! {
            #ident: fields.get(stringify!(#ident), source_text).contextualize(#err_text)?
        }
    });
    comma_separated_list(fields)
}

pub fn deserialize_named_struct(fields_named: &FieldsNamed, struct_name: &Ident) -> proc_macro2::TokenStream {
    let fields = deserialize_named_fields(fields_named, struct_name.to_string());
    let err_text = format!(
        "Unable to read a {} because it is not a list of bindings.",
        struct_name
    );

    quote! {
        let mut fields = normalized.binding_map().contextualize(#err_text)?;
        Ok(Self {
            #fields
        })
    }
}

fn deserialize_unnamed_fields(fields_unnamed: &FieldsUnnamed, err_name: impl AsRef<str>) -> proc_macro2::TokenStream {
    let err_name = err_name.as_ref();
    let field_deserializers = fields_unnamed.unnamed.iter().enumerate().map(|(idx, _)| {
        let err_text = format!(
            "Unable to read field {} of tuple struct '{}'",
            idx, err_name
        );
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
    struct_name: &Ident
) -> proc_macro2::TokenStream {
    let field_deserializers = deserialize_unnamed_fields(fields_unnamed, struct_name.to_string());
    let err_text = format!("Failed to fetch list of fields for a {}", struct_name);
    quote! {
        let exprs = normalized.next_or_err().contextualize(#err_text)?;
        let mut exprs = exprs.into_iter();
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
            let field_deserializers = named.named.iter().map(|field| {
                let ident = field.ident.as_ref().unwrap();
                let name_str = ident.to_string();
                let err_text = format!(
                    "Unable to read field '{}' of enum variant '{}'",
                    name_str, field_name
                );
                quote! {
                    #ident: binding_map
                        .get(#name_str, source_text)
                        .contextualize(#err_text)?
                }
            });
            let field_list = comma_separated_list(field_deserializers);

            quote! {
                #field_name => {
                    use ::easy_config::config_error::Contextualize;
                    let mut binding_map = fields
                        .into_iter()
                        .binding_map()
                        .contextualize(format!(
                            "Unable to read enum variant '{}' because it is not a list of bindings",
                            #field_name
                        ))?;

                    Ok(#enum_name::#variant_ident { #field_list })
                }
            }
        },
        Fields::Unnamed(unnamed) => {
            let field_deserializers = unnamed.unnamed.iter().enumerate().map(|(idx, _)| {
                let err_text = format!(
                    "Unable to read field {} of enum variant '{}'",
                    idx, field_name
                );
                quote! {
                    fields_iter
                        .deserialize_next(source_text)
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
