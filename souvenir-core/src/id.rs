use crate::encoding::{decode_id, encode_id, id_interval, split_id, suffix_interval};
use crate::error::{Error, Result};
use crate::prefix::Prefix;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

/// Type of the underlying data stored in an [`Id`], which is an array of
/// 16 bytes. Every bit pattern represents a valid identifier.
pub type IdBytes = [u8; 16];

/// A 128-bit identifier with a 1–4 letter prefix and approximately 109.142
/// bits of random suffix entropy.
///
/// ```
/// # use souvenir_core::id::Id;
/// # #[cfg(feature = "rand")]
/// # {
/// let id: Id = Id::random("user".parse().unwrap());
/// println!("{}", id);
/// # }
///
/// let id2: Id = Id::parse("user_02v58c5a3fy30k560qrtg4").unwrap();
/// assert_eq!(id2.to_string(), "user_02v58c5a3fy30k560qrtg4");
/// ```
#[cfg_attr(
    feature = "diesel",
    derive(::diesel::AsExpression, ::diesel::FromSqlRow)
)]
#[cfg_attr(all(feature = "diesel", feature = "postgres"), diesel(sql_type = ::diesel::sql_types::Uuid))]
#[cfg_attr(all(feature = "diesel", feature = "mysql"), diesel(sql_type = ::diesel::sql_types::Binary))]
#[cfg_attr(all(feature = "diesel", feature = "sqlite"), diesel(sql_type = ::diesel::sql_types::Text))]
#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Id(IdBytes);

impl Id {
    /// Create a new [`Id`] with the provided bytes.
    pub const fn new(value: [u8; 16]) -> Self {
        Self(value)
    }

    /// Create a new [`Id`] with the following prefix and suffix.
    /// If the suffix is not valid for the provided prefix, this will error.
    pub fn from_parts(prefix: Prefix, suffix: u128) -> Result<Self> {
        if !suffix_interval(prefix).contains(&suffix) {
            return Err(Error::InvalidData);
        }
        let value = *id_interval(prefix).start() + suffix;
        Ok(Self::new(value.to_be_bytes()))
    }

    /// Change the prefix while preserving the suffix, if it fits the destination.
    pub fn with_prefix(self, prefix: Prefix) -> Result<Self> {
        Self::from_parts(prefix, self.suffix())
    }

    /// Get the data value of the identifier.
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    /// Get the data value of the identifier.
    pub const fn to_bytes(self) -> [u8; 16] {
        self.0
    }

    /// Get the data value of the identifier as a [`u128`].
    pub const fn to_u128(self) -> u128 {
        u128::from_be_bytes(self.0)
    }

    /// Get the data value of the identifier as an [`i128`].
    pub const fn to_i128(self) -> i128 {
        i128::from_be_bytes(self.0)
    }

    /// Get the prefix of this identifier.
    pub const fn prefix(self) -> Prefix {
        unsafe { Prefix::new_unchecked(split_id(self.to_u128()).0) }
    }

    /// Get the suffix of this identifier.
    pub const fn suffix(self) -> u128 {
        split_id(self.to_u128()).1
    }

    /// Test to see if the provided string is a valid [`Id`].
    pub fn test(value: &str) -> bool {
        Self::parse(value).is_ok()
    }

    /// Attempt to parse the provided string into an [`Id`].
    pub fn parse(value: &str) -> Result<Self> {
        decode_id(value)
    }
}

impl Debug for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", encode_id(*self))
    }
}

impl Default for Id {
    fn default() -> Self {
        Self::from(*id_interval(Prefix::default()).start())
    }
}

impl FromStr for Id {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

impl From<Id> for u128 {
    fn from(value: Id) -> Self {
        value.to_u128()
    }
}

impl From<Id> for i128 {
    fn from(value: Id) -> Self {
        value.to_i128()
    }
}

impl From<Id> for IdBytes {
    fn from(value: Id) -> Self {
        value.to_bytes()
    }
}

impl From<u128> for Id {
    fn from(value: u128) -> Self {
        Self::new(value.to_be_bytes())
    }
}

impl From<i128> for Id {
    fn from(value: i128) -> Self {
        Self::new(value.to_be_bytes())
    }
}

impl From<IdBytes> for Id {
    fn from(value: IdBytes) -> Self {
        Self::new(value)
    }
}

impl TryFrom<&[u8]> for Id {
    type Error = Error;
    fn try_from(value: &[u8]) -> Result<Self> {
        let bytes = value.try_into().map_err(|_| Error::InvalidLength {
            expected: 16,
            found: value.len(),
        })?;
        Ok(Self::new(bytes))
    }
}
