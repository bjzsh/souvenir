use crate::encoding::{parse_base32, stringify_base32};
use crate::{Error, Type};
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::str::FromStr;

/// Type of the underlying data stored in an [`Id`], which is an array of
/// 16 bytes.
pub type IdBytes = [u8; 16];

/// A typed 128-bit identifier.
///
/// ```
/// use souvenir::{Type, Id};
///
/// struct User {
///     // fields omitted
/// }
///
/// impl Type for User {
///     const PREFIX: &'static str = "user";
/// }
///
/// let id: Id<User> = Id::random();
/// println!("{}", id);
///
/// let id2: Id<User> = Id::parse("user_02v58c5a3fy30k560qrtg4rb2k").unwrap();
/// assert_eq!(id2.to_string(), "user_02v58c5a3fy30k560qrtg4rb2k");
/// ```
#[cfg_attr(
    feature = "diesel",
    derive(::diesel::AsExpression, ::diesel::FromSqlRow)
)]
#[cfg_attr(feature = "diesel", diesel(sql_type = ::diesel::sql_types::Int8))]
pub struct Id<T: Type + ?Sized> {
    marker: PhantomData<T>,
    value: IdBytes,
}

impl<T: Type + ?Sized> Id<T> {
    /// Create a new [`Id<T>`] with the following underlying value.
    pub fn new(value: [u8; 16]) -> Self {
        Self {
            marker: PhantomData,
            value,
        }
    }

    /// Get the data value of the identifier.
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.value
    }

    /// Get the data value of the identifier.
    pub fn to_bytes(self) -> [u8; 16] {
        self.value
    }

    /// Get the data value of the identifier as a [`u64`].
    pub fn to_u128(self) -> u128 {
        u128::from_be_bytes(self.value)
    }

    /// Get the data value of the identifier as an [`i64`].
    pub fn to_i128(self) -> i128 {
        i128::from_be_bytes(self.value)
    }

    /// Test to see if the provided string is a valid [`Id<T>`].
    pub fn test(value: &str) -> bool {
        Self::parse(value).is_ok()
    }

    /// Attempt to parse the provided string into an [`Id<T>`].
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

    /// Get the prefix of this identifier.
    pub fn prefix(self) -> &'static str {
        T::PREFIX
    }

    /// Cast this [`Id`] into an [`Id`] of a different type. Does not check
    /// if the target type has the same prefix as this type.
    pub const fn cast<U: Type + ?Sized>(self) -> Id<U> {
        Id {
            marker: PhantomData,
            value: self.value,
        }
    }
}

impl<T: Type + ?Sized> Copy for Id<T> {}

impl<T: Type + ?Sized> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Type + ?Sized> Default for Id<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T: Type + ?Sized> std::hash::Hash for Id<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<T: Type + ?Sized> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: Type + ?Sized> Eq for Id<T> {}

impl<T: Type + ?Sized> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Type + ?Sized> Ord for Id<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl<T: Type + ?Sized> Debug for Id<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl<T: Type + ?Sized> Display for Id<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}_{}",
            T::PREFIX,
            stringify_base32(self.value).expect("id value to stringify correctly")
        )
    }
}

impl<T: Type + ?Sized> FromStr for Id<T> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl<T: Type + ?Sized> From<Id<T>> for u128 {
    fn from(value: Id<T>) -> Self {
        value.to_u128()
    }
}

impl<T: Type + ?Sized> From<Id<T>> for i128 {
    fn from(value: Id<T>) -> Self {
        value.to_i128()
    }
}

impl<T: Type + ?Sized> From<Id<T>> for IdBytes {
    fn from(value: Id<T>) -> Self {
        value.to_bytes()
    }
}

impl<T: Type + ?Sized> From<u128> for Id<T> {
    fn from(value: u128) -> Self {
        Self::new(value.to_be_bytes())
    }
}

impl<T: Type + ?Sized> From<i128> for Id<T> {
    fn from(value: i128) -> Self {
        Self::new(value.to_be_bytes())
    }
}

impl<T: Type + ?Sized> From<IdBytes> for Id<T> {
    fn from(value: IdBytes) -> Self {
        Self::new(value)
    }
}

impl<T: Type + ?Sized> TryFrom<&[u8]> for Id<T> {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self::new(value.try_into().map_err(|_| Error::InvalidData)?))
    }
}
