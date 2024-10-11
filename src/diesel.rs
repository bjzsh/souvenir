use crate::{Id, Identifiable};
use diesel::{backend::Backend, deserialize, sql_types::BigInt};

macro_rules! to_sql_raw {
    ($db: ty) => {
        use crate::{Id, Identifiable};
        use diesel::{serialize, sql_types::BigInt};

        impl<T: Identifiable> serialize::ToSql<BigInt, $db> for Id<T>
        where
            i64: serialize::ToSql<diesel::sql_types::BigInt, $db>,
        {
            fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, $db>) -> serialize::Result {
                let bytes = i64::from_be_bytes(self.value());
                <i64 as serialize::ToSql<BigInt, $db>>::to_sql(&bytes, &mut out.reborrow())
            }
        }
    };
}

#[cfg(feature = "diesel-postgres")]
mod pg {
    to_sql_raw!(::diesel::pg::Pg);
}

#[cfg(feature = "diesel-mysql")]
mod mysql {
    to_sql_raw!(::diesel::mysql::Mysql);
}

macro_rules! to_sql_default {
    ($db: ty) => {
        use crate::{Id, Identifiable};
        use diesel::{serialize, sql_types::BigInt};

        impl<T: Identifiable> serialize::ToSql<BigInt, $db> for Id<T>
        where
            i64: serialize::ToSql<BigInt, $db>,
        {
            fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, $db>) -> serialize::Result {
                out.set_value(i64::from_be_bytes(self.value()));
                Ok(serialize::IsNull::No)
            }
        }
    };
}

#[cfg(feature = "diesel-sqlite")]
mod sqlite {
    to_sql_default!(::diesel::sqlite::Sqlite);
}

impl<T: Identifiable, B: Backend> deserialize::FromSql<BigInt, B> for Id<T>
where
    i64: deserialize::FromSql<BigInt, B>,
{
    fn from_sql(bytes: <B as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        <i64 as deserialize::FromSql<BigInt, B>>::from_sql(bytes)
            .map(|id| Id::new(id.to_be_bytes()))
    }
}
