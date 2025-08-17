use quote::{ToTokens};

pub fn separated_list<I, T, S>(entries: I, separator: S) -> proc_macro2::TokenStream
where
    I: IntoIterator<Item = T>,
    T: ToTokens,
    S: ToTokens,
{
    let mut iter = entries.into_iter().peekable();
    let mut out = proc_macro2::TokenStream::new();

    while let Some(entry) = iter.next() {
        entry.to_tokens(&mut out);
        if iter.peek().is_some() {
            separator.to_tokens(&mut out);
        }
    }

    out
}

pub fn comma_separated_list<I, T>(entries: I) -> proc_macro2::TokenStream
where
    I: IntoIterator<Item = T>,
    T: ToTokens,
{
    separated_list(entries, proc_macro2::Punct::new(',', proc_macro2::Spacing::Alone))
}