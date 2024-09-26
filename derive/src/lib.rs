use proc_macro::TokenStream;
use quote::quote;
use syn;
use syn::DeriveInput;

#[proc_macro_derive(Serialize)]
pub fn serialize(item: TokenStream) -> TokenStream {
    // Parse the input TokenStream into a DeriveInput syntax tree
    let ast: DeriveInput = syn::parse(item).unwrap();

    // Get the name of the struct
    let name = &ast.ident;

    // Match the data of the input to ensure it's a struct
    let fields = if let syn::Data::Struct(syn::DataStruct { fields: syn::Fields::Named(ref fields), .. }) = ast.data {
        fields
    } else {
        panic!("Serialize can only be derived for structs with named fields");
    };

    // Iterate over each field and generate Expression::Pair for each field
    let field_serializations = fields.named.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap(); // Get the field name
        let field_name_str = field_name.to_string(); // Convert field name to string

        quote! {
            Expression::Pair(#field_name_str.to_string(), Box::new(self.#field_name.serialize()))
        }
    });

    // Generate the implementation of the Serialize trait
    let gen = quote! {
        impl Serialize for #name {
            fn serialize(&self) -> Expression {
                Expression::Collection(vec![
                    #(#field_serializations),*
                ])
            }
        }
    };

    // Return the generated code as a TokenStream
    gen.into()
}
