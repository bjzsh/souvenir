use sqlx::Type;
#[cfg(feature = "mysql")]
use sqlx::mysql::{MySql, MySqlTypeInfo};
#[cfg(feature = "postgres")]
use sqlx::postgres::{PgTypeInfo, PgTypeKind, Postgres, types::Oid};

/// Postgres `uuid`.
#[cfg(feature = "postgres")]
pub(crate) const PG_UUID: Oid = Oid(2950);

/// Postgres `uuid[]`.
#[cfg(feature = "postgres")]
pub(crate) const PG_UUID_ARRAY: Oid = Oid(2951);

/// Postgres text types, including domains over them (such as a `souvenir`
/// domain).
#[cfg(feature = "postgres")]
pub(crate) fn pg_text(ty: &PgTypeInfo) -> bool {
    <&str as Type<Postgres>>::compatible(ty)
        || matches!(ty.kind(), PgTypeKind::Domain(base) if pg_text(base))
}

/// Postgres `bytea`, including domains over it.
#[cfg(feature = "postgres")]
pub(crate) fn pg_bytea(ty: &PgTypeInfo) -> bool {
    <Vec<u8> as Type<Postgres>>::compatible(ty)
        || matches!(ty.kind(), PgTypeKind::Domain(base) if pg_bytea(base))
}

/// Postgres `uuid`, including domains over it.
#[cfg(feature = "postgres")]
pub(crate) fn pg_uuid(ty: &PgTypeInfo) -> bool {
    *ty == PgTypeInfo::with_oid(PG_UUID)
        || matches!(ty.kind(), PgTypeKind::Domain(base) if pg_uuid(base))
}

/// MySQL string columns, binary or not. MySQL flags text columns with a
/// binary collation (such as `ascii_bin`, the recommended one) as binary, so
/// the flag can't tell the forms apart; the Rust type decides.
#[cfg(feature = "mysql")]
pub(crate) fn mysql_string(ty: &MySqlTypeInfo) -> bool {
    <&str as Type<MySql>>::compatible(ty) || <&[u8] as Type<MySql>>::compatible(ty)
}
