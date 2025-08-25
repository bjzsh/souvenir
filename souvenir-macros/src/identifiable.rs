use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Field, Fields, Generics, Ident, parse_macro_input};

pub fn identifiable(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident,
        data,
        generics,
        ..
    } = parse_macro_input!(input as DeriveInput);

    match data {
        Data::Struct(data) => identifiable_for_struct(ident, generics, data),
        Data::Enum(_data) => panic!("#[derive(Identifiable)] cannot be used on enums!"),
        Data::Union(_data) => panic!("#[derive(Identifiable)] cannot be used on unions!"),
    }
}

fn identifiable_for_struct(ident: Ident, generics: Generics, data: DataStruct) -> TokenStream {
    let target = find_target_field(data.fields);

    let where_clause = &generics.where_clause;
    let target_ident = target.ident.unwrap();

    quote! {
        impl #generics ::souvenir::Identifiable for #ident #generics #where_clause {
            fn id(&self) -> ::souvenir::Id {
                self.#target_ident
            }
        }
    }
    .into()
}

fn is_suitable_field(field: &Field) -> bool {
    field.attrs.iter().any(|attr| {
        attr.path().is_ident("souvenir")
            && attr
                .parse_nested_meta(|meta| {
                    if meta.path.is_ident("id") {
                        Ok(())
                    } else {
                        Err(meta.error("unrecognized repr"))
                    }
                })
                .is_ok()
    })
}

fn find_target_field(fields: Fields) -> Field {
    let fields: Vec<_> = match fields {
        Fields::Unit => vec![],
        Fields::Unnamed(f) => f.unnamed.into_iter().filter(is_suitable_field).collect(),
        Fields::Named(f) => f.named.into_iter().filter(is_suitable_field).collect(),
    };

    if fields.len() > 1 {
        panic!("only one field can be specified as #[identifier]");
    }

    fields
        .into_iter()
        .next()
        .expect("could not find suitable id field! please specify manually")
}
