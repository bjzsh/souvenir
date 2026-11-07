use super::codec::pg_encode_text;
use super::compat::{PG_UUID, PG_UUID_ARRAY, pg_bytea, pg_text, pg_uuid};
use crate::integration::stored::{StoredBinary, decode_stored_binary};
use crate::{Error, Id, Souvenir, Storage, Tagged};
use sqlx::postgres::{
    PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef, Postgres,
};
use sqlx::{Decode, Encode, Type, encode::IsNull, error::BoxDynError};
use uuid::Uuid;

impl Type<Postgres> for Souvenir {
    fn type_info() -> PgTypeInfo {
        <&str as Type<Postgres>>::type_info()
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        pg_text(ty)
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
        Ok(Self::parse(value.as_str()?)?)
    }
}

impl<T: Tagged> Type<Postgres> for Id<T> {
    fn type_info() -> PgTypeInfo {
        match T::STORAGE {
            Storage::Text => <&str as Type<Postgres>>::type_info(),
            Storage::Bytes => <Vec<u8> as Type<Postgres>>::type_info(),
            Storage::Uuid => PgTypeInfo::with_oid(PG_UUID),
        }
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        match T::STORAGE {
            Storage::Text => pg_text(ty),
            Storage::Bytes => pg_bytea(ty),
            Storage::Uuid => pg_uuid(ty),
        }
    }
}

impl<T: Tagged> PgHasArrayType for Id<T> {
    fn array_type_info() -> PgTypeInfo {
        match T::STORAGE {
            Storage::Text => <String as PgHasArrayType>::array_type_info(),
            Storage::Bytes => <Vec<u8> as PgHasArrayType>::array_type_info(),
            Storage::Uuid => PgTypeInfo::with_oid(PG_UUID_ARRAY),
        }
    }
}

impl<T: Tagged> Encode<'_, Postgres> for Id<T> {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        match T::STORAGE {
            Storage::Text => pg_encode_text(self.to_untyped(), buf),
            Storage::Bytes | Storage::Uuid => {
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
