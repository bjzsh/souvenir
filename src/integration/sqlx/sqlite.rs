use crate::integration::stored::{StoredBinary, decode_stored_binary};
use crate::{Id, Souvenir, Storage, Tagged};
use sqlx::sqlite::{Sqlite, SqliteArgumentsBuffer, SqliteTypeInfo, SqliteValueRef};
use sqlx::{Decode, Encode, Type, encode::IsNull, error::BoxDynError};

impl Type<Sqlite> for Souvenir {
    fn type_info() -> SqliteTypeInfo {
        <&str as Type<Sqlite>>::type_info()
    }

    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <&str as Type<Sqlite>>::compatible(ty)
    }
}

// The buffer can't be pushed to from outside sqlx, so encoding goes through
// sqlx's own `String` and `&[u8]` impls.

impl Encode<'_, Sqlite> for Souvenir {
    fn encode_by_ref(&self, args: &mut SqliteArgumentsBuffer) -> Result<IsNull, BoxDynError> {
        <String as Encode<'_, Sqlite>>::encode(self.to_string(), args)
    }
}

impl Decode<'_, Sqlite> for Souvenir {
    fn decode(value: SqliteValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(Self::parse(<&str as Decode<Sqlite>>::decode(value)?)?)
    }
}

impl<T: Tagged> Type<Sqlite> for Id<T> {
    fn type_info() -> SqliteTypeInfo {
        match T::STORAGE {
            Storage::Text => <&str as Type<Sqlite>>::type_info(),
            Storage::Bytes | Storage::Uuid => <&[u8] as Type<Sqlite>>::type_info(),
        }
    }

    fn compatible(ty: &SqliteTypeInfo) -> bool {
        match T::STORAGE {
            Storage::Text => <&str as Type<Sqlite>>::compatible(ty),
            Storage::Bytes | Storage::Uuid => <&[u8] as Type<Sqlite>>::compatible(ty),
        }
    }
}

impl<T: Tagged> Encode<'_, Sqlite> for Id<T> {
    fn encode_by_ref(&self, args: &mut SqliteArgumentsBuffer) -> Result<IsNull, BoxDynError> {
        match T::STORAGE {
            Storage::Text => <String as Encode<'_, Sqlite>>::encode(self.to_string(), args),
            Storage::Bytes | Storage::Uuid => {
                <&[u8] as Encode<'_, Sqlite>>::encode(StoredBinary::of(self).as_slice(), args)
            }
        }
    }
}

impl<T: Tagged> Decode<'_, Sqlite> for Id<T> {
    fn decode(value: SqliteValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(match T::STORAGE {
            Storage::Text => Self::parse(<&str as Decode<Sqlite>>::decode(value)?)?,
            Storage::Bytes | Storage::Uuid => {
                decode_stored_binary(<&[u8] as Decode<Sqlite>>::decode(value)?)?
            }
        })
    }
}
