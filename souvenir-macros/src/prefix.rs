use proc_macro::TokenStream;
use quote::quote;
use souvenir_core::prefix::Prefix;
use syn::{LitStr, parse_macro_input};

pub fn prefix(input: TokenStream) -> TokenStream {
    let literal = parse_macro_input!(input as LitStr);
    let value = literal.value();

    if let Ok(prefix) = Prefix::parse(&value) {
        let raw = prefix.to_u32();

        return quote! {
            unsafe { ::souvenir::Prefix::new_unchecked(#raw) }
        }
        .into();
    }

    panic!("\"{}\" is not a valid prefix!", value);
}
