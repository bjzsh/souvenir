use proc_macro::TokenStream;
use quote::quote;
use souvenir_core::{encoding::decode_prefix, id::Id};
use syn::{LitStr, Path, parse::Parse, parse_macro_input};

enum IdInput {
    Literal(LitStr),
    Tagged(Path),
}

impl Parse for IdInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(if input.peek(LitStr) {
            Self::Literal(input.parse()?)
        } else {
            Self::Tagged(input.parse()?)
        })
    }
}

pub fn id(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as IdInput);

    match input {
        IdInput::Literal(literal) => {
            let value = literal.value();
            if let Ok(id) = Id::parse(&value) {
                let bytes = id.to_bytes();
                return quote! { ::souvenir::Id::new([#(#bytes,)*]) }.into();
            }
            if let Ok(prefix) = decode_prefix(&value) {
                let prefix = prefix.to_u32();
                return quote! {
                    ::souvenir::Id::random(unsafe {
                        ::souvenir::Prefix::new_unchecked(#prefix)
                    })
                }
                .into();
            }
            syn::Error::new(literal.span(), "expected a valid identifier or prefix")
                .to_compile_error()
                .into()
        }
        IdInput::Tagged(path) => quote! {
            ::souvenir::Id::random(<#path as ::souvenir::Tagged>::PREFIX)
        }
        .into(),
    }
}
