use super::codec::pg_encode_text;
use super::compat::{
    PG_SOUVENIR, PG_SOUVENIR_ARRAY, PG_UUID, PG_UUID_ARRAY, pg_bytea, pg_native, pg_text, pg_uuid,
};
use crate::integration::stored::{StoredBinary, decode_stored_binary};
use crate::{Error, Id, Souvenir, Storage, Tagged};
use sqlx::postgres::{
    PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef, Postgres,
};
use sqlx::{Decode, Encode, Type, encode::IsNull, error::BoxDynError};
use uuid::Uuid;

// Untyped IDs are written as text (which converts to the extension's type on
// assignment) and read from text or the extension's type.
impl Type<Postgres> for Souvenir {
    fn type_info() -> PgTypeInfo {
        <&str as Type<Postgres>>::type_info()
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        pg_text(ty) || pg_native(ty)
    }
}

impl PgHasArrayType for Souvenir {
    fn array_type_info() -> PgTypeInfo {
        <String as PgHasArrayType>::array_type_info()
    }
}

impl Encode<'_, Postgres> for Souvenir {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        pg_encode_text(*self, buf);
        Ok(IsNull::No)
    }
}

impl Decode<'_, Postgres> for Souvenir {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        decode_text_or_native(value, Self::parse, Self::from_slice)
    }
}

/// Read text, or the extension's type in either wire format: its binary
/// format is the 20 bytes, its text format the ID string.
fn decode_text_or_native<T>(
    value: PgValueRef<'_>,
    parse: fn(&str) -> crate::Result<T>,
    from_slice: fn(&[u8]) -> crate::Result<T>,
) -> Result<T, BoxDynError> {
    let native = pg_native(&sqlx::ValueRef::type_info(&value));
    Ok(match value.format() {
        PgValueFormat::Binary if native => from_slice(value.as_bytes()?)?,
        _ => parse(value.as_str()?)?,
    })
}

impl<T: Tagged> Type<Postgres> for Id<T> {
    fn type_info() -> PgTypeInfo {
        match T::STORAGE {
            Storage::Text => <&str as Type<Postgres>>::type_info(),
            Storage::Bytes => <Vec<u8> as Type<Postgres>>::type_info(),
            Storage::Uuid => PgTypeInfo::with_oid(PG_UUID),
            Storage::Native => PgTypeInfo::with_name(PG_SOUVENIR),
        }
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        match T::STORAGE {
            Storage::Text => pg_text(ty),
            Storage::Bytes => pg_bytea(ty),
            Storage::Uuid => pg_uuid(ty),
            Storage::Native => pg_native(ty),
        }
    }
}

impl<T: Tagged> PgHasArrayType for Id<T> {
    fn array_type_info() -> PgTypeInfo {
        match T::STORAGE {
            Storage::Text => <String as PgHasArrayType>::array_type_info(),
            Storage::Bytes => <Vec<u8> as PgHasArrayType>::array_type_info(),
            Storage::Uuid => PgTypeInfo::with_oid(PG_UUID_ARRAY),
            Storage::Native => PgTypeInfo::with_name(PG_SOUVENIR_ARRAY),
        }
    }
}

impl<T: Tagged> Encode<'_, Postgres> for Id<T> {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        match T::STORAGE {
            Storage::Text => pg_encode_text(self.to_untyped(), buf),
            Storage::Bytes | Storage::Native | Storage::Uuid => {
                buf.extend_from_slice(StoredBinary::of(self).as_slice())
            }
        }
        Ok(IsNull::No)
    }
}

impl<T: Tagged> Decode<'_, Postgres> for Id<T> {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(match T::STORAGE {
            Storage::Text => Self::parse(value.as_str()?)?,
            // Vec<u8> handles both the binary and the `\x...` text format.
            Storage::Bytes => decode_stored_binary(&<Vec<u8> as Decode<Postgres>>::decode(value)?)?,
            Storage::Native => decode_text_or_native(value, Self::parse, Self::from_slice)?,
            Storage::Uuid => {
                let uuid = match value.format() {
                    PgValueFormat::Binary => {
                        Uuid::from_slice(value.as_bytes()?).map_err(|_| Error::InvalidLength)?
                    }
                    PgValueFormat::Text => {
                        Uuid::try_parse(value.as_str()?).map_err(|_| Error::InvalidCharacter)?
                    }
                };
                Self::from_uuid(uuid)?
            }
        })
    }
}
