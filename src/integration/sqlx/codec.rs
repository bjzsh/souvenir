// `Souvenir::encode` is spelled out because sqlx's `Encode::encode` is in
// scope.

use crate::Souvenir;
use crate::spec::MAX_TEXT_LEN;
#[cfg(feature = "mysql")]
use sqlx::mysql::{MySql, MySqlValueRef};
#[cfg(feature = "postgres")]
use sqlx::postgres::PgArgumentBuffer;
#[cfg(feature = "mysql")]
use sqlx::{Decode, Encode, encode::IsNull, error::BoxDynError};

/// Append the text form to a Postgres argument buffer.
#[cfg(feature = "postgres")]
pub(crate) fn pg_encode_text(id: Souvenir, buf: &mut PgArgumentBuffer) {
    buf.extend_from_slice(Souvenir::encode(&id, &mut [0; MAX_TEXT_LEN]).as_bytes());
}

/// Write the text form as a MySQL string.
#[cfg(feature = "mysql")]
pub(crate) fn mysql_encode_text(id: Souvenir, buf: &mut Vec<u8>) -> Result<IsNull, BoxDynError> {
    <&str as Encode<'_, MySql>>::encode(Souvenir::encode(&id, &mut [0; MAX_TEXT_LEN]), buf)
}

/// Read a MySQL string (which may be flagged binary) and parse it.
#[cfg(feature = "mysql")]
pub(crate) fn mysql_decode_text<T>(
    value: MySqlValueRef<'_>,
    parse: fn(&str) -> crate::Result<T>,
) -> Result<T, BoxDynError> {
    let bytes = <&[u8] as Decode<MySql>>::decode(value)?;
    let text = core::str::from_utf8(bytes).map_err(|_| crate::Error::InvalidCharacter)?;
    Ok(parse(text)?)
}
