#[cfg(feature = "postgres")]
mod pg {
    use crate::id::Id;
    use sqlx::postgres::{
        PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef, Postgres,
        types::Oid,
    };
    use sqlx::{Decode, Encode, Type, encode::IsNull, error::BoxDynError};

    impl Type<Postgres> for Id {
        fn type_info() -> PgTypeInfo {
            PgTypeInfo::with_oid(Oid(2950))
        }
    }

    impl PgHasArrayType for Id {
        fn array_type_info() -> PgTypeInfo {
            PgTypeInfo::with_oid(Oid(2951))
        }
    }

    impl Encode<'_, Postgres> for Id {
        fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
            buf.extend_from_slice(self.as_bytes());
            Ok(IsNull::No)
        }
    }

    impl Decode<'_, Postgres> for Id {
        fn decode(value: PgValueRef) -> Result<Self, BoxDynError> {
            match value.format() {
                PgValueFormat::Binary => Self::try_from(value.as_bytes()?),
                PgValueFormat::Text => value.as_str()?.parse(),
            }
            .map_err(Into::into)
        }
    }
}

#[cfg(feature = "mysql")]
mod mysql {
    use crate::id::Id;
    use sqlx::mysql::{MySql, MySqlTypeInfo, MySqlValueRef};
    use sqlx::{Decode, Encode, Type, encode::IsNull, error::BoxDynError};

    impl Type<MySql> for Id {
        fn type_info() -> MySqlTypeInfo {
            <&[u8] as sqlx::Type<MySql>>::type_info()
        }

        fn compatible(ty: &MySqlTypeInfo) -> bool {
            <&[u8] as sqlx::Type<MySql>>::compatible(ty)
        }
    }

    impl Encode<'_, MySql> for Id {
        fn encode_by_ref(&self, buf: &mut Vec<u8>) -> Result<IsNull, BoxDynError> {
            <&[u8] as Encode<'_, MySql>>::encode(self.as_bytes(), buf)
        }
    }

    impl Decode<'_, MySql> for Id {
        fn decode(value: MySqlValueRef) -> Result<Self, BoxDynError> {
            let bytes = <&[u8] as Decode<MySql>>::decode(value)?;
            Self::try_from(bytes).map_err(Into::into)
        }
    }
}

#[cfg(feature = "sqlite")]
mod sqlite {
    use crate::id::Id;
    use sqlx::sqlite::{Sqlite, SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef};
    use sqlx::{Decode, Encode, Type, encode::IsNull, error::BoxDynError};
    use std::borrow::Cow;

    impl Type<Sqlite> for Id {
        fn type_info() -> SqliteTypeInfo {
            <&str as sqlx::Type<Sqlite>>::type_info()
        }

        fn compatible(ty: &SqliteTypeInfo) -> bool {
            <&str as sqlx::Type<Sqlite>>::compatible(ty)
        }
    }

    impl<'q> Encode<'q, Sqlite> for Id {
        fn encode_by_ref(
            &self,
            args: &mut Vec<SqliteArgumentValue<'q>>,
        ) -> Result<IsNull, BoxDynError> {
            args.push(SqliteArgumentValue::Text(Cow::Owned(self.to_string())));
            Ok(IsNull::No)
        }
    }

    impl Decode<'_, Sqlite> for Id {
        fn decode(value: SqliteValueRef<'_>) -> Result<Self, BoxDynError> {
            <&str as Decode<Sqlite>>::decode(value)?
                .parse()
                .map_err(Into::into)
        }
    }
}
