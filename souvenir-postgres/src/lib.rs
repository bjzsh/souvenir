//! `souvenir`: a native Postgres type for Souvenir IDs.
//!
//! Values are stored as the 20-byte binary form (SPEC.md section 3): fixed
//! length, no varlena header, compared byte-wise. They print and parse as the
//! canonical text form, and the binary wire format (`send`/`recv`) is the same
//! 20 bytes, so clients can use either.
//!
//! The type, its operators and its operator classes are declared by hand in
//! SQL (see `sql/`), because pgrx's derived types are variable-length.
//! Everything else is a plain `#[pg_extern]`.

use core::ffi::CStr;
use pgrx::callconv::{Arg, ArgAbi, BoxRet, FcInfo};
use pgrx::datum::{Datum, TimestampWithTimeZone, Uuid};
use pgrx::pg_sys;
use pgrx::pgrx_sql_entity_graph::metadata::{
    ArgumentError, ReturnsError, ReturnsRef, SqlMappingRef, SqlTranslatable, TypeOrigin,
};
use pgrx::prelude::*;
use souvenir_core::{Prefix, Souvenir};
use std::ffi::CString;

pgrx::pg_module_magic!();

const BYTE_LEN: usize = Souvenir::BYTE_LEN;

/// A `souvenir` value as a function argument or result.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct PgSouvenir(pub Souvenir);

// ---------------------------------------------------------------------------
// Datum plumbing: a pass-by-reference, fixed-length (20 byte) type.
// ---------------------------------------------------------------------------

impl FromDatum for PgSouvenir {
    unsafe fn from_polymorphic_datum(
        datum: pg_sys::Datum,
        is_null: bool,
        _typoid: pg_sys::Oid,
    ) -> Option<Self> {
        if is_null {
            return None;
        }
        // SAFETY: a non-null `souvenir` datum points to its 20 bytes
        // (`INTERNALLENGTH = 20`, never toasted: `STORAGE = plain`).
        let bytes = unsafe { *datum.cast_mut_ptr::<[u8; BYTE_LEN]>() };
        // Only `souvenir_in` and `souvenir_recv` create values, and both
        // validate, so this can only fail on corrupted storage.
        match Souvenir::from_bytes(bytes) {
            Ok(id) => Some(Self(id)),
            Err(e) => raise(
                PgSqlErrorCode::ERRCODE_DATA_CORRUPTED,
                format!("corrupted souvenir value: {e}"),
            ),
        }
    }
}

impl IntoDatum for PgSouvenir {
    fn into_datum(self) -> Option<pg_sys::Datum> {
        // SAFETY: palloc returns BYTE_LEN writable bytes in the current memory
        // context, which Postgres owns from here on. Alignment is `char`.
        unsafe {
            let ptr = pg_sys::palloc(BYTE_LEN).cast::<[u8; BYTE_LEN]>();
            ptr.write(self.0.to_bytes());
            Some(pg_sys::Datum::from(ptr))
        }
    }

    fn type_oid() -> pg_sys::Oid {
        pgrx::wrappers::regtypein("souvenir")
    }
}

// SAFETY: arguments of SQL type `souvenir` are unboxed with `FromDatum`
// above, which reads exactly the type's 20 bytes.
unsafe impl<'fcx> ArgAbi<'fcx> for PgSouvenir {
    unsafe fn unbox_arg_unchecked(arg: Arg<'_, 'fcx>) -> Self {
        let index = arg.index();
        unsafe { arg.unbox_arg_using_from_datum() }
            .unwrap_or_else(|| panic!("argument {index} must not be null"))
    }
}

// SAFETY: results are boxed with `IntoDatum` above.
unsafe impl BoxRet for PgSouvenir {
    unsafe fn box_into<'fcx>(self, fcinfo: &mut FcInfo<'fcx>) -> Datum<'fcx> {
        unsafe { fcinfo.return_optional_datum(self.into_datum()) }
    }
}

// SAFETY: the SQL type `souvenir` is exactly the representation above.
unsafe impl SqlTranslatable for PgSouvenir {
    const TYPE_IDENT: &'static str = pgrx::pgrx_resolved_type!(PgSouvenir);
    const TYPE_ORIGIN: TypeOrigin = TypeOrigin::External;
    const ARGUMENT_SQL: Result<SqlMappingRef, ArgumentError> =
        Ok(SqlMappingRef::literal("souvenir"));
    const RETURN_SQL: Result<ReturnsRef, ReturnsError> =
        Ok(ReturnsRef::One(SqlMappingRef::literal("souvenir")));
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

fn raise(code: PgSqlErrorCode, message: String) -> ! {
    ereport!(ERROR, code, message);
}

fn text_error(e: souvenir_core::Error, input: &str) -> ! {
    raise(
        PgSqlErrorCode::ERRCODE_INVALID_TEXT_REPRESENTATION,
        format!("invalid souvenir {input:?}: {e}"),
    )
}

fn parse_prefix(text: &str) -> Prefix {
    Prefix::parse(text).unwrap_or_else(|_| {
        raise(
            PgSqlErrorCode::ERRCODE_INVALID_PARAMETER_VALUE,
            format!("invalid souvenir prefix {text:?}: must be 1-8 characters of a-z"),
        )
    })
}

fn out_of_range(what: &str) -> ! {
    raise(
        PgSqlErrorCode::ERRCODE_DATETIME_FIELD_OVERFLOW,
        format!("{what} is outside the range a souvenir can hold"),
    )
}

// ---------------------------------------------------------------------------
// I/O functions (referenced by CREATE TYPE in sql/type.sql)
// ---------------------------------------------------------------------------

#[pg_extern(immutable, strict, parallel_safe, requires = ["shell_type"])]
fn souvenir_in(input: &CStr) -> PgSouvenir {
    let text = input
        .to_str()
        .unwrap_or_else(|_| text_error(souvenir_core::Error::InvalidCharacter, "<non-UTF-8>"));
    match Souvenir::parse(text) {
        Ok(id) => PgSouvenir(id),
        Err(e) => text_error(e, text),
    }
}

#[pg_extern(immutable, strict, parallel_safe, requires = ["shell_type"])]
fn souvenir_out(id: PgSouvenir) -> CString {
    let mut buf = [0; Souvenir::MAX_TEXT_LEN];
    // The text form is ASCII without NULs.
    CString::new(id.0.encode(&mut buf)).unwrap_or_default()
}

#[pg_extern(immutable, strict, parallel_safe, requires = ["shell_type"])]
fn souvenir_recv(buf: pgrx::datum::Internal) -> PgSouvenir {
    // SAFETY: Postgres passes receive functions a StringInfo holding the
    // message; pq_getmsgbytes checks the length and raises on short input.
    let bytes = unsafe {
        let msg = buf
            .unwrap()
            .unwrap_or_else(|| panic!("no message"))
            .cast_mut_ptr();
        let ptr = pg_sys::pq_getmsgbytes(msg, BYTE_LEN as i32).cast::<[u8; BYTE_LEN]>();
        *ptr
    };
    match Souvenir::from_bytes(bytes) {
        Ok(id) => PgSouvenir(id),
        Err(e) => raise(
            PgSqlErrorCode::ERRCODE_INVALID_BINARY_REPRESENTATION,
            format!("invalid binary souvenir: {e}"),
        ),
    }
}

#[pg_extern(immutable, strict, parallel_safe, requires = ["shell_type"])]
fn souvenir_send(id: PgSouvenir) -> Vec<u8> {
    id.0.to_bytes().to_vec()
}

// ---------------------------------------------------------------------------
// Comparison and hashing (operators and opclasses in sql/operators.sql)
// ---------------------------------------------------------------------------

#[pg_extern(immutable, strict, parallel_safe, requires = ["shell_type"])]
fn souvenir_eq(a: PgSouvenir, b: PgSouvenir) -> bool {
    a == b
}

#[pg_extern(immutable, strict, parallel_safe, requires = ["shell_type"])]
fn souvenir_ne(a: PgSouvenir, b: PgSouvenir) -> bool {
    a != b
}

#[pg_extern(immutable, strict, parallel_safe, requires = ["shell_type"])]
fn souvenir_lt(a: PgSouvenir, b: PgSouvenir) -> bool {
    a < b
}

#[pg_extern(immutable, strict, parallel_safe, requires = ["shell_type"])]
fn souvenir_le(a: PgSouvenir, b: PgSouvenir) -> bool {
    a <= b
}

#[pg_extern(immutable, strict, parallel_safe, requires = ["shell_type"])]
fn souvenir_gt(a: PgSouvenir, b: PgSouvenir) -> bool {
    a > b
}

#[pg_extern(immutable, strict, parallel_safe, requires = ["shell_type"])]
fn souvenir_ge(a: PgSouvenir, b: PgSouvenir) -> bool {
    a >= b
}

#[pg_extern(immutable, strict, parallel_safe, requires = ["shell_type"])]
fn souvenir_cmp(a: PgSouvenir, b: PgSouvenir) -> i32 {
    a.cmp(&b) as i32
}

#[pg_extern(immutable, strict, parallel_safe, requires = ["shell_type"])]
fn souvenir_hash(id: PgSouvenir) -> i32 {
    let bytes = id.0.to_bytes();
    // SAFETY: hash_bytes reads exactly `keylen` bytes from `k`.
    unsafe { pg_sys::hash_bytes(bytes.as_ptr(), BYTE_LEN as i32) as i32 }
}

#[pg_extern(immutable, strict, parallel_safe, requires = ["shell_type"])]
fn souvenir_hash_extended(id: PgSouvenir, seed: i64) -> i64 {
    let bytes = id.0.to_bytes();
    // SAFETY: as above.
    unsafe { pg_sys::hash_bytes_extended(bytes.as_ptr(), BYTE_LEN as i32, seed as u64) as i64 }
}

// ---------------------------------------------------------------------------
// Conversions
// ---------------------------------------------------------------------------

/// The 20-byte binary form.
#[pg_extern(immutable, strict, parallel_safe, requires = ["shell_type"])]
fn souvenir_to_bytea(id: PgSouvenir) -> Vec<u8> {
    id.0.to_bytes().to_vec()
}

/// Decode the 20-byte binary form.
#[pg_extern(immutable, strict, parallel_safe, requires = ["shell_type"])]
fn souvenir_from_bytea(bytes: &[u8]) -> PgSouvenir {
    match Souvenir::from_slice(bytes) {
        Ok(id) => PgSouvenir(id),
        Err(e) => raise(
            PgSqlErrorCode::ERRCODE_INVALID_BINARY_REPRESENTATION,
            format!("invalid binary souvenir: {e}"),
        ),
    }
}

/// The ID as a UUIDv7. The prefix is not part of the UUID.
#[pg_extern(immutable, strict, parallel_safe, requires = ["shell_type"])]
fn souvenir_to_uuid(id: PgSouvenir) -> Uuid {
    Uuid::from_bytes(*id.0.to_uuid().as_bytes())
}

/// The ID with this prefix whose timestamp and random bits are in `uuid`, a
/// UUIDv7 made by `souvenir_to_uuid`.
#[pg_extern(immutable, strict, parallel_safe, requires = ["shell_type"])]
fn souvenir_from_uuid(prefix: &str, uuid: Uuid) -> PgSouvenir {
    let uuid = souvenir_core::uuid::Uuid::from_bytes(*uuid.as_bytes());
    match Souvenir::from_uuid(parse_prefix(prefix), uuid) {
        Ok(id) => PgSouvenir(id),
        Err(e) => raise(
            PgSqlErrorCode::ERRCODE_INVALID_PARAMETER_VALUE,
            format!("invalid souvenir UUID: {e}"),
        ),
    }
}

// ---------------------------------------------------------------------------
// Generation and inspection
// ---------------------------------------------------------------------------

/// A new ID with this prefix, from the current time and the OS random source.
#[pg_extern(volatile, strict, parallel_safe, requires = ["shell_type"])]
fn gen_souvenir(prefix: &str) -> PgSouvenir {
    match Souvenir::new(parse_prefix(prefix)) {
        Ok(id) => PgSouvenir(id),
        Err(e) => raise(
            PgSqlErrorCode::ERRCODE_SYSTEM_ERROR,
            format!("could not generate a souvenir: {e}"),
        ),
    }
}

/// The prefix, such as `user`.
#[pg_extern(immutable, strict, parallel_safe, requires = ["shell_type"])]
fn souvenir_prefix(id: PgSouvenir) -> String {
    id.0.prefix().as_str().to_owned()
}

/// Whether the ID has this prefix.
#[pg_extern(immutable, strict, parallel_safe, requires = ["shell_type"])]
fn souvenir_has_prefix(id: PgSouvenir, prefix: &str) -> bool {
    id.0.prefix() == parse_prefix(prefix)
}

/// When the ID was created.
#[pg_extern(immutable, strict, parallel_safe, requires = ["shell_type"])]
fn souvenir_timestamp(id: PgSouvenir) -> TimestampWithTimeZone {
    let pg_micros = unix_ms_to_pg_micros(id.0.timestamp_ms());
    TimestampWithTimeZone::try_from(pg_micros).unwrap_or_else(|_| out_of_range("timestamp"))
}

/// Whether `text` is a valid ID.
#[pg_extern(immutable, strict, parallel_safe)]
fn souvenir_is_valid(text: &str) -> bool {
    Souvenir::is_valid(text)
}

/// The smallest ID with this prefix: `id >= souvenir_min('user')` with
/// `id <= souvenir_max('user')` selects every user ID with an index range
/// scan.
#[pg_extern(immutable, strict, parallel_safe, requires = ["shell_type"], name = "souvenir_min")]
fn souvenir_min(prefix: &str) -> PgSouvenir {
    bound(prefix, 0, 0)
}

/// The largest ID with this prefix.
#[pg_extern(immutable, strict, parallel_safe, requires = ["shell_type"], name = "souvenir_max")]
fn souvenir_max(prefix: &str) -> PgSouvenir {
    bound(prefix, Souvenir::TIMESTAMP_MAX, Souvenir::RANDOM_MAX)
}

/// The smallest ID with this prefix created at or after `at`.
#[pg_extern(immutable, strict, parallel_safe, requires = ["shell_type"], name = "souvenir_min")]
fn souvenir_min_at(prefix: &str, at: TimestampWithTimeZone) -> PgSouvenir {
    bound(prefix, timestamp_ms(at, Rounding::Up), 0)
}

/// The largest ID with this prefix created at or before `at`.
#[pg_extern(immutable, strict, parallel_safe, requires = ["shell_type"], name = "souvenir_max")]
fn souvenir_max_at(prefix: &str, at: TimestampWithTimeZone) -> PgSouvenir {
    bound(
        prefix,
        timestamp_ms(at, Rounding::Down),
        Souvenir::RANDOM_MAX,
    )
}

fn bound(prefix: &str, timestamp_ms: u64, random: u128) -> PgSouvenir {
    match Souvenir::from_parts(parse_prefix(prefix), timestamp_ms, random) {
        Ok(id) => PgSouvenir(id),
        Err(_) => out_of_range("timestamp"),
    }
}

// ---------------------------------------------------------------------------
// Time conversions. Postgres counts microseconds from 2000-01-01; IDs count
// milliseconds from 1970-01-01. Bounds clamp to the representable range, so
// `souvenir_min('user', '-infinity')` is `souvenir_min('user')`.
// ---------------------------------------------------------------------------

/// Microseconds between the Unix and Postgres epochs.
const PG_EPOCH_OFFSET_MICROS: i64 = 946_684_800_000_000;

fn unix_ms_to_pg_micros(ms: u64) -> i64 {
    // ms < 2^48, so neither step overflows.
    ms as i64 * 1000 - PG_EPOCH_OFFSET_MICROS
}

#[derive(Clone, Copy)]
enum Rounding {
    Up,
    Down,
}

fn timestamp_ms(at: TimestampWithTimeZone, rounding: Rounding) -> u64 {
    let pg_micros: i64 = at.into();
    let unix_micros = i128::from(pg_micros) + i128::from(PG_EPOCH_OFFSET_MICROS);
    let ms = match rounding {
        Rounding::Up => {
            unix_micros.div_euclid(1000) + i128::from(unix_micros.rem_euclid(1000) != 0)
        }
        Rounding::Down => unix_micros.div_euclid(1000),
    };
    ms.clamp(0, i128::from(Souvenir::TIMESTAMP_MAX)) as u64
}

// ---------------------------------------------------------------------------
// SQL: the type itself, then operators, opclasses and casts.
// ---------------------------------------------------------------------------

extension_sql!(
    "CREATE TYPE souvenir; -- shell, completed in sql/type.sql\n",
    name = "shell_type",
    bootstrap
);

extension_sql_file!("../sql/type.sql", name = "type", finalize);

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    include!("tests.rs");
}

/// Required by `cargo pgrx test`.
#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {}

    #[must_use]
    pub fn postgresql_conf_options() -> Vec<&'static str> {
        vec![]
    }
}
