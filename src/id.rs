use crate::encoding::{parse_base32, stringify_base32};
use crate::{Error, Identifiable};
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(
    feature = "diesel",
    derive(::diesel::AsExpression, ::diesel::FromSqlRow)
)]
#[cfg_attr(feature = "diesel", diesel(sql_type = ::diesel::sql_types::Int8))]
pub struct Id<T: Identifiable> {
    marker: PhantomData<T>,
    pub(crate) value: u64,
}

impl<T: Identifiable> Copy for Id<T> {}

impl<T: Identifiable> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Identifiable> Id<T> {
    pub fn new(value: u64) -> Self {
        Self {
            marker: PhantomData,
            value,
        }
    }

    pub fn value(self) -> u64 {
        self.value
    }

    pub fn from_i64(value: i64) -> Self {
        Self::new(u64::from_le_bytes(value.to_le_bytes()))
    }

    pub fn as_i64(self) -> i64 {
        i64::from_le_bytes(self.value.to_le_bytes())
    }

    pub fn test(value: &str) -> bool {
        Self::parse(value).is_ok()
    }

    pub fn parse(value: &str) -> Result<Self, Error> {
        let (prefix, value) = value.split_once('_').ok_or(Error::InvalidData)?;

        if prefix != T::PREFIX {
            return Err(Error::PrefixMismatch {
                expected: T::PREFIX,
                actual: String::from(prefix),
            });
        }

        Ok(Self::new(parse_base32(value)?))
    }
}

impl<T: Identifiable> Display for Id<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}_{}",
            T::PREFIX,
            stringify_base32(self.value).expect("id value to stringify correctly")
        )
    }
}

impl<T: Identifiable> From<Id<T>> for u64 {
    fn from(value: Id<T>) -> Self {
        value.value
    }
}

impl<T: Identifiable> From<Id<T>> for i64 {
    fn from(value: Id<T>) -> Self {
        value.as_i64()
    }
}

impl<T: Identifiable> From<u64> for Id<T> {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl<T: Identifiable> From<i64> for Id<T> {
    fn from(value: i64) -> Self {
        Self::from_i64(value)
    }
}
