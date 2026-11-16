//! The `souvenir` extension's native type through sqlx.

use souvenir::{Error, Id, Souvenir};
use souvenir_integration_tests::{
    Order, PG_EXTENSION, User, assert_db_err, assert_db_order, assert_err, fixtures, one,
    order_ids, org_row, url,
};
use sqlx::Row as _;
use sqlx::postgres::PgPoolOptions;

#[tokio::test]
async fn sqlx_postgres_native() {
    let Some(url) = url("SOUVENIR_TEST_PG_NATIVE_URL") else {
        return;
    };
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .unwrap();
    sqlx::raw_sql(PG_EXTENSION).execute(&pool).await.unwrap();
    sqlx::query(
        "CREATE TEMP TABLE sqlx_native (k INT PRIMARY KEY,
         mixed souvenir NOT NULL, ord souvenir NOT NULL UNIQUE,
         made souvenir NOT NULL DEFAULT gen_souvenir('user'))",
    )
    .execute(&pool)
    .await
    .unwrap();

    let rows = fixtures();
    let orders = order_ids();
    for (f, order) in rows.iter().zip(&orders) {
        // Typed IDs are sent as the type's 20 bytes; untyped ones as text,
        // which converts on assignment.
        sqlx::query("INSERT INTO sqlx_native (k, mixed, ord) VALUES ($1, $2, $3)")
            .bind(f.k)
            .bind(f.any)
            .bind(*order)
            .execute(&pool)
            .await
            .unwrap();
    }

    // Binary protocol.
    let stored: Vec<(i32, Souvenir, Id<Order>)> =
        sqlx::query_as("SELECT k, mixed, ord FROM sqlx_native ORDER BY k")
            .fetch_all(&pool)
            .await
            .unwrap();
    for ((k, any, order), (f, expected)) in stored.iter().zip(rows.iter().zip(&orders)) {
        assert_eq!((*k, *any, *order), (f.k, f.any, *expected));
    }
    assert_eq!(stored.len(), rows.len());

    // Text protocol: the type prints as the ID string.
    let text_rows = sqlx::raw_sql("SELECT mixed, ord FROM sqlx_native ORDER BY k")
        .fetch_all(&pool)
        .await
        .unwrap();
    for (row, (f, order)) in text_rows.iter().zip(rows.iter().zip(&orders)) {
        assert_eq!(row.get::<Souvenir, _>(0), f.any);
        assert_eq!(row.get::<Id<Order>, _>(1), *order);
    }

    // Generated in SQL, read in Rust.
    let made: Souvenir = one(&pool, "SELECT made FROM sqlx_native LIMIT 1")
        .await
        .unwrap();
    assert_eq!(made.prefix(), "user");
    let made: Id<User> = Id::try_from(made).unwrap();
    let prefix: String = sqlx::query_scalar("SELECT souvenir_prefix($1::souvenir)")
        .bind(made.to_untyped())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(prefix, "user");

    // The database orders IDs the way Rust does.
    assert_db_order::<_, Souvenir>(&pool, "sqlx_native", "mixed", rows.iter().map(|f| f.any)).await;
    assert_db_order::<_, Id<Order>>(&pool, "sqlx_native", "ord", orders.iter().copied()).await;

    // Typed parameters compare directly with the column.
    let k: i32 = sqlx::query_scalar("SELECT k FROM sqlx_native WHERE ord = $1")
        .bind(orders[7])
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(k, 7);

    // Arrays.
    let all: Vec<Id<Order>> =
        sqlx::query_scalar("SELECT array_agg(ord ORDER BY k) FROM sqlx_native")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(all, orders);
    let found: i64 = sqlx::query_scalar("SELECT count(*) FROM sqlx_native WHERE ord = ANY($1)")
        .bind(&orders[..5])
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(found, 5);

    // Prefix range scans.
    let users: i64 =
        one(&pool, "SELECT count(*) FROM sqlx_native WHERE mixed BETWEEN souvenir_min('user') AND souvenir_max('user')")
            .await
            .unwrap();
    let expected = rows.iter().filter(|f| f.any.prefix() == "user").count();
    assert_eq!(users, expected as i64);

    // Reading a natively stored ID with the wrong prefix fails.
    let org = org_row().k;
    assert_err(
        one::<_, Id<Order>>(
            &pool,
            &format!("SELECT mixed FROM sqlx_native WHERE k = {org}"),
        )
        .await,
        Error::PrefixMismatch,
    );

    // The type itself rejects anything that isn't an ID.
    let junk = sqlx::query("INSERT INTO sqlx_native (k, mixed, ord) VALUES (999, 'junk', 'junk')")
        .execute(&pool)
        .await;
    assert_db_err(junk, "invalid souvenir");
}
