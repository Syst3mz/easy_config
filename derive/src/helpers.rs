use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::{Field, Fields, FieldsNamed, FieldsUnnamed, Variant};

pub fn serialize_named_field(accessor: impl ToTokens, field: &Field) -> proc_macro2::TokenStream {
    let field_ident = field.ident.as_ref().unwrap();
    let field_name = string_from(field_ident.to_string());

    let uncommented = quote! {
        ::easy_config::expression::Expression::binding(
            #field_name,
            #accessor #field_ident.serialize()
        )
    };
    append_comment(uncommented, &field)
}
pub fn serialize_named_fields(prefix: impl ToTokens, fields_named: &FieldsNamed) -> proc_macro2::TokenStream {
    let prefix = prefix.into_token_stream();

    let entries = fields_named.named.iter().map(|field| {
        serialize_named_field(prefix.clone(), &field)
    });

    serialize_into_list(entries)
}

pub fn serialize_unnamed_fields(prefix: impl ToTokens, fields_unnamed: &FieldsUnnamed) -> proc_macro2::TokenStream {
    let entries = fields_unnamed.unnamed.iter().enumerate().map(|(index, field)| {
        let index = syn::Index::from(index);
        append_comment(quote! {
                    #prefix #index.serialize()
        }, field)
    });

    serialize_into_list(entries)
}
pub fn extract_comment(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("comment") {
            if let Ok(lit) = attr.parse_args::<syn::LitStr>() {
                return Some(lit.value());
            }
        }
    }
    None
}

pub fn string_from(string: impl AsRef<str>) -> proc_macro2::TokenStream {
    let string = string.as_ref();
    quote! { String::from(#string) }
}
pub fn append_comment(uncommented: proc_macro2::TokenStream, field: &Field) -> proc_macro2::TokenStream {
    if let Some(comment) = extract_comment(&field.attrs) {
        quote! { #uncommented.with_comment(#comment) }
    } else {
        uncommented
    }
}

pub fn serialize_into_list(entries: impl IntoIterator<Item=impl ToTokens>) -> proc_macro2::TokenStream {
    let entries = entries.into_iter();
    quote! {
        ::easy_config::expression::Expression::list(vec![
            #(#entries),*
        ])
    }
}


fn serialize_variant_with_no_fields() -> proc_macro2::TokenStream {
    quote! {
        ::easy_config::expression::Expression::list(vec![])
    }
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
            let bindings: Vec<Ident> = (0..fields_unnamed.unnamed.len())
                .map(|i| syn::Ident::new(&format!("f{}", i), proc_macro2::Span::call_site()))
                .collect();
            quote! { ( #( #bindings ),* ) }
        }
        Fields::Unit => quote! {},
    };

    quote! {
        #enum_name::#variant_name #receiver => #to
    }
}

pub fn serialize_variant_arm(enum_name: &Ident, variant: &Variant) -> proc_macro2::TokenStream {

    let fields = match &variant.fields {
        Fields::Named(named) => serialize_named_fields(quote! {}, named),
        Fields::Unnamed(unnamed) => serialize_unnamed_fields(quote! {}, unnamed),
        Fields::Unit => serialize_variant_with_no_fields()
    };

    let variant_name_string = variant.ident.to_string();

    let container = quote! {
        ::easy_config::expression::Expression::list(vec![
            ::easy_config::expression::Expression::presence(#variant_name_string),
            #fields
        ])
    };
    prepend_arm(enum_name, variant, container)
}