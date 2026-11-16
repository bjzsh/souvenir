use super::support::write_value;
use crate::sql_types::Souvenir as SouvenirSql;
use crate::{Error, Id, Souvenir, Storage, Tagged};
use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{self, IsNull, ToSql};
use diesel::sql_types::Uuid;
use std::io::Write;

backend_to_sql!(Pg, |out, value| write_value(out, value));

impl<T: Tagged> ToSql<Uuid, Pg> for Id<T> {
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, Pg>) -> serialize::Result {
        require_storage!(
            T,
            Storage::Uuid,
            "a Uuid column needs a Tagged type with Storage::Uuid"
        );
        out.write_all(self.to_uuid().as_bytes())?;
        Ok(IsNull::No)
    }
}

impl<T: Tagged> FromSql<Uuid, Pg> for Id<T> {
    fn from_sql(value: PgValue<'_>) -> deserialize::Result<Self> {
        require_storage!(
            T,
            Storage::Uuid,
            "a Uuid column needs a Tagged type with Storage::Uuid"
        );
        let uuid = ::uuid::Uuid::from_slice(value.as_bytes()).map_err(|_| Error::InvalidLength)?;
        Ok(Self::from_uuid(uuid)?)
    }
}

// The extension's `souvenir` type. Diesel's Postgres connection reads and
// writes it in binary, which is the 20 bytes.

impl ToSql<SouvenirSql, Pg> for Souvenir {
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, Pg>) -> serialize::Result {
        out.write_all(self.as_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<SouvenirSql, Pg> for Souvenir {
    fn from_sql(value: PgValue<'_>) -> deserialize::Result<Self> {
        Ok(Self::from_slice(value.as_bytes())?)
    }
}

impl<T: Tagged> ToSql<SouvenirSql, Pg> for Id<T> {
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, Pg>) -> serialize::Result {
        require_storage!(
            T,
            Storage::Native,
            "a souvenir column needs a Tagged type with Storage::Native"
        );
        out.write_all(self.as_bytes())?;
        Ok(IsNull::No)
    }
}

impl<T: Tagged> FromSql<SouvenirSql, Pg> for Id<T> {
    fn from_sql(value: PgValue<'_>) -> deserialize::Result<Self> {
        require_storage!(
            T,
            Storage::Native,
            "a souvenir column needs a Tagged type with Storage::Native"
        );
        Ok(Self::from_slice(value.as_bytes())?)
    }
}
