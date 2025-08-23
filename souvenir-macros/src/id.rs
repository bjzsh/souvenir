use proc_macro::TokenStream;
use quote::quote;
use souvenir_core::{encoding::decode_prefix, id::Id};
use syn::{LitStr, parse_macro_input};

pub fn id(input: TokenStream) -> TokenStream {
    let literal = parse_macro_input!(input as LitStr);
    let value = literal.value();

    if let Ok(id) = Id::parse(&value) {
        let bytes = id.to_bytes();

        return quote! {
            ::souvenir::Id::from_bytes_unchecked([#(#bytes,)*])
        }
        .into();
    }

    if let Ok(prefix) = decode_prefix(&value) {
        let prefix = prefix as u128;

        return quote! {
            ::souvenir::Id::from_bytes_unchecked(
                (#prefix | (::rand::random::<u128>() & ((1 << 108) - 1)))
                    .to_be_bytes()
            )
        }
        .into();
    }

    panic!("id!(...) was not provided a valid input!");
}
