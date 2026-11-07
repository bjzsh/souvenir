use souvenir::{Error, Id, Souvenir};
use souvenir_integration_tests::{
    Account, Event, Fixture, User, assert_db_order, assert_err, fixtures, one, org_row,
};
use sqlx::sqlite::SqlitePoolOptions;

type Row = (i32, Souvenir, Id<User>, Id<Event>, Id<Account>, Vec<u8>);

#[tokio::test]
async fn sqlx_sqlite() {
    // One connection: every connection to `sqlite::memory:` is a new database.
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query(
        "CREATE TABLE ids (k INTEGER PRIMARY KEY, any_text TEXT NOT NULL, user_text TEXT NOT NULL,
         event_bytes BLOB NOT NULL, account_uuid BLOB NOT NULL, any_bytes BLOB NOT NULL)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let rows = fixtures();
    for f in &rows {
        sqlx::query("INSERT INTO ids VALUES (?, ?, ?, ?, ?, ?)")
            .bind(f.k)
            .bind(f.any)
            .bind(f.user)
            .bind(f.event)
            .bind(f.account)
            .bind(&f.any.as_bytes()[..]) // untyped bytes: the escape hatch
            .execute(&pool)
            .await
            .unwrap();
    }

    let stored: Vec<Row> = sqlx::query_as("SELECT * FROM ids ORDER BY k")
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

    let forms: (String, String, i64, i64) = sqlx::query_as(
        "SELECT typeof(user_text), user_text, length(event_bytes), length(account_uuid)
         FROM ids WHERE k = 0",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(forms, ("text".into(), rows[0].user.to_string(), 20, 16));

    assert_db_order::<_, Souvenir>(&pool, "ids", "any_text", rows.iter().map(|f| f.any)).await;
    assert_db_order::<_, Id<User>>(&pool, "ids", "user_text", rows.iter().map(|f| f.user)).await;
    assert_db_order::<_, Id<Event>>(&pool, "ids", "event_bytes", rows.iter().map(|f| f.event))
        .await;
    assert_db_order::<_, Id<Account>>(&pool, "ids", "account_uuid", rows.iter().map(|f| f.account))
        .await;

    let org = org_row().k;
    assert_err(
        one::<_, Id<User>>(&pool, &format!("SELECT any_text FROM ids WHERE k = {org}")).await,
        Error::PrefixMismatch,
    );
    assert_err(
        one::<_, Id<Event>>(&pool, &format!("SELECT any_bytes FROM ids WHERE k = {org}")).await,
        Error::PrefixMismatch,
    );

    // The declared form decides how a value is read; nothing is guessed.
    assert_err(
        one::<_, Souvenir>(&pool, "SELECT 'abcdefghijklmnopqrst'").await,
        Error::MissingSeparator,
    );
    assert_err(
        one::<_, Souvenir>(&pool, "SELECT 'user_00T35PG1SG9ECH32TYMRKDQ6V'").await,
        Error::InvalidCharacter,
    );
    assert_err(
        one::<_, Id<Event>>(&pool, "SELECT x'0102'").await,
        Error::InvalidLength,
    );
    assert_err(
        one::<_, Id<Event>>(&pool, "SELECT zeroblob(20)").await,
        Error::ZeroPrefix,
    );
    assert_err(
        one::<_, Id<Account>>(&pool, "SELECT zeroblob(16)").await,
        Error::NotUuidV7,
    );
    assert_err(
        one::<_, Id<Account>>(&pool, "SELECT event_bytes FROM ids LIMIT 1").await,
        Error::InvalidLength,
    );
    assert!(
        one::<_, Id<User>>(&pool, "SELECT event_bytes FROM ids LIMIT 1")
            .await
            .is_err()
    );
    assert!(
        one::<_, Id<Event>>(&pool, "SELECT user_text FROM ids LIMIT 1")
            .await
            .is_err()
    );
}
