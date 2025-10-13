use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::{Field, Fields, FieldsNamed, FieldsUnnamed, Variant};
use crate::shared::comma_separated_list;

fn normal_form(name: &Ident, entries: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    quote! {
        ::easy_config::expression::Expression::list(vec![
            ::easy_config::expression::Expression::presence(stringify!(#name)),
            #entries
        ])
    }
}

fn extract_comment(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("comment") {
            if let Ok(lit) = attr.parse_args::<syn::LitStr>() {
                return Some(lit.value());
            }
        }
    }
    None
}

pub fn append_comment(uncommented: proc_macro2::TokenStream, field: &Field) -> proc_macro2::TokenStream {
    if let Some(comment) = extract_comment(&field.attrs) {
        quote! { #uncommented.with_comment(#comment) }
    } else {
        uncommented
    }
}

pub fn serialize_into_list<I: IntoIterator<Item = impl ToTokens>>(entries: I) -> proc_macro2::TokenStream {
    let entries = comma_separated_list(entries);
    quote! {
        ::easy_config::expression::Expression::list(vec![ #entries ])
    }
}

fn destructure_unnamed(fields_unnamed: &FieldsUnnamed) -> Vec<Ident> {
    (0..fields_unnamed.unnamed.len())
        .map(|i| syn::Ident::new(&format!("f{}", i), proc_macro2::Span::call_site()))
        .collect()
}

pub fn serialize_named_field(accessor: impl ToTokens, field: &Field) -> proc_macro2::TokenStream {
    let field_ident = field.ident.as_ref().unwrap();

    let uncommented = quote! {
        ::easy_config::expression::Expression::binding(
            stringify!(#field_ident),
            #accessor #field_ident.serialize()
        )
    };
    append_comment(uncommented, &field)
}
pub fn serialize_named_fields(prefix: impl ToTokens, structure_name: &Ident, fields_named: &FieldsNamed) -> proc_macro2::TokenStream {
    let prefix = prefix.into_token_stream();

    let entries = fields_named.named.iter().map(|field| {
        serialize_named_field(prefix.clone(), &field)
    });

    normal_form(structure_name, serialize_into_list(entries))
}

pub fn serialize_unnamed_fields(prefix: impl ToTokens, fields_unnamed: &FieldsUnnamed, struct_name: &Ident) -> proc_macro2::TokenStream {
    let entries = fields_unnamed.unnamed.iter().enumerate().map(|(index, field)| {
        let index = syn::Index::from(index);
        append_comment(quote! {
            #prefix #index.serialize()
        }, field)
    });

    let entries = serialize_into_list(entries);
    normal_form(struct_name, entries)
}


fn serialize_variant_with_no_fields(variant_name: &Ident) -> proc_macro2::TokenStream {

    normal_form(variant_name, quote! {
        ::easy_config::expression::Expression::list(vec![])
    })
}

fn prepend_arm(enum_name: &Ident, variant: &Variant, to: impl ToTokens) -> proc_macro2::TokenStream {
    let variant_name = &variant.ident;

    let receiver = match &variant.fields {
        Fields::Named(fields_named) => {
            let bindings: Vec<_> = fields_named.named.iter()
                .map(|f| f.ident.as_ref().unwrap())
                .collect();
            quote! { { #( #bindings ),* } }
        }
        Fields::Unnamed(fields_unnamed) => {
            let bindings = destructure_unnamed(fields_unnamed);
            quote! { ( #( #bindings ),* ) }
        }
        Fields::Unit => quote! {},
    };

    quote! {
        #enum_name::#variant_name #receiver => #to
    }
}

fn serialize_unnamed_variant(fields_unnamed: &FieldsUnnamed, variant_name: &Ident) -> proc_macro2::TokenStream {
    // Generate direct serialization calls for each field variable (f0, f1, f2, etc.)
    let entries = (0..fields_unnamed.unnamed.len()).map(|index| {
        let field_var = syn::Ident::new(&format!("f{}", index), proc_macro2::Span::call_site());
        let field = &fields_unnamed.unnamed[index];
        append_comment(quote! {
            #field_var.serialize()
        }, field)
    });

    normal_form(variant_name, serialize_into_list(entries))
}
pub fn serialize_variant_arm(enum_name: &Ident, variant: &Variant) -> proc_macro2::TokenStream {
    let fields = match &variant.fields {
        Fields::Named(named) => serialize_named_fields(quote! {}, &variant.ident, named),
        Fields::Unnamed(unnamed) => serialize_unnamed_variant(unnamed, &variant.ident),
        Fields::Unit => serialize_variant_with_no_fields(&variant.ident)
    };

    prepend_arm(enum_name, variant, fields)
}