use crate::encoding::{parse_base32, stringify_base32};
use crate::{Error, Identifiable};
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

/// Type of the underlying data stored in an `Id`.
pub type IdBytes = [u8; 8];

/// A typed 64-bit identifier.
///
/// ```
/// use souvenir::{Identifiable, Id};
///
/// struct User {
///     // fields omitted
/// }
///
/// impl Identifiable for User {
///     const PREFIX: &'static str = "user";
/// }
///
/// let id: Id<User> = Id::random();
/// let id2: Id<User> = Id::parse("user_4n3y65asan4bj").unwrap();
///
/// println!("{}", id);
///
/// ```
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(
    feature = "diesel",
    derive(::diesel::AsExpression, ::diesel::FromSqlRow)
)]
#[cfg_attr(feature = "diesel", diesel(sql_type = ::diesel::sql_types::Int8))]
pub struct Id<T: Identifiable> {
    marker: PhantomData<T>,
    value: IdBytes,
}

impl<T: Identifiable> Copy for Id<T> {}

impl<T: Identifiable> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Identifiable> Id<T> {
    /// Create a new `Id<T>` with the following underlying value.
    pub fn new(value: [u8; 8]) -> Self {
        Self {
            marker: PhantomData,
            value,
        }
    }

    /// Get the data value of the identifier.
    pub fn value(self) -> [u8; 8] {
        self.value
    }

    /// Convert a `u64` to an `Id<T>`.
    pub fn from_u64(value: u64) -> Self {
        Self::new(value.to_be_bytes())
    }

    /// Convert an `i64` to an `Id<T>`.
    pub fn from_i64(value: i64) -> Self {
        Self::new(value.to_be_bytes())
    }

    /// Get the data value of the identifier as a `u64`.
    pub fn to_u64(self) -> u64 {
        u64::from_be_bytes(self.value)
    }

    /// Get the data value of the identifier as an `i64`.
    pub fn to_i64(self) -> i64 {
        i64::from_be_bytes(self.value)
    }

    /// Test to see if the provided string is a valid `Id<T>`.
    pub fn test(value: &str) -> bool {
        Self::parse(value).is_ok()
    }

    /// Attempt to parse the provided string into an `Id<T>`.
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

    /// Get the prefix of this identifier
    pub const fn prefix(&self) -> &'static str {
        T::PREFIX
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
        value.to_u64()
    }
}

impl<T: Identifiable> From<Id<T>> for i64 {
    fn from(value: Id<T>) -> Self {
        value.to_i64()
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
