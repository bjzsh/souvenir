use crate::Identifiable;
use base58::{FromBase58, ToBase58};
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
    pub(crate) value: [u8; 8],
}

impl<T: Identifiable> Copy for Id<T> {}

impl<T: Identifiable> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Identifiable> Id<T> {
    pub fn new(value: [u8; 8]) -> Self {
        Self {
            marker: PhantomData,
            value,
        }
    }

    pub fn from_u64(value: u64) -> Self {
        Self::new(value.to_le_bytes())
    }

    pub fn from_i64(value: i64) -> Self {
        Self::new(value.to_le_bytes())
    }

    pub fn as_u64(self) -> u64 {
        u64::from_le_bytes(self.value)
    }

    pub fn as_i64(self) -> i64 {
        i64::from_le_bytes(self.value)
    }

    pub fn test(value: &str) -> bool {
        Self::parse(value).is_some()
    }

    pub fn parse(value: &str) -> Option<Self> {
        let (prefix, value) = value.split_once('_')?;

        if prefix != T::prefix() {
            return None;
        }

        value
            .from_base58()
            .ok()
            .map(|vec| -> Option<[u8; 8]> { vec.try_into().ok() })
            .flatten()
            .map(|value| Self::new(value))
    }
}

impl<T: Identifiable> Display for Id<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}_{}",
            T::prefix(),
            self.value.to_base58()
        )
    }
}

impl<T: Identifiable> From<Id<T>> for u64 {
    fn from(value: Id<T>) -> Self {
        value.as_u64()
    }
}

impl<T: Identifiable> From<Id<T>> for i64 {
    fn from(value: Id<T>) -> Self {
        value.as_i64()
    }
}

impl<T: Identifiable> From<u64> for Id<T> {
    fn from(value: u64) -> Self {
        Self::from_u64(value)
    }
}

impl<T: Identifiable> From<i64> for Id<T> {
    fn from(value: i64) -> Self {
        Self::from_i64(value)
    }
}