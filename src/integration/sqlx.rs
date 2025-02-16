#[cfg(feature = "sqlx-postgres")]
mod pg {
    use crate::{Id, Type};
    use sqlx::postgres::{
        types::Oid, PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef,
        Postgres,
    };
    use sqlx::{encode::IsNull, error::BoxDynError, Decode, Encode};

    impl<T: Type> sqlx::Type<Postgres> for Id<T> {
        fn type_info() -> PgTypeInfo {
            PgTypeInfo::with_oid(Oid(2950))
        }
    }

    impl<T: Type> PgHasArrayType for Id<T> {
        fn array_type_info() -> PgTypeInfo {
            PgTypeInfo::with_oid(Oid(2951))
        }
    }

    impl<T: Type> Encode<'_, Postgres> for Id<T> {
        fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
            buf.extend_from_slice(self.as_bytes());
            Ok(IsNull::No)
        }
    }

    impl<T: Type> Decode<'_, Postgres> for Id<T> {
        fn decode(value: PgValueRef) -> Result<Self, BoxDynError> {
            match value.format() {
                PgValueFormat::Binary => Self::try_from(value.as_bytes()?),
                PgValueFormat::Text => value.as_str()?.parse(),
            }
            .map_err(Into::into)
        }
    }
}

#[cfg(feature = "sqlx-mysql")]
mod mysql {
    use crate::{Id, Type};
    use sqlx::mysql::{MySql, MySqlTypeInfo, MySqlValueRef};
    use sqlx::{encode::IsNull, error::BoxDynError, Decode, Encode};

    impl<T: Type> sqlx::Type<MySql> for Id<T> {
        fn type_info() -> MySqlTypeInfo {
            <&[u8] as sqlx::Type<MySql>>::type_info()
        }

        fn compatible(ty: &MySqlTypeInfo) -> bool {
            <&[u8] as sqlx::Type<MySql>>::compatible(ty)
        }
    }

    impl<T: Type> Encode<'_, MySql> for Id<T> {
        fn encode_by_ref(&self, buf: &mut Vec<u8>) -> Result<IsNull, BoxDynError> {
            <&[u8] as Encode<'_, MySql>>::encode(self.as_bytes(), buf)
        }
    }

    impl<T: Type> Decode<'_, MySql> for Id<T> {
        fn decode(value: MySqlValueRef) -> Result<Self, BoxDynError> {
            let bytes = <&[u8] as Decode<MySql>>::decode(value)?;
            Self::try_from(bytes).map_err(Into::into)
        }
    }
}

#[cfg(feature = "sqlx-sqlite")]
mod sqlite {
    use crate::{Id, Type};
    use sqlx::sqlite::{Sqlite, SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef};
    use sqlx::{encode::IsNull, error::BoxDynError, Decode, Encode};
    use std::borrow::Cow;

    impl<T: Type> sqlx::Type<Sqlite> for Id<T> {
        fn type_info() -> SqliteTypeInfo {
            <&str as sqlx::Type<Sqlite>>::type_info()
        }

        fn compatible(ty: &SqliteTypeInfo) -> bool {
            <&str as sqlx::Type<Sqlite>>::compatible(ty)
        }
    }

    impl<'q, T: Type> Encode<'q, Sqlite> for Id<T> {
        fn encode_by_ref(
            &self,
            args: &mut Vec<SqliteArgumentValue<'q>>,
        ) -> Result<IsNull, BoxDynError> {
            args.push(SqliteArgumentValue::Text(Cow::Owned(self.to_string())));
            Ok(IsNull::No)
        }
    }

    impl<T: Type> Decode<'_, Sqlite> for Id<T> {
        fn decode(value: SqliteValueRef<'_>) -> Result<Self, BoxDynError> {
            <&str as Decode<Sqlite>>::decode(value)?
                .parse()
                .map_err(Into::into)
        }
    }
}
