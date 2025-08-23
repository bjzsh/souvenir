#[cfg(feature = "postgres")]
mod pg {
    use crate::id::Id;
    use diesel::pg::{Pg, PgValue};
    use diesel::{deserialize, serialize, sql_types::Uuid};
    use std::io::Write;

    impl serialize::ToSql<Uuid, Pg> for Id {
        fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, Pg>) -> serialize::Result {
            out.write_all(self.as_bytes())
                .map(|_| serialize::IsNull::No)
                .map_err(Into::into)
        }
    }

    impl deserialize::FromSql<Uuid, Pg> for Id {
        fn from_sql(value: PgValue<'_>) -> deserialize::Result<Self> {
            value.as_bytes().try_into().map_err(Into::into)
        }
    }
}

#[cfg(feature = "mysql")]
mod mysql {
    use crate::id::Id;
    use diesel::mysql::{Mysql, MysqlValue};
    use diesel::{deserialize, serialize, sql_types::Binary};
    use std::io::Write;

    impl serialize::ToSql<Binary, Mysql> for Id {
        fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, Mysql>) -> serialize::Result {
            out.write_all(self.as_bytes())
                .map(|_| serialize::IsNull::No)
                .map_err(Into::into)
        }
    }

    impl deserialize::FromSql<Binary, Mysql> for Id {
        fn from_sql(value: MysqlValue<'_>) -> deserialize::Result<Self> {
            value.as_bytes().try_into().map_err(Into::into)
        }
    }
}

#[cfg(feature = "sqlite")]
mod sqlite {
    use crate::id::Id;
    use diesel::sqlite::{Sqlite, SqliteValue};
    use diesel::{deserialize, serialize, sql_types::Text};

    impl serialize::ToSql<Text, Sqlite> for Id {
        fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, Sqlite>) -> serialize::Result {
            out.set_value(self.to_string());
            Ok(serialize::IsNull::No)
        }
    }

    impl deserialize::FromSql<Text, Sqlite> for Id {
        fn from_sql(value: SqliteValue<'_, '_, '_>) -> deserialize::Result<Self> {
            <String as deserialize::FromSql<Text, Sqlite>>::from_sql(value)?
                .parse()
                .map_err(Into::into)
        }
    }
}
