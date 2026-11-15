//! Needs the MySQL client library: run with `--features diesel-mysql`.
#![cfg(feature = "diesel-mysql")]

use diesel::prelude::*;
use souvenir::{Id, Souvenir};
use souvenir_integration_tests::{
    Account, Event, MYSQL_TEXT, User, assert_db_err, assert_err, assert_sorted, fixtures, org_row,
    url, wrong_prefix,
};

diesel::table! {
    diesel_ids (k) {
        k -> Integer,
        any_text -> Text,
        user_text -> Text,
        event_bytes -> Binary,
        account_uuid -> Binary,
        any_bytes -> Binary,
    }
}

#[derive(Debug, PartialEq, Insertable, Queryable)]
#[diesel(table_name = diesel_ids)]
struct Row {
    k: i32,
    any_text: Souvenir,
    user_text: Id<User>,
    event_bytes: Id<Event>,
    account_uuid: Id<Account>,
    any_bytes: Souvenir,
}

#[test]
fn diesel_mysql() {
    let Some(url) = url("SOUVENIR_TEST_MYSQL_URL") else {
        return;
    };
    let mut conn = MysqlConnection::establish(&url).unwrap();
    diesel::sql_query(format!(
        "CREATE TEMPORARY TABLE diesel_ids (k INT PRIMARY KEY,
         any_text {MYSQL_TEXT} NOT NULL
             CHECK (REGEXP_LIKE(any_text, '^[a-z]{{1,8}}_[0-3][0-9a-hjkmnp-tv-z]{{24}}$', 'c')),
         user_text {MYSQL_TEXT} NOT NULL,
         event_bytes BINARY(20) NOT NULL, account_uuid BINARY(16) NOT NULL,
         any_bytes BINARY(20) NOT NULL)"
    ))
    .execute(&mut conn)
    .unwrap();

    let rows: Vec<Row> = fixtures()
        .into_iter()
        .map(|f| Row {
            k: f.k,
            any_text: f.any,
            user_text: f.user,
            event_bytes: f.event,
            account_uuid: f.account,
            any_bytes: f.any,
        })
        .collect();
    diesel::insert_into(diesel_ids::table)
        .values(&rows)
        .execute(&mut conn)
        .unwrap();

    let stored: Vec<Row> = diesel_ids::table
        .order(diesel_ids::k)
        .load(&mut conn)
        .unwrap();
    assert_eq!(stored, rows);

    let by_text: Vec<Id<User>> = diesel_ids::table
        .select(diesel_ids::user_text)
        .order(diesel_ids::user_text)
        .load(&mut conn)
        .unwrap();
    assert_sorted(rows.iter().map(|r| r.user_text), &by_text);
    let by_bytes: Vec<Id<Event>> = diesel_ids::table
        .select(diesel_ids::event_bytes)
        .order(diesel_ids::event_bytes)
        .load(&mut conn)
        .unwrap();
    assert_sorted(rows.iter().map(|r| r.event_bytes), &by_bytes);
    let by_uuid: Vec<Id<Account>> = diesel_ids::table
        .select(diesel_ids::account_uuid)
        .order(diesel_ids::account_uuid)
        .load(&mut conn)
        .unwrap();
    assert_sorted(rows.iter().map(|r| r.account_uuid), &by_uuid);

    let org = diesel_ids::table.filter(diesel_ids::k.eq(org_row().k));
    assert_err(
        org.select(diesel_ids::any_text)
            .first::<Id<User>>(&mut conn),
        wrong_prefix::<User>(),
    );
    assert_err(
        org.select(diesel_ids::any_bytes)
            .first::<Id<Event>>(&mut conn),
        wrong_prefix::<Event>(),
    );

    // The CHECK constraint rejects anything that isn't a canonical ID.
    let junk = diesel::sql_query(
        "INSERT INTO diesel_ids VALUES (999, 'abcdefghijklmnopqrst', 'x', x'00', x'00', x'00')",
    )
    .execute(&mut conn);
    assert_db_err(junk, "constraint");

    // The declared form decides how a value is read: a UUID column's 16
    // bytes are not an ID's 20.
    assert_err(
        diesel_ids::table
            .select(diesel_ids::account_uuid)
            .first::<Id<Event>>(&mut conn),
        souvenir::Error::InvalidLength,
    );
}
