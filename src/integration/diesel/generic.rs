// `ToSql` for `Text` and for `Id<T>` needs each backend's output buffer, so
// the backend modules supply those.

use crate::integration::stored::decode_stored_binary;
use crate::{Id, Souvenir, Storage, Tagged};
use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, ToSql};
use diesel::sql_types::{Binary, Text};

impl<DB: Backend> ToSql<Binary, DB> for Souvenir
where
    [u8]: ToSql<Binary, DB>,
{
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, DB>) -> serialize::Result {
        <[u8] as ToSql<Binary, DB>>::to_sql(self.as_bytes().as_slice(), out)
    }
}

impl<DB: Backend> FromSql<Binary, DB> for Souvenir
where
    Vec<u8>: FromSql<Binary, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
        Ok(Self::from_slice(
            &<Vec<u8> as FromSql<Binary, DB>>::from_sql(bytes)?,
        )?)
    }
}

impl<DB: Backend> FromSql<Text, DB> for Souvenir
where
    String: FromSql<Text, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
        Ok(Self::parse(&<String as FromSql<Text, DB>>::from_sql(
            bytes,
        )?)?)
    }
}

impl<DB: Backend, T: Tagged> FromSql<Text, DB> for Id<T>
where
    String: FromSql<Text, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
        require_storage!(
            T,
            Storage::Text,
            "a Text column needs a Tagged type with Storage::Text"
        );
        Ok(Self::parse(&<String as FromSql<Text, DB>>::from_sql(
            bytes,
        )?)?)
    }
}

impl<DB: Backend, T: Tagged> FromSql<Binary, DB> for Id<T>
where
    Vec<u8>: FromSql<Binary, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
        require_storage!(
            T,
            Storage::Bytes | Storage::Uuid,
            "a Binary column needs a Tagged type with Storage::Bytes or Storage::Uuid"
        );
        let bytes = <Vec<u8> as FromSql<Binary, DB>>::from_sql(bytes)?;
        Ok(decode_stored_binary(&bytes)?)
    }
}
