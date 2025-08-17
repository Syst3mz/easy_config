use proc_macro2::Ident;
use quote::quote;
use syn::FieldsNamed;
use crate::shared::comma_separated_list;

pub fn deserialize_named_fields(fields_named: &FieldsNamed, struct_name: impl AsRef<str>) -> proc_macro2::TokenStream {
    let struct_name = struct_name.as_ref();
    let fields = fields_named.named.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();
        let name_str = ident.to_string();
        quote! {
            #ident: ::easy_config::serialization::deserialize_field_from_map_or_error(#name_str, &mut mapping, error_span, source_text)
            .contextualize(format!("Unable to read a {} because the mandatory field {} is not present or could not be deserialized.", #struct_name, #name_str))?
        }
    });
    let fields = comma_separated_list(fields);
    quote! {
        use ::easy_config::config_error::Contextualize;
        let (mut mapping, error_span) = exprs.convert_binding_list_to_hashmap_of_values(source_text)?;
        Ok(Self {
            #fields
        })
    }
}
pub fn deserialize_unit_struct(struct_name: &Ident) -> proc_macro2::TokenStream {
    quote! {return #struct_name {};}
}