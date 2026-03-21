struct EasyConfigAttrs {
    default: bool,
    comment: Option<String>,
}

impl EasyConfigAttrs {
    fn from_attrs(attrs: &[syn::Attribute]) -> Result<Self, syn::Error> {
        let mut default = false;
        let mut comment = None;

        // capture doc comments automatically
        for attr in attrs {
            if attr.path().is_ident("doc") {
                if let syn::Meta::NameValue(nv) = &attr.meta {
                    if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &nv.value {
                        let doc = s.value().trim().to_string();
                        comment = Some(match comment {
                            Some(existing) => format!("{}\n{}", existing, doc),
                            None => doc,
                        });
                    }
                }
            }
            if attr.path().is_ident("easy_config") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("default") {
                        default = true;
                        Ok(())
                    } else if meta.path.is_ident("comment") {
                        let value = meta.value()?;
                        let s: syn::LitStr = value.parse()?;
                        comment = Some(s.value());
                        Ok(())
                    } else {
                        Err(meta.error("unknown easy_config attribute"))
                    }
                })?;
            }
        }
        Ok(Self { default, comment })
    }
}