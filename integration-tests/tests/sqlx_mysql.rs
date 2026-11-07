use souvenir::{Error, Id, Souvenir};
use souvenir_integration_tests::{
    Account, Event, Fixture, MYSQL_TEXT, User, assert_db_err, assert_db_order, assert_err,
    fixtures, one, org_row, url,
};
use sqlx::mysql::MySqlPoolOptions;

type Row = (i32, Souvenir, Id<User>, Id<Event>, Id<Account>, Vec<u8>);

#[tokio::test]
async fn sqlx_mysql() {
    let Some(url) = url("SOUVENIR_TEST_MYSQL_URL") else {
        return;
    };
    let pool = MySqlPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .unwrap();
    sqlx::query(&format!(
        "CREATE TEMPORARY TABLE sqlx_ids (k INT PRIMARY KEY,
         any_text {MYSQL_TEXT} NOT NULL
             CHECK (REGEXP_LIKE(any_text, '^[a-z]{{1,8}}_[0-3][0-9a-hjkmnp-tv-z]{{24}}$', 'c')),
         user_text {MYSQL_TEXT} NOT NULL,
         event_bytes BINARY(20) NOT NULL, account_uuid BINARY(16) NOT NULL,
         any_bytes BINARY(20) NOT NULL)"
    ))
    .execute(&pool)
    .await
    .unwrap();

    let rows = fixtures();
    for f in &rows {
        sqlx::query("INSERT INTO sqlx_ids VALUES (?, ?, ?, ?, ?, ?)")
            .bind(f.k)
            .bind(f.any)
            .bind(f.user)
            .bind(f.event)
            .bind(f.account)
            .bind(&f.any.as_bytes()[..])
            .execute(&pool)
            .await
            .unwrap();
    }

    let stored: Vec<Row> = sqlx::query_as("SELECT * FROM sqlx_ids ORDER BY k")
        .fetch_all(&pool)
        .await
        .unwrap();
    let stored: Vec<Fixture> = stored
        .into_iter()
        .map(|(k, any, user, event, account, any_bytes)| {
            assert_eq!(Souvenir::from_slice(&any_bytes), Ok(any));
            Fixture {
                k,
                any,
                user,
                event,
                account,
            }
        })
        .collect();
    assert_eq!(stored, rows);

    assert_db_order::<_, Souvenir>(&pool, "sqlx_ids", "any_text", rows.iter().map(|f| f.any)).await;
    assert_db_order::<_, Id<User>>(&pool, "sqlx_ids", "user_text", rows.iter().map(|f| f.user))
        .await;
    assert_db_order::<_, Id<Event>>(
        &pool,
        "sqlx_ids",
        "event_bytes",
        rows.iter().map(|f| f.event),
    )
    .await;
    assert_db_order::<_, Id<Account>>(
        &pool,
        "sqlx_ids",
        "account_uuid",
        rows.iter().map(|f| f.account),
    )
    .await;

    let org = org_row().k;
    assert_err(
        one::<_, Id<User>>(
            &pool,
            &format!("SELECT any_text FROM sqlx_ids WHERE k = {org}"),
        )
        .await,
        Error::PrefixMismatch,
    );
    assert_err(
        one::<_, Id<Event>>(
            &pool,
            &format!("SELECT any_bytes FROM sqlx_ids WHERE k = {org}"),
        )
        .await,
        Error::PrefixMismatch,
    );

    // The CHECK constraint rejects anything that isn't a canonical ID.
    let junk = sqlx::query(
        "INSERT INTO sqlx_ids VALUES (999, 'abcdefghijklmnopqrst', 'x', x'00', x'00', x'00')",
    )
    .execute(&pool)
    .await;
    assert_db_err(junk, "constraint");

    // The declared form decides how a value is read; nothing is guessed.
    assert_err(
        one::<_, Id<Event>>(&pool, "SELECT user_text FROM sqlx_ids LIMIT 1").await,
        Error::InvalidLength,
    );
    assert_err(
        one::<_, Id<Account>>(&pool, "SELECT event_bytes FROM sqlx_ids LIMIT 1").await,
        Error::InvalidLength,
    );
    assert_err(
        one::<_, Id<Event>>(&pool, "SELECT x'0102'").await,
        Error::InvalidLength,
    );
    assert!(
        one::<_, Id<User>>(&pool, "SELECT event_bytes FROM sqlx_ids LIMIT 1")
            .await
            .is_err()
    );
}
