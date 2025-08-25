use crate::encoding::{decode_id, encode_id, validate_id};
use crate::error::{Error, Result};
use crate::prefix::Prefix;
use crate::suffix::Suffix;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

/// Type of the underlying data stored in an [`Id`], which is an array of
/// 16 bytes. Note that not all possible values represent valid identifiers.
pub type IdBytes = [u8; 16];

/// A 128-bit identifier consisting of a 1-4 character tag and 108 random bits.
///
/// ```
/// # use souvenir_core::id::Id;
/// let id: Id = Id::random("user".parse().unwrap());
/// println!("{}", id);
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
    /// Create a new [`Id`] with the provided prefix and suffix.
    pub fn new(prefix: Prefix, suffix: Suffix) -> Self {
        let prefix = (prefix.to_u32() as u128) << 108;
        let suffix = suffix.to_u128();

        unsafe { Self::from_bytes_unchecked((prefix | suffix).to_be_bytes()) }
    }

    /// Create a new [`Id`] with the following bytes. If the provided bytes do
    /// not form a valid [`Id`], this method will error.
    pub fn from_bytes(value: [u8; 16]) -> Result<Self> {
        validate_id(value)
    }

    /// Create a new [`Id`] with the provided raw value.
    /// The value is not checked to be a valid [`Id`].
    ///
    /// # Safety
    /// This method is unsafe because the API assumes that the provided value
    /// is valid in order to provide memory safety.
    pub const unsafe fn from_bytes_unchecked(value: [u8; 16]) -> Self {
        Self(value)
    }

    /// Get the data value of the identifier.
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    /// Get the data value of the identifier.
    pub fn to_bytes(self) -> [u8; 16] {
        self.0
    }

    /// Get the data value of the identifier as a [`u128`].
    pub fn to_u128(self) -> u128 {
        u128::from_be_bytes(self.0)
    }

    /// Get the data value of the identifier as an [`i128`].
    pub fn to_i128(self) -> i128 {
        i128::from_be_bytes(self.0)
    }

    /// Get the prefix of this identifier.
    pub fn prefix(self) -> Prefix {
        unsafe { Prefix::new_unchecked((self.to_u128() >> 108) as u32) }
    }

    /// Get the suffix of this identifier.
    pub fn suffix(self) -> Suffix {
        Suffix::new(self.to_u128())
    }

    /// Cast this [`Id`] into an [`Id`] with a different prefix.
    pub fn cast(self, prefix: Prefix) -> Self {
        Self::new(prefix, Suffix::new(u128::from_be_bytes(self.0)))
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
        Self::new(Prefix::default(), Suffix::default())
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

impl TryFrom<u128> for Id {
    type Error = Error;

    fn try_from(value: u128) -> Result<Self> {
        Self::from_bytes(value.to_be_bytes())
    }
}

impl TryFrom<i128> for Id {
    type Error = Error;

    fn try_from(value: i128) -> Result<Self> {
        Self::from_bytes(value.to_be_bytes())
    }
}

impl TryFrom<IdBytes> for Id {
    type Error = Error;

    fn try_from(value: IdBytes) -> Result<Self> {
        Self::from_bytes(value)
    }
}

impl TryFrom<&[u8]> for Id {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self> {
        Self::from_bytes(value.try_into().map_err(|_| Error::InvalidData)?)
    }
}
