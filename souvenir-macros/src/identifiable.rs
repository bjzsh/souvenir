use crate::attr;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Index, Member, parse2};

pub fn derive(input: TokenStream) -> TokenStream {
    expand(input).unwrap_or_else(syn::Error::into_compile_error)
}

fn expand(input: TokenStream) -> syn::Result<TokenStream> {
    let input: DeriveInput = parse2(input)?;
    let ident = &input.ident;
    let Data::Struct(data) = &input.data else {
        return Err(syn::Error::new_spanned(
            ident,
            "#[derive(Identifiable)] can only be used on structs",
        ));
    };

    let mut found = None;
    for (i, field) in data.fields.iter().enumerate() {
        if !attr::parse(&field.attrs)?.id {
            continue;
        }
        if found.is_some() {
            return Err(syn::Error::new_spanned(
                field,
                "only one field can be marked #[souvenir(id)]",
            ));
        }
        let member = match &field.ident {
            Some(name) => Member::Named(name.clone()),
            None => Member::Unnamed(Index::from(i)),
        };
        found = Some((member, &field.ty));
    }
    // Without a marked field, use the one named `id`.
    let found = found.or_else(|| {
        data.fields.iter().find_map(|field| {
            let name = field.ident.as_ref().filter(|name| *name == "id")?;
            Some((Member::Named(name.clone()), &field.ty))
        })
    });
    let (member, ty) = found.ok_or_else(|| {
        syn::Error::new_spanned(
            ident,
            "add a field named `id`, or mark the ID field with #[souvenir(id)]",
        )
    })?;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics ::souvenir::Identifiable for #ident #ty_generics #where_clause {
            type Tag = <#ty as ::souvenir::Identifiable>::Tag;

            fn id(&self) -> ::souvenir::Id<Self::Tag> {
                ::souvenir::Identifiable::id(&self.#member)
            }
        }
    })
}
