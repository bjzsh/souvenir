use crate::attr;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{DeriveInput, parse2};

pub fn derive(input: TokenStream) -> TokenStream {
    expand(input).unwrap_or_else(syn::Error::into_compile_error)
}

fn expand(input: TokenStream) -> syn::Result<TokenStream> {
    let input: DeriveInput = parse2(input)?;
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let attrs = attr::parse(&input.attrs)?;

    let prefix = attrs.prefix.ok_or_else(|| {
        syn::Error::new_spanned(ident, "missing #[souvenir(prefix = \"...\")] attribute")
    })?;

    if let Some(form) = &attrs.storage {
        if !matches!(form.to_string().as_str(), "Text" | "Bytes" | "Uuid") {
            return Err(syn::Error::new_spanned(
                form,
                "storage must be `Text`, `Bytes` or `Uuid`",
            ));
        }
    }

    let storage = attrs.storage.as_ref().map(|form| {
        quote! { const STORAGE: ::souvenir::Storage = ::souvenir::Storage::#form; }
    });

    // `prefix!` checks in a const item of its own, so `cargo check` reports
    // an invalid prefix even when the impl is generic.
    let checked = quote_spanned! { prefix.span() =>
        ::souvenir::prefix!(#prefix)
    };

    Ok(quote! {
        impl #impl_generics ::souvenir::Tagged for #ident #ty_generics #where_clause {
            const PREFIX: ::souvenir::Prefix = #checked;
            #storage
        }
    })
}
