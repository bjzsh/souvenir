use super::codec::{mysql_decode_text, mysql_encode_text};
use super::compat::mysql_string;
use crate::integration::stored::{StoredBinary, decode_stored_binary};
use crate::{Id, Souvenir, Storage, Tagged};
use sqlx::mysql::{MySql, MySqlTypeInfo, MySqlValueRef};
use sqlx::{Decode, Encode, Type, encode::IsNull, error::BoxDynError};

impl Type<MySql> for Souvenir {
    fn type_info() -> MySqlTypeInfo {
        <&str as Type<MySql>>::type_info()
    }

    fn compatible(ty: &MySqlTypeInfo) -> bool {
        mysql_string(ty)
    }
}

impl Encode<'_, MySql> for Souvenir {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> Result<IsNull, BoxDynError> {
        mysql_encode_text(*self, buf)
    }
}

impl Decode<'_, MySql> for Souvenir {
    fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
        mysql_decode_text(value, Self::parse)
    }
}

impl<T: Tagged> Type<MySql> for Id<T> {
    fn type_info() -> MySqlTypeInfo {
        match T::STORAGE {
            Storage::Text => <&str as Type<MySql>>::type_info(),
            Storage::Bytes | Storage::Uuid => <&[u8] as Type<MySql>>::type_info(),
        }
    }

    fn compatible(ty: &MySqlTypeInfo) -> bool {
        mysql_string(ty)
    }
}

impl<T: Tagged> Encode<'_, MySql> for Id<T> {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> Result<IsNull, BoxDynError> {
        match T::STORAGE {
            Storage::Text => mysql_encode_text(self.to_untyped(), buf),
            Storage::Bytes | Storage::Uuid => {
                <&[u8] as Encode<'_, MySql>>::encode(StoredBinary::of(self).as_slice(), buf)
            }
        }
    }
}

impl<T: Tagged> Decode<'_, MySql> for Id<T> {
    fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
        match T::STORAGE {
            Storage::Text => mysql_decode_text(value, Self::parse),
            Storage::Bytes | Storage::Uuid => {
                Ok(decode_stored_binary(<&[u8] as Decode<MySql>>::decode(
                    value,
                )?)?)
            }
        }
    }
}
