use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DataEnum, DataStruct, DeriveInput, Field};

fn extract_comment(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("Comment") {
            if let Ok(lit) = attr.parse_args::<syn::LitStr>() {
                return Some(lit.value());
            }
        }
    }
    None
}

fn serialize_unit_field() -> proc_macro2::TokenStream {
    quote! { Expression::list(vec![]) }
}

fn string_from(string: impl AsRef<str>) -> proc_macro2::TokenStream {
    let string = string.as_ref();
    quote! { String::from(#string) }
}
fn serialize_named_field(field: &Field) -> proc_macro2::TokenStream {
    let field_ident = field.ident.as_ref().unwrap();
    let field_name = string_from(field_ident.to_string());

    let uncommented = quote! {
        Expression::binding(
            #field_name,
            self.#field_ident.serialize()
        )
    };

    if let Some(comment) = extract_comment(&field.attrs) {
        quote! { #uncommented.with_comment(#comment) }
    } else {
        uncommented
    }
}

fn generate_config_for_struct(input: &DeriveInput, data: &syn::DataStruct) -> TokenStream {
    let struct_name = &input.ident;

    let serialize_body = match &data.fields {
        syn::Fields::Named(fields_named) => {
            let entries = fields_named.named.iter().flat_map(|field| {
                serialize_named_field(&field)
            });

            quote! {
                Expression::list(vec![
                    #(#entries),*
                ])
            }
        }
        syn::Fields::Unnamed(fields_unnamed) => {
            let indices: Vec<syn::Index> = (0..fields_unnamed.unnamed.len())
                .map(|i| syn::Index::from(i))
                .collect();

            let entries = indices.iter().map(|index| {
                quote! {
                    self.#index.serialize()
                }
            });

            quote! {
                Expression::list(vec![
                    #(#entries),*
                ])
            }
        }
        syn::Fields::Unit => serialize_unit_field()
    };

    quote! {
        use easy_config::expression::{Expression, ExpressionIterator};
        use easy_config::serialization::serialization_error::SerializationError;
        impl Config for #struct_name {
            fn serialize(&self) -> Expression {
                #serialize_body
            }

            fn deserialize(exprs: &mut ExpressionIterator, source_text: impl AsRef<str>) -> Result<Self, SerializationError> {
                todo!("deserialize for struct {}", stringify!(#struct_name));
            }
        }
    }
        .into()
}

fn generate_config_for_enum(p0: &DeriveInput, p1: &DataEnum) -> TokenStream {
    todo!()
}

#[proc_macro_derive(Config, attributes(Comment))]
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