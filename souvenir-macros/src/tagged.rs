use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, DeriveInput, LitStr, parse_macro_input};

pub fn tagged(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident,
        generics,
        attrs,
        ..
    } = parse_macro_input!(input as DeriveInput);

    let tag = find_tag(attrs);
    let where_clause = &generics.where_clause;

    quote! {
        impl #generics ::souvenir::Tagged for #ident #generics #where_clause {
            const PREFIX: ::souvenir::Prefix = ::souvenir::prefix!(#tag);
        }
    }
    .into()
}

fn find_tag(attrs: Vec<Attribute>) -> LitStr {
    let filtered = attrs
        .into_iter()
        .filter(|attr| attr.path().is_ident("souvenir"));
    let mut tag: Option<LitStr> = None;

    for attr in filtered {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("tag") {
                let value = meta.value()?;
                let s: LitStr = value.parse()?;

                if tag.is_some() {
                    panic!("found multiple tags!");
                }

                tag = Some(s);
            }

            Ok(())
        })
        .unwrap()
    }

    tag.expect("could not find tag attr")
}
