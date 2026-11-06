use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::parse::{Parse, ParseStream};
use syn::{LitStr, Token, Type, parse2};

struct IdInput {
    ty: Option<Type>,
    literal: LitStr,
}

impl Parse for IdInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(LitStr) {
            let literal = input.parse()?;
            return Ok(Self { ty: None, literal });
        }
        let ty = input.parse()?;
        input.parse::<Token![,]>()?;
        let literal = input.parse()?;
        Ok(Self {
            ty: Some(ty),
            literal,
        })
    }
}

pub fn id(input: TokenStream) -> TokenStream {
    let IdInput { ty, literal } = match parse2(input) {
        Ok(input) => input,
        Err(e) => return e.to_compile_error(),
    };

    let parsed = quote_spanned! { literal.span() =>
        match ::souvenir::Souvenir::parse(#literal) {
            ::core::result::Result::Ok(id) => id,
            ::core::result::Result::Err(e) => ::core::panic!("{}", e.message()),
        }
    };

    match ty {
        None => quote! {{
            const ID: ::souvenir::Souvenir = #parsed;
            ID
        }},
        Some(ty) => {
            let typed = quote_spanned! { literal.span() =>
                const {
                    match ::souvenir::Id::<#ty>::parse(#literal) {
                        ::core::result::Result::Ok(id) => id,
                        ::core::result::Result::Err(e) => ::core::panic!("{}", e.message()),
                    }
                }
            };

            quote! {{
                const _: ::souvenir::Souvenir = #parsed;
                #typed
            }}
        }
    }
}
