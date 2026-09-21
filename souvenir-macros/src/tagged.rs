use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, LitStr, parse_macro_input};

pub fn tagged(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let tag = match find_tag(&input) {
        Ok(tag) => tag,
        Err(error) => return error.to_compile_error().into(),
    };
    let ident = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        impl #impl_generics ::souvenir::Tagged for #ident #type_generics #where_clause {
            const PREFIX: ::souvenir::Prefix = ::souvenir::prefix!(#tag);
        }
    }
    .into()
}

fn find_tag(input: &DeriveInput) -> syn::Result<LitStr> {
    let mut tag = None;
    for attr in input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("souvenir"))
    {
        attr.parse_nested_meta(|meta| {
            if !meta.path.is_ident("tag") {
                return Err(meta.error("expected tag = \"prefix\""));
            }
            if tag.is_some() {
                return Err(meta.error("only one tag may be specified"));
            }
            tag = Some(meta.value()?.parse()?);
            Ok(())
        })?;
    }
    tag.ok_or_else(|| {
        syn::Error::new_spanned(&input.ident, "missing #[souvenir(tag = \"prefix\")]")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_attributes_return_diagnostics() {
        for input in [
            syn::parse_quote!(
                struct Missing;
            ),
            syn::parse_quote!(
                #[souvenir(tag = 1)]
                struct WrongType;
            ),
            syn::parse_quote!(
                #[souvenir(tag = "a", tag = "b")]
                struct Duplicate;
            ),
            syn::parse_quote!(
                #[souvenir(other)]
                struct Unknown;
            ),
        ] {
            assert!(find_tag(&input).is_err());
        }
        let input = syn::parse_quote!(
            #[souvenir(tag = "user")]
            struct User;
        );
        assert_eq!(find_tag(&input).unwrap().value(), "user");
    }
}
