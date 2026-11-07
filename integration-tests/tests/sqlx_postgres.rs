use souvenir::{Error, Id, Souvenir};
use souvenir_integration_tests::{
    Account, Event, Fixture, PG_DOMAIN, User, assert_db_err, assert_db_order, assert_err, fixtures,
    one, org_row, url,
};
use sqlx::Row as _;
use sqlx::postgres::PgPoolOptions;

type Row = (i32, Souvenir, Id<User>, Id<Event>, Id<Account>, Vec<u8>);
type Arrays = (Vec<Id<User>>, Vec<Id<Event>>, Vec<Id<Account>>);

fn fixture((k, any, user, event, account, any_bytes): Row) -> Fixture {
    assert_eq!(Souvenir::from_slice(&any_bytes), Ok(any));
    Fixture {
        k,
        any,
        user,
        event,
        account,
    }
}

#[tokio::test]
async fn sqlx_postgres() {
    let Some(url) = url("SOUVENIR_TEST_PG_URL") else {
        return;
    };
    // One connection, so the temporary table is visible to every query.
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .unwrap();
    sqlx::raw_sql(PG_DOMAIN).execute(&pool).await.unwrap();
    sqlx::query(
        "CREATE TEMP TABLE sqlx_ids (k INT PRIMARY KEY,
         any_text souvenir NOT NULL, user_text souvenir NOT NULL,
         event_bytes BYTEA NOT NULL CHECK (octet_length(event_bytes) = 20),
         account_uuid UUID NOT NULL, any_bytes BYTEA NOT NULL)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let rows = fixtures();
    for f in &rows {
        sqlx::query("INSERT INTO sqlx_ids VALUES ($1, $2, $3, $4, $5, $6)")
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

    // Binary protocol.
    let stored: Vec<Row> = sqlx::query_as("SELECT * FROM sqlx_ids ORDER BY k")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(stored.into_iter().map(fixture).collect::<Vec<_>>(), rows);

    // Text protocol (simple query): text, `\x...` bytea and hyphenated uuids.
    let text_rows = sqlx::raw_sql("SELECT * FROM sqlx_ids ORDER BY k")
        .fetch_all(&pool)
        .await
        .unwrap();
    let stored: Vec<Fixture> = text_rows
        .iter()
        .map(|r| fixture((r.get(0), r.get(1), r.get(2), r.get(3), r.get(4), r.get(5))))
        .collect();
    assert_eq!(stored, rows);

    let uuid: String = one(&pool, "SELECT account_uuid::text FROM sqlx_ids WHERE k = 0")
        .await
        .unwrap();
    let expected = rows[0].account.to_uuid().to_string();
    assert_eq!(uuid, expected);

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

    let arrays: Arrays = sqlx::query_as(
        "SELECT array_agg(user_text::text ORDER BY k), array_agg(event_bytes ORDER BY k),
                array_agg(account_uuid ORDER BY k) FROM sqlx_ids",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(arrays.0, rows.iter().map(|f| f.user).collect::<Vec<_>>());
    assert_eq!(arrays.1, rows.iter().map(|f| f.event).collect::<Vec<_>>());
    assert_eq!(arrays.2, rows.iter().map(|f| f.account).collect::<Vec<_>>());

    // Plain text types work as well as the domain.
    let plain: Souvenir = one(&pool, "SELECT 'org_00t35pg1sgajz7jf1pz84mv4e'::varchar")
        .await
        .unwrap();
    assert_eq!(plain.prefix(), "org");

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

    // The domain rejects anything that isn't a canonical ID.
    let junk = sqlx::query(
        "INSERT INTO sqlx_ids VALUES (999, 'junk', 'junk', '\\x00', gen_random_uuid(), '\\x00')",
    )
    .execute(&pool)
    .await;
    assert_db_err(junk, "souvenir_check");

    // The declared form decides how a value is read; nothing is guessed.
    assert_err(
        one::<_, Id<Event>>(&pool, "SELECT '\\x0102'::bytea").await,
        Error::InvalidLength,
    );
    assert_err(
        one::<_, Id<Account>>(&pool, "SELECT '01a196d0-0e60-4a97-b3c9-e1b7d04a6c8e'::uuid").await,
        Error::NotUuidV7,
    );
    assert!(
        one::<_, Id<User>>(&pool, "SELECT event_bytes FROM sqlx_ids LIMIT 1")
            .await
            .is_err()
    );
    assert!(
        one::<_, Id<Event>>(&pool, "SELECT user_text FROM sqlx_ids LIMIT 1")
            .await
            .is_err()
    );
    assert!(
        one::<_, Id<Account>>(&pool, "SELECT event_bytes FROM sqlx_ids LIMIT 1")
            .await
            .is_err()
    );
}
