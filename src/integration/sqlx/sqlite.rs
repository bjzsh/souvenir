use crate::integration::stored::{StoredBinary, decode_stored_binary};
use crate::{Id, Souvenir, Storage, Tagged};
use sqlx::sqlite::{Sqlite, SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef};
use sqlx::{Decode, Encode, Type, encode::IsNull, error::BoxDynError};
use std::borrow::Cow;

impl Type<Sqlite> for Souvenir {
    fn type_info() -> SqliteTypeInfo {
        <&str as Type<Sqlite>>::type_info()
    }

    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <&str as Type<Sqlite>>::compatible(ty)
    }
}

impl<'q> Encode<'q, Sqlite> for Souvenir {
    fn encode_by_ref(
        &self,
        args: &mut Vec<SqliteArgumentValue<'q>>,
    ) -> Result<IsNull, BoxDynError> {
        args.push(SqliteArgumentValue::Text(Cow::Owned(self.to_string())));
        Ok(IsNull::No)
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

impl<'q, T: Tagged> Encode<'q, Sqlite> for Id<T> {
    fn encode_by_ref(
        &self,
        args: &mut Vec<SqliteArgumentValue<'q>>,
    ) -> Result<IsNull, BoxDynError> {
        args.push(match T::STORAGE {
            Storage::Text => SqliteArgumentValue::Text(Cow::Owned(self.to_string())),
            Storage::Bytes | Storage::Uuid => {
                SqliteArgumentValue::Blob(Cow::Owned(StoredBinary::of(self).as_slice().to_vec()))
            }
        });
        Ok(IsNull::No)
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
