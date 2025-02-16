#[cfg(feature = "diesel-postgres")]
mod pg {
    use crate::{Id, Type};
    use diesel::pg::{Pg, PgValue};
    use diesel::{deserialize, serialize, sql_types::Uuid};
    use std::io::Write;

    impl<T: Type> serialize::ToSql<Uuid, Pg> for Id<T> {
        fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, Pg>) -> serialize::Result {
            out.write_all(self.as_bytes())
                .map(|_| serialize::IsNull::No)
                .map_err(Into::into)
        }
    }

    impl<T: Type> deserialize::FromSql<Uuid, Pg> for Id<T> {
        fn from_sql(value: PgValue<'_>) -> deserialize::Result<Self> {
            value.as_bytes().try_into().map_err(Into::into)
        }
    }
}

#[cfg(feature = "diesel-mysql")]
mod mysql {
    use crate::{Id, Type};
    use diesel::mysql::{Mysql, MysqlValue};
    use diesel::{deserialize, serialize, sql_types::Binary};
    use std::io::Write;

    impl<T: Type> serialize::ToSql<Binary, Mysql> for Id<T> {
        fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, Mysql>) -> serialize::Result {
            out.write_all(self.as_bytes())
                .map(|_| serialize::IsNull::No)
                .map_err(Into::into)
        }
    }

    impl<T: Type> deserialize::FromSql<Binary, Mysql> for Id<T> {
        fn from_sql(value: MysqlValue<'_>) -> deserialize::Result<Self> {
            value.as_bytes().try_into().map_err(Into::into)
        }
    }
}

#[cfg(feature = "diesel-sqlite")]
mod sqlite {
    use crate::{Id, Type};
    use diesel::sqlite::{Sqlite, SqliteValue};
    use diesel::{deserialize, serialize, sql_types::Text};

    impl<T: Type> serialize::ToSql<Text, Sqlite> for Id<T> {
        fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, Sqlite>) -> serialize::Result {
            out.set_value(self.to_string());
            Ok(serialize::IsNull::No)
        }
    }

    impl<T: Type> deserialize::FromSql<Text, Sqlite> for Id<T> {
        fn from_sql(value: SqliteValue<'_, '_, '_>) -> deserialize::Result<Self> {
            <String as deserialize::FromSql<Text, Sqlite>>::from_sql(value)?
                .parse()
                .map_err(Into::into)
        }
    }
}
