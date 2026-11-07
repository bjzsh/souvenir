use super::support::Value;
use diesel::serialize::IsNull;
use diesel::sqlite::Sqlite;

backend_to_sql!(Sqlite, |out, value| {
    match value {
        Value::Text(id) => out.set_value(id.to_string()),
        Value::Binary(bytes) => out.set_value(bytes.to_vec()),
    }
    Ok(IsNull::No)
});
