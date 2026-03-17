use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DataStruct, DeriveInput, Fields, FieldsNamed, FieldsUnnamed};

type DeriveResult = Result<proc_macro2::TokenStream, syn::Error>;

fn has_schema() -> proc_macro2::TokenStream {
    quote! { easy_config::schema::HasSchema }
}
fn schema_type() -> proc_macro2::TokenStream {
    quote! {easy_config::schema::Schema }
}

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
    let schema_type = schema_type();

    let fields = fields_named.named.iter().map(|field| {
            let name = field.ident.as_ref().unwrap().to_string();
            let ty = &field.ty;
            quote!((#name, <#ty as #has_schema>::SCHEMA))
    });

    let name_string = name_ident.to_string();

    Ok(wrap_with_schema(name_ident, quote!({
        static FIELDS: &[(&str, #schema_type)] = &[#(#fields),*];
        #schema_type::Struct(#name_string, FIELDS)
    })))
}

fn derive_unnamed_struct(name_ident: &proc_macro2::Ident, fields_unnamed: &FieldsUnnamed) -> DeriveResult {
    let has_schema = has_schema();
    let schema_type = schema_type();

    let fields = fields_unnamed.unnamed.iter().map(|field| {
        let ty = &field.ty;
        quote! {<#ty as #has_schema>::SCHEMA}
    });

    let name_string = name_ident.to_string();
    Ok(wrap_with_schema(name_ident, quote!({
        static FIELDS: &[#schema_type] = &[#(#fields),*];
        #schema_type::TupleStruct(#name_string, FIELDS)
    })))
}

fn derive_unit_struct(name_ident: &proc_macro2::Ident) -> DeriveResult {
    let schema_type = schema_type();
    let name_string = name_ident.to_string();
    Ok(wrap_with_schema(name_ident, quote!(#schema_type::UnitStruct(#name_string))))
}

fn derive_struct(input: &DeriveInput, struct_definition: &DataStruct) -> DeriveResult {

    match &struct_definition.fields {
        Fields::Named(n) => derive_named_struct(&input.ident, n),
        Fields::Unnamed(u) => derive_unnamed_struct(&input.ident, u),
        Fields::Unit => derive_unit_struct(&input.ident),
    }
}

fn derive_easy_config_inner(input: DeriveInput) -> DeriveResult {
    match &input.data {
        syn::Data::Struct(data) => derive_struct(&input, data),
        syn::Data::Enum(_) => todo!(),  // derive_enum(&input, data),
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