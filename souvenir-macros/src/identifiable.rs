use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Member, parse_macro_input};

pub fn identifiable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let fields = match &input.data {
        Data::Struct(data) => &data.fields,
        _ => {
            return syn::Error::new_spanned(&input.ident, "Identifiable requires a struct")
                .to_compile_error()
                .into();
        }
    };
    let target = match find_target_field(fields) {
        Ok(target) => target,
        Err(error) => return error.to_compile_error().into(),
    };
    let ident = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    quote! {
        impl #impl_generics ::souvenir::Identifiable for #ident #type_generics #where_clause {
            fn id(&self) -> ::souvenir::Id { self.#target }
        }
    }
    .into()
}

fn find_target_field(fields: &Fields) -> syn::Result<Member> {
    let mut target = None;
    for (index, field) in fields.iter().enumerate() {
        for attr in field
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("souvenir"))
        {
            attr.parse_nested_meta(|meta| {
                if !meta.path.is_ident("id") {
                    return Err(meta.error("expected id"));
                }
                if target.is_some() {
                    return Err(meta.error("only one #[souvenir(id)] field may be specified"));
                }
                target = Some(match &field.ident {
                    Some(ident) => Member::Named(ident.clone()),
                    None => Member::Unnamed(index.into()),
                });
                Ok(())
            })?;
        }
    }
    target.ok_or_else(|| syn::Error::new_spanned(fields, "missing #[souvenir(id)] field"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fields(source: &str) -> Fields {
        match syn::parse_str::<DeriveInput>(source).unwrap().data {
            Data::Struct(data) => data.fields,
            _ => panic!("test fixture must be a struct"),
        }
    }

    #[test]
    fn invalid_fields_return_diagnostics() {
        for source in [
            "struct A { id: Id }",
            "struct A { #[souvenir(id)] a: Id, #[souvenir(id)] b: Id }",
            "struct A { #[souvenir(other)] id: Id }",
            "struct A { #[souvenir(id = 1)] id: Id }",
        ] {
            assert!(find_target_field(&fields(source)).is_err());
        }
    }

    #[test]
    fn named_and_tuple_fields_are_supported() {
        let named = fields("struct A { #[souvenir(id)] id: Id }");
        assert!(matches!(find_target_field(&named), Ok(Member::Named(_))));
        let tuple = fields("struct A(u8, #[souvenir(id)] Id);");
        assert!(
            matches!(find_target_field(&tuple), Ok(Member::Unnamed(index)) if index.index == 1)
        );
    }
}
