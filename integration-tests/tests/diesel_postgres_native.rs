//! The `souvenir` extension's native type through diesel.

use diesel::prelude::*;
use souvenir::{Error, Id, Souvenir};
use souvenir_integration_tests::{
    Order, PG_EXTENSION, assert_err, assert_sorted, fixtures, order_ids, org_row, url,
};

diesel::table! {
    use diesel::sql_types::Integer;
    use souvenir::sql_types::Souvenir;

    diesel_native (k) {
        k -> Integer,
        mixed -> Souvenir,
        ord -> Souvenir,
    }
}

#[derive(Debug, PartialEq, Insertable, Queryable)]
#[diesel(table_name = diesel_native)]
struct Row {
    k: i32,
    mixed: Souvenir,
    ord: Id<Order>,
}

#[test]
fn diesel_postgres_native() {
    let Some(url) = url("SOUVENIR_TEST_PG_NATIVE_URL") else {
        return;
    };
    let mut conn = PgConnection::establish(&url).unwrap();
    diesel::sql_query(PG_EXTENSION).execute(&mut conn).unwrap();
    diesel::sql_query(
        "CREATE TEMP TABLE diesel_native (k INT PRIMARY KEY, mixed souvenir NOT NULL,
         ord souvenir NOT NULL)",
    )
    .execute(&mut conn)
    .unwrap();

    let rows: Vec<Row> = fixtures()
        .into_iter()
        .zip(order_ids())
        .map(|(f, ord)| Row {
            k: f.k,
            mixed: f.any,
            ord,
        })
        .collect();
    diesel::insert_into(diesel_native::table)
        .values(&rows)
        .execute(&mut conn)
        .unwrap();

    let stored: Vec<Row> = diesel_native::table
        .order(diesel_native::k)
        .load(&mut conn)
        .unwrap();
    assert_eq!(stored, rows);

    let sorted: Vec<Id<Order>> = diesel_native::table
        .select(diesel_native::ord)
        .order(diesel_native::ord)
        .load(&mut conn)
        .unwrap();
    assert_sorted(rows.iter().map(|r| r.ord), &sorted);

    let k: i32 = diesel_native::table
        .select(diesel_native::k)
        .filter(diesel_native::ord.eq(rows[7].ord))
        .first(&mut conn)
        .unwrap();
    assert_eq!(k, 7);

    assert_err(
        diesel_native::table
            .select(diesel_native::mixed)
            .filter(diesel_native::k.eq(org_row().k))
            .first::<Id<Order>>(&mut conn),
        Error::PrefixMismatch,
    );
}
