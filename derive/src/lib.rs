use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DataEnum, DataStruct, DeriveInput, Fields, FieldsNamed, FieldsUnnamed};
use crate::consts::*;

mod consts;

type DeriveResult = Result<proc_macro2::TokenStream, syn::Error>;

fn wrap_with_schema(name_ident: &proc_macro2::Ident, tokens: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let has_schema = has_schema();
    let schema_type = schema_type();
    quote! {
        impl #has_schema for #name_ident {
            const SCHEMA: #schema_type = #tokens;
        }
    }
}

fn derive_named_struct(name_ident: &proc_macro2::Ident, fields_named: &FieldsNamed) -> DeriveResult {
    let has_schema = has_schema();
    let named_field_type = named_field_type();
    let fields_type = fields_type();
    let struct_schema_type = struct_schema_type();
    let schema_type = schema_type();

    let fields = fields_named.named.iter().map(|field| {
        let name = field.ident.as_ref().unwrap().to_string();
        let ty = &field.ty;
        quote!(#named_field_type { name: #name, schema: <#ty as #has_schema>::SCHEMA })
    });

    let name_string = name_ident.to_string();

    Ok(wrap_with_schema(name_ident, quote!({
        static FIELDS: &[#named_field_type] = &[#(#fields),*];
        static STRUCT_SCHEMA: #struct_schema_type = #struct_schema_type {
            name: #name_string,
            fields: #fields_type::Named(FIELDS),
        };
        #schema_type::Struct(&STRUCT_SCHEMA)
    })))
}
fn derive_unnamed_struct(name_ident: &proc_macro2::Ident, fields_unnamed: &FieldsUnnamed) -> DeriveResult {
    let has_schema = has_schema();
    let fields_type = fields_type();
    let struct_schema_type = struct_schema_type();
    let schema_type = schema_type();

    let name_string = name_ident.to_string();

    if fields_unnamed.unnamed.len() == 1 {
        let ty = &fields_unnamed.unnamed[0].ty;
        Ok(wrap_with_schema(name_ident, quote!({
            static STRUCT_SCHEMA: #struct_schema_type = #struct_schema_type {
                name: #name_string,
                fields: #fields_type::NewType(&<#ty as #has_schema>::SCHEMA),
            };
            #schema_type::Struct(&STRUCT_SCHEMA)
        })))
    } else {
        let fields = fields_unnamed.unnamed.iter().map(|field| {
            let ty = &field.ty;
            quote!(<#ty as #has_schema>::SCHEMA)
        });
        Ok(wrap_with_schema(name_ident, quote!({
            static FIELDS: &[#schema_type] = &[#(#fields),*];
            static STRUCT_SCHEMA: #struct_schema_type = #struct_schema_type {
                name: #name_string,
                fields: #fields_type::Tuple(FIELDS),
            };
            #schema_type::Struct(&STRUCT_SCHEMA)
        })))
    }
}
fn derive_unit_struct(name_ident: &proc_macro2::Ident) -> DeriveResult {
    let fields_type = fields_type();
    let struct_schema_type = struct_schema_type();
    let schema_type = schema_type();
    let name_string = name_ident.to_string();
    Ok(wrap_with_schema(name_ident, quote!({
        static STRUCT_SCHEMA: #struct_schema_type = #struct_schema_type {
            name: #name_string,
            fields: #fields_type::Unit,
        };
        #schema_type::Struct(&STRUCT_SCHEMA)
    })))
}
fn derive_struct(input: &DeriveInput, struct_definition: &DataStruct) -> DeriveResult {
    match &struct_definition.fields {
        Fields::Named(n) => derive_named_struct(&input.ident, n),
        Fields::Unnamed(u) => derive_unnamed_struct(&input.ident, u),
        Fields::Unit => derive_unit_struct(&input.ident),
    }
}

fn derive_unit_variant(variant_name: impl AsRef<str>) -> proc_macro2::TokenStream {
    let variant_name = variant_name.as_ref();
    let variant_schema_type = variant_schema_type();
    let fields_type = fields_type();

    quote! (#variant_schema_type {
        name: #variant_name,
        fields: #fields_type::Unit,
    })
}
fn derive_unnamed_variant(variant_name: impl AsRef<str>, fields_unnamed: &FieldsUnnamed) -> proc_macro2::TokenStream {
    let variant_name = variant_name.as_ref();
    let variant_type = variant_schema_type();
    let fields_type = fields_type();
    let has_schema = has_schema();

    if fields_unnamed.unnamed.len() == 1 {
        let ty = &fields_unnamed.unnamed[0].ty;
        return quote! {
            #variant_type {
                name: #variant_name,
                fields: #fields_type::NewType(&<#ty as #has_schema>::SCHEMA),
            }
        }
    }

    let fields = fields_unnamed.unnamed.iter().map(|f| {
        let ty = &f.ty;
        quote! { <#ty as #has_schema>::SCHEMA }
    });
    quote! {
            #variant_type {
                name: #variant_name,
                fields: #fields_type::Tuple(&[#(#fields),*]),
            }
        }
}

fn derive_named_variant(variant_name: impl AsRef<str>, fields_named: &FieldsNamed) -> proc_macro2::TokenStream {
    let variant_name = variant_name.as_ref();
    let variant_type = variant_schema_type();
    let fields_type = fields_type();
    let has_schema = has_schema();
    let named_field_type = named_field_type();

    let fields = fields_named.named.iter().map(|f| {
        let name = f.ident.as_ref().unwrap().to_string();
        let ty = &f.ty;
        quote! {
            #named_field_type { name: #name, schema: <#ty as #has_schema>::SCHEMA }
        }
    });

    quote! {
        #variant_type {
            name: #variant_name,
            fields: #fields_type::Named(&[#(#fields),*]),
        }
    }
}

fn derive_enum(input: &DeriveInput, enum_definition: &DataEnum) -> DeriveResult {
    let name_ident = &input.ident;
    let name_string = name_ident.to_string();
    let schema_type = schema_type();
    let enum_schema_type = enum_schema_type();

    let variants = enum_definition.variants.iter().map(|variant| {
        let variant_name = variant.ident.to_string();
        match &variant.fields {
            Fields::Unit => derive_unit_variant(&variant_name),
            Fields::Unnamed(fields) => derive_unnamed_variant(&variant_name, fields),
            Fields::Named(fields) => derive_named_variant(&variant_name, fields),
        }
    });

    Ok(wrap_with_schema(name_ident, quote!({
        static VARIANTS: &[easy_config::schema::NamedBagOfFields] = &[#(#variants),*];
        static ENUM_SCHEMA: #enum_schema_type = #enum_schema_type {
            name: #name_string,
            variants: VARIANTS,
        };
        #schema_type::Enum(&ENUM_SCHEMA)
    })))
}

fn derive_easy_config_inner(input: DeriveInput) -> DeriveResult {
    match &input.data {
        syn::Data::Struct(data) => derive_struct(&input, data),
        syn::Data::Enum(data) => derive_enum(&input, data),
        syn::Data::Union(_) => Err(syn::Error::new_spanned(
            &input.ident,
            "EasyConfig cannot be derived for unions",
        )),
    }
}

#[proc_macro_derive(EasyConfig, attributes(easy_config))]
pub fn derive_easy_config(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive_easy_config_inner(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[cfg(test)]
mod runner;