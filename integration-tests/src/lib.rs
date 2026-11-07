#![forbid(unsafe_code)]

//! Shared fixtures for the database integration tests in `tests/`.
//!
//! Postgres and MySQL tests run only when `SOUVENIR_TEST_PG_URL` /
//! `SOUVENIR_TEST_MYSQL_URL` are set; SQLite tests always run in memory.
//!
//! With diesel, a column whose SQL type contradicts the prefix's declared
//! storage fails to build (`Event` is stored as bytes, not text):
//!
//! ```compile_fail
//! use diesel::prelude::*;
//! use souvenir::Id;
//! use souvenir_integration_tests::Event;
//!
//! diesel::table! { probe (k) { k -> Integer, id -> Text, } }
//!
//! let mut conn = SqliteConnection::establish(":memory:").unwrap();
//! let _: Vec<Id<Event>> = probe::table.select(probe::id).load(&mut conn).unwrap();
//! ```
//!
//! The same code with a `Binary` column builds:
//!
//! ```
//! use diesel::prelude::*;
//! use souvenir::Id;
//! use souvenir_integration_tests::Event;
//!
//! diesel::table! { probe (k) { k -> Integer, id -> Binary, } }
//!
//! let mut conn = SqliteConnection::establish(":memory:").unwrap();
//! diesel::sql_query("CREATE TABLE probe (k INTEGER PRIMARY KEY, id BLOB)").execute(&mut conn).unwrap();
//! let ids: Vec<Id<Event>> = probe::table.select(probe::id).load(&mut conn).unwrap();
//! assert!(ids.is_empty());
//! ```

use souvenir::{Id, Prefix, Souvenir, tagged};

tagged! {
    /// Stored as text (the default).
    pub User = "user";
    /// Stored as 20 bytes.
    pub Event = "event", storage = Bytes;
    /// Stored as a UUIDv7.
    pub Account = "acct", storage = Uuid;
}

/// Prefixes chosen so that some are prefixes of others.
pub const PREFIXES: [&str; 7] = ["u", "us", "user", "users", "org", "a", "zzzzzzzz"];

/// One row of fixture IDs, one per storage form plus an untyped ID.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fixture {
    pub k: i32,
    pub any: Souvenir,
    pub user: Id<User>,
    pub event: Id<Event>,
    pub account: Id<Account>,
}

/// Deterministic IDs whose timestamps are not in insertion order.
pub fn fixtures() -> Vec<Fixture> {
    (0..60)
        .map(|i| {
            let prefix = PREFIXES[i as usize % PREFIXES.len()];
            let t = 1_793_531_580_000 + ((i * 7919) % 61) as u64;
            let random = (i as u128).wrapping_mul(0x9E37_79B9_7F4A_7C15_F39C_C060_5CED_C834)
                & Souvenir::RANDOM_MAX;
            Fixture {
                k: i,
                any: Souvenir::from_parts(Prefix::parse(prefix).unwrap(), t, (i as u128) << 40)
                    .unwrap(),
                user: Id::from_parts(t, random).unwrap(),
                event: Id::from_parts(t + 1, random ^ 1).unwrap(),
                account: Id::from_parts(t + 2, random ^ 2).unwrap(),
            }
        })
        .collect()
}

/// The fixture whose untyped ID has prefix `org`, for mismatch tests.
pub fn org_row() -> Fixture {
    fixtures()
        .into_iter()
        .find(|f| f.any.prefix() == "org")
        .unwrap()
}

/// The environment variable value, or `None` with a note that the test is
/// skipped.
pub fn url(var: &str) -> Option<String> {
    let url = std::env::var(var).ok().filter(|v| !v.is_empty());
    if url.is_none() {
        eprintln!("{var} is not set; skipping");
    }
    url
}

/// Check that `got` is `items` sorted.
pub fn assert_sorted<T: Ord + Clone + std::fmt::Debug>(
    items: impl IntoIterator<Item = T>,
    got: &[T],
) {
    let mut sorted: Vec<T> = items.into_iter().collect();
    sorted.sort();
    assert_eq!(got, sorted.as_slice());
}

/// Run `sql` and decode its single value as `T`.
pub async fn one<DB, T>(pool: &sqlx::Pool<DB>, sql: &str) -> Result<T, sqlx::Error>
where
    DB: sqlx::Database,
    for<'c> &'c mut DB::Connection: sqlx::Executor<'c, Database = DB>,
    for<'q> DB::Arguments<'q>: sqlx::IntoArguments<'q, DB>,
    (T,): for<'r> sqlx::FromRow<'r, DB::Row>,
    T: Send + Unpin,
{
    sqlx::query_scalar(sql).fetch_one(pool).await
}

/// Check that `SELECT column FROM table ORDER BY column` returns `expected`
/// sorted, i.e. that the database orders the column the way Rust does.
pub async fn assert_db_order<DB, T>(
    pool: &sqlx::Pool<DB>,
    table: &str,
    column: &str,
    expected: impl IntoIterator<Item = T>,
) where
    DB: sqlx::Database,
    for<'c> &'c mut DB::Connection: sqlx::Executor<'c, Database = DB>,
    for<'q> DB::Arguments<'q>: sqlx::IntoArguments<'q, DB>,
    (T,): for<'r> sqlx::FromRow<'r, DB::Row>,
    T: Ord + Clone + std::fmt::Debug + Send + Unpin,
{
    let sql = format!("SELECT {column} FROM {table} ORDER BY {column}");
    let got: Vec<T> = sqlx::query_scalar(&sql).fetch_all(pool).await.unwrap();
    assert_sorted(expected, &got);
}

/// Check that a result failed with `expected`, a Souvenir error, which
/// database errors wrap: they contain its message.
#[track_caller]
pub fn assert_err<T: std::fmt::Debug, E: std::fmt::Display>(
    result: Result<T, E>,
    expected: souvenir::Error,
) {
    assert_db_err(result, expected.message());
}

/// Check that a result failed with an error mentioning `text`, such as a
/// database's own constraint error.
#[track_caller]
pub fn assert_db_err<T: std::fmt::Debug, E: std::fmt::Display>(result: Result<T, E>, text: &str) {
    match result {
        Ok(v) => panic!("expected an error mentioning {text:?}, got Ok({v:?})"),
        Err(e) => assert!(e.to_string().contains(text), "expected {text:?}, got: {e}"),
    }
}
/// The recommended Postgres domain for text IDs (see the README).
pub const PG_DOMAIN: &str = r#"
DO $$ BEGIN
    CREATE DOMAIN souvenir AS text COLLATE "C"
        CHECK (VALUE ~ '^[a-z]{1,8}_[0-3][0-9a-hjkmnp-tv-z]{24}$');
EXCEPTION WHEN duplicate_object OR unique_violation THEN NULL;
END $$"#;

/// The recommended MySQL column type for text IDs (see the README).
pub const MYSQL_TEXT: &str = "VARCHAR(34) CHARACTER SET ascii COLLATE ascii_bin";
