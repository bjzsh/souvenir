use super::support::write_value;
use diesel::mysql::Mysql;

backend_to_sql!(Mysql, |out, value| write_value(out, value));
