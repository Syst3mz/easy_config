use quote::quote;

pub(crate) fn has_schema() -> proc_macro2::TokenStream {
    quote! { easy_config::schema::HasSchema }
}
pub(crate) fn schema_type() -> proc_macro2::TokenStream {
    quote! { easy_config::schema::Schema }
}
pub(crate) fn named_field_type() -> proc_macro2::TokenStream {
    quote! { easy_config::schema::NamedField }
}
pub(crate) fn fields_type() -> proc_macro2::TokenStream {
    quote! { easy_config::schema::Fields }
}
pub(crate) fn struct_schema_type() -> proc_macro2::TokenStream {
    quote! { easy_config::schema::StructSchema }
}
pub(crate) fn variant_schema_type() -> proc_macro2::TokenStream {
    quote! { easy_config::schema::Variant }
}

pub(crate) fn enum_schema_type() -> proc_macro2::TokenStream {
    quote! { easy_config::schema::EnumSchema }
}