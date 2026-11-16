// SQL-level tests, run inside Postgres by `cargo pgrx test` (included into
// `mod tests` in lib.rs). Parsing and encoding come from the `souvenir`
// crate and are covered by its conformance tests; these check the type's
// SQL behavior.

use pgrx::prelude::*;

const USER: &str = "user_00t35pg1sg9ech32tymrkdq6v";
/// The same ID's 20 bytes.
const USER_HEX: &str = "\\xa107104cc406865b4039825cc88c5af53136dcdb";

fn one<T: IntoDatum + FromDatum>(sql: &str) -> Option<T> {
    Spi::get_one::<T>(sql).unwrap_or_else(|e| panic!("{sql}: {e}"))
}

fn text(sql: &str) -> String {
    one::<String>(sql).unwrap_or_else(|| panic!("{sql} returned NULL"))
}

/// The query plan, one line per row of EXPLAIN output.
fn plan(sql: &str) -> String {
    Spi::connect(|client| {
        let rows = client.select(&format!("EXPLAIN (COSTS OFF) {sql}"), None, &[]).unwrap();
        rows.map(|row| row.get::<String>(1).unwrap().unwrap_or_default())
            .collect::<Vec<_>>()
            .join("\n")
    })
}

fn flag(sql: &str) -> bool {
    one::<bool>(sql).unwrap_or_else(|| panic!("{sql} returned NULL"))
}

#[pg_test]
fn text_round_trip() {
    assert_eq!(text(&format!("SELECT '{USER}'::souvenir::text")), USER);
}

#[pg_test]
fn stored_in_20_bytes() {
    assert_eq!(one::<i32>(&format!("SELECT pg_column_size('{USER}'::souvenir)")), Some(20));
    assert_eq!(
        one::<i16>("SELECT typlen FROM pg_type WHERE typname = 'souvenir'"),
        Some(20)
    );
}

#[pg_test]
fn binary_form_matches_spec() {
    assert_eq!(
        text(&format!("SELECT souvenir_send('{USER}')::text")),
        USER_HEX
    );
    assert_eq!(text(&format!("SELECT ('{USER}'::souvenir)::bytea::text")), USER_HEX);
    assert_eq!(text(&format!("SELECT '{USER_HEX}'::bytea::souvenir::text")), USER);
}

#[pg_test(error = "invalid souvenir \"User_00t35pg1sg9ech32tymrkdq6v\": prefix must be 1-8 characters of a-z")]
fn rejects_invalid_text() {
    one::<String>("SELECT 'User_00t35pg1sg9ech32tymrkdq6v'::souvenir::text");
}

#[pg_test(error = "invalid binary souvenir: input has the wrong length")]
fn rejects_short_bytea() {
    one::<String>("SELECT '\\x00'::bytea::souvenir::text");
}

#[pg_test(error = "invalid binary souvenir: prefix field is zero, which is not any prefix")]
fn rejects_zero_bytes() {
    one::<String>("SELECT decode(repeat('00', 20), 'hex')::souvenir::text");
}

#[pg_test]
fn is_valid() {
    assert!(flag(&format!("SELECT souvenir_is_valid('{USER}')")));
    assert!(!flag("SELECT souvenir_is_valid('user_')"));
}

#[pg_test]
fn orders_by_prefix_then_time() {
    let sorted = text(
        "SELECT string_agg(souvenir_prefix(id), ',' ORDER BY id) FROM (VALUES
            (souvenir_min('users')), (souvenir_max('us')), (souvenir_min('user')),
            (souvenir_max('user')), (souvenir_min('u')), (souvenir_min('v'))
         ) AS t(id)",
    );
    assert_eq!(sorted, "u,us,user,user,users,v");
    assert!(flag(
        "SELECT souvenir_min('user', '2026-01-01') < souvenir_min('user', '2026-01-02')"
    ));
}

#[pg_test]
fn comparison_operators() {
    let a = "souvenir_min('a')";
    let b = "souvenir_min('b')";
    for (op, expected) in [("=", false), ("<>", true), ("<", true), ("<=", true), (">", false), (">=", false)] {
        assert_eq!(flag(&format!("SELECT {a} {op} {b}")), expected, "{op}");
    }
    assert!(flag(&format!("SELECT '{USER}'::souvenir = '{USER}'::souvenir")));
}

#[pg_test]
fn btree_index_range_scan() {
    Spi::run(
        "CREATE TABLE ids (id souvenir PRIMARY KEY);
         INSERT INTO ids SELECT gen_souvenir(p) FROM unnest(ARRAY['org','user','users']) p,
             generate_series(1, 200);
         ANALYZE ids;
         SET enable_seqscan = off;",
    )
    .unwrap();
    let count = one::<i64>(
        "SELECT count(*) FROM ids WHERE id BETWEEN souvenir_min('user') AND souvenir_max('user')",
    );
    assert_eq!(count, Some(200));
    let plan = plan(
        "SELECT * FROM ids WHERE id BETWEEN souvenir_min('user') AND souvenir_max('user')",
    );
    assert!(plan.contains("Index Cond") && plan.contains("ids_pkey"), "{plan}");
}

#[pg_test]
fn hash_join_and_distinct() {
    Spi::run(
        "CREATE TABLE a (id souvenir); CREATE TABLE b (id souvenir);
         INSERT INTO a SELECT gen_souvenir('user') FROM generate_series(1, 100);
         INSERT INTO b SELECT id FROM a; INSERT INTO b SELECT id FROM a;
         CREATE INDEX ON a USING hash (id);
         SET enable_mergejoin = off; SET enable_nestloop = off;",
    )
    .unwrap();
    assert_eq!(one::<i64>("SELECT count(*) FROM a JOIN b USING (id)"), Some(200));
    assert_eq!(one::<i64>("SELECT count(DISTINCT id) FROM b"), Some(100));
    let plan = plan("SELECT * FROM a JOIN b USING (id)");
    assert!(plan.contains("Hash Join"), "{plan}");
}

#[pg_test]
fn generated_ids() {
    assert_eq!(text("SELECT souvenir_prefix(gen_souvenir('event'))"), "event");
    assert!(flag("SELECT gen_souvenir('user') <> gen_souvenir('user')"));
    assert!(flag(
        "SELECT abs(extract(epoch FROM souvenir_timestamp(gen_souvenir('user')) - now())) < 60"
    ));
}

#[pg_test(error = "invalid souvenir prefix \"User\": must be 1-8 characters of a-z")]
fn rejects_invalid_prefix() {
    one::<String>("SELECT gen_souvenir('User')::text");
}

#[pg_test]
fn default_values() {
    Spi::run(
        "CREATE TABLE users (id souvenir PRIMARY KEY DEFAULT gen_souvenir('user'), name text);
         INSERT INTO users (name) VALUES ('ada');",
    )
    .unwrap();
    assert!(flag("SELECT souvenir_has_prefix(id, 'user') FROM users"));
    assert!(!flag("SELECT souvenir_has_prefix(id, 'org') FROM users"));
}

#[pg_test]
fn timestamp() {
    // 1_793_531_580_000 ms is 2026-11-01 11:13:00 UTC.
    assert_eq!(
        text(&format!(
            "SELECT to_char(souvenir_timestamp('{USER}') AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI:SS.MS')"
        )),
        "2026-11-01 11:13:00.000"
    );
}

#[pg_test]
fn time_bounds_round_inward() {
    // Sub-millisecond bounds round so the range never includes IDs outside it.
    assert_eq!(
        text("SELECT souvenir_min('user', '1970-01-01 00:00:00.0005+00')::text"),
        "user_0000000000g00000000000000"
    );
    assert!(flag(
        "SELECT souvenir_max('user', '1970-01-01 00:00:00.0015+00') = souvenir_max('user', '1970-01-01 00:00:00.001+00')"
    ));
    // Infinite bounds clamp to the representable range.
    assert!(flag("SELECT souvenir_min('user', '-infinity') = souvenir_min('user')"));
    assert!(flag("SELECT souvenir_max('user', 'infinity') = souvenir_max('user')"));
}

#[pg_test]
fn uuid_round_trip() {
    let uuid = text(&format!("SELECT souvenir_to_uuid('{USER}')::text"));
    assert_eq!(&uuid[14..15], "7", "version 7: {uuid}");
    assert_eq!(text(&format!("SELECT souvenir_from_uuid('user', '{uuid}')::text")), USER);
}

#[pg_test]
fn assignment_casts_from_text() {
    Spi::run(&format!(
        "CREATE TABLE t (id souvenir);
         INSERT INTO t VALUES ('{USER}'::text);
         INSERT INTO t VALUES ('{USER}'::varchar);"
    ))
    .unwrap();
    assert_eq!(one::<i64>(&format!("SELECT count(*) FROM t WHERE id = '{USER}'")), Some(2));
    // ... and to text, so string functions and comparisons with text work.
    assert!(flag("SELECT starts_with(id::text, 'user_') FROM t LIMIT 1"));
}

#[pg_test]
fn null_handling() {
    assert_eq!(one::<String>("SELECT souvenir_prefix(NULL)"), None);
    assert_eq!(one::<String>("SELECT NULL::souvenir::text"), None);
}

#[pg_test]
fn arrays() {
    assert_eq!(
        text(&format!(
            "SELECT (ARRAY['{USER}'::souvenir, souvenir_min('a')])[1]::text"
        )),
        USER
    );
}
