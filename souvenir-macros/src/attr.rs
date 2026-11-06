use syn::{Attribute, Ident, LitStr, Path, Result};

#[derive(Default)]
pub struct SouvenirAttrs {
    pub prefix: Option<LitStr>,
    pub storage: Option<Ident>,
    pub id: bool,
}

pub fn parse(attrs: &[Attribute]) -> Result<SouvenirAttrs> {
    let mut out = SouvenirAttrs::default();

    for attr in attrs.iter().filter(|a| a.path().is_ident("souvenir")) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("prefix") {
                if out.prefix.is_some() {
                    return Err(meta.error("duplicate `prefix`"));
                }
                out.prefix = Some(meta.value()?.parse()?);
                Ok(())
            } else if meta.path.is_ident("storage") {
                if out.storage.is_some() {
                    return Err(meta.error("duplicate `storage`"));
                }
                let path: Path = meta.value()?.parse()?;
                out.storage = Some(path.require_ident()?.clone());
                Ok(())
            } else if meta.path.is_ident("id") {
                if out.id {
                    return Err(meta.error("duplicate `id`"));
                }
                out.id = true;
                Ok(())
            } else {
                Err(meta
                    .error("expected `prefix = \"...\"`, `storage = Text | Bytes | Uuid` or `id`"))
            }
        })?;
    }

    Ok(out)
}
