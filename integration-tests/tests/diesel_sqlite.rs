use diesel::prelude::*;
use souvenir::{Error, Id, Souvenir};
use souvenir_integration_tests::{
    Account, Event, User, assert_err, assert_sorted, fixtures, org_row,
};

diesel::table! {
    ids (k) {
        k -> Integer,
        any_text -> Text,
        user_text -> Text,
        event_bytes -> Binary,
        account_uuid -> Binary,
        any_bytes -> Binary,
    }
}

#[derive(Debug, PartialEq, Insertable, Queryable)]
#[diesel(table_name = ids)]
struct Row {
    k: i32,
    any_text: Souvenir,
    user_text: Id<User>,
    event_bytes: Id<Event>,
    account_uuid: Id<Account>,
    any_bytes: Souvenir, // diesel maps untyped IDs to Binary columns too
}

#[test]
fn diesel_sqlite() {
    let mut conn = SqliteConnection::establish(":memory:").unwrap();
    diesel::sql_query(
        "CREATE TABLE ids (k INTEGER PRIMARY KEY, any_text TEXT NOT NULL, user_text TEXT NOT NULL,
         event_bytes BLOB NOT NULL, account_uuid BLOB NOT NULL, any_bytes BLOB NOT NULL)",
    )
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
    diesel::insert_into(ids::table)
        .values(&rows)
        .execute(&mut conn)
        .unwrap();

    let stored: Vec<Row> = ids::table.order(ids::k).load(&mut conn).unwrap();
    assert_eq!(stored, rows);

    use diesel::dsl::sql;
    use diesel::sql_types::BigInt;
    let lengths: (i64, i64) = ids::table
        .select((
            sql::<BigInt>("length(event_bytes)"),
            sql::<BigInt>("length(account_uuid)"),
        ))
        .first(&mut conn)
        .unwrap();
    assert_eq!(lengths, (20, 16));

    let by_text: Vec<Id<User>> = ids::table
        .select(ids::user_text)
        .order(ids::user_text)
        .load(&mut conn)
        .unwrap();
    assert_sorted(rows.iter().map(|r| r.user_text), &by_text);
    let by_bytes: Vec<Id<Event>> = ids::table
        .select(ids::event_bytes)
        .order(ids::event_bytes)
        .load(&mut conn)
        .unwrap();
    assert_sorted(rows.iter().map(|r| r.event_bytes), &by_bytes);
    let by_uuid: Vec<Id<Account>> = ids::table
        .select(ids::account_uuid)
        .order(ids::account_uuid)
        .load(&mut conn)
        .unwrap();
    assert_sorted(rows.iter().map(|r| r.account_uuid), &by_uuid);
    let by_any: Vec<Souvenir> = ids::table
        .select(ids::any_bytes)
        .order(ids::any_bytes)
        .load(&mut conn)
        .unwrap();
    assert_sorted(rows.iter().map(|r| r.any_bytes), &by_any);

    let found: Vec<i32> = ids::table
        .select(ids::k)
        .filter(ids::user_text.eq(rows[3].user_text))
        .filter(ids::event_bytes.eq(rows[3].event_bytes))
        .filter(ids::account_uuid.eq(rows[3].account_uuid))
        .load(&mut conn)
        .unwrap();
    assert_eq!(found, [3]);

    let org = ids::table.filter(ids::k.eq(org_row().k));
    assert_err(
        org.select(ids::any_text).first::<Id<User>>(&mut conn),
        Error::PrefixMismatch,
    );
    assert_err(
        org.select(ids::any_bytes).first::<Id<Event>>(&mut conn),
        Error::PrefixMismatch,
    );

    diesel::sql_query(
        "INSERT INTO ids VALUES (999, 'abcdefghijklmnopqrst', 'x', zeroblob(20), zeroblob(16), x'0102')",
    )
    .execute(&mut conn)
    .unwrap();
    let bad = ids::table.filter(ids::k.eq(999));
    assert_err(
        bad.select(ids::any_text).first::<Souvenir>(&mut conn),
        Error::MissingSeparator,
    );
    assert_err(
        bad.select(ids::event_bytes).first::<Id<Event>>(&mut conn),
        Error::ZeroPrefix,
    );
    assert_err(
        bad.select(ids::account_uuid)
            .first::<Id<Account>>(&mut conn),
        Error::NotUuidV7,
    );
    assert_err(
        bad.select(ids::any_bytes).first::<Souvenir>(&mut conn),
        Error::InvalidLength,
    );
}
