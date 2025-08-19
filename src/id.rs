use crate::Error;
use crate::encoding::{decode_id, decode_prefix, encode_id, encode_prefix, encode_suffix};
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

/// Type of the underlying data stored in an [`Id`], which is an array of
/// 16 bytes. Note that not all possible values represent valid identifiers.
pub type IdBytes = [u8; 16];

/// A typed 128-bit identifier.
///
/// ```
/// use souvenir::Id;
///
/// let id: Id = Id::random("user").unwrap();
/// println!("{}", id);
///
/// let id2: Id = Id::parse("user_02v58c5a3fy30k560qrtg4").unwrap();
/// assert_eq!(id2.to_string(), "user_02v58c5a3fy30k560qrtg4");
/// ```
#[cfg_attr(
    feature = "diesel",
    derive(::diesel::AsExpression, ::diesel::FromSqlRow)
)]
#[cfg_attr(feature = "diesel", diesel(sql_type = ::diesel::sql_types::Int8))]
#[derive(Copy, Clone, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Id(IdBytes);

impl Id {
    /// Create a new [`Id`] with the following bytes.
    /// Will error if the format is not valid.
    pub fn new(value: [u8; 16]) -> Result<Self, Error> {
        let _ = encode_id(value)?;
        Ok(Self::from_bytes_unchecked(value))
    }

    /// Create a new [`Id`] given a prefix and suffix.
    /// Note that the upper 20 bits from the suffix are discarded.
    pub fn from_parts(prefix: &str, suffix: [u8; 16]) -> Result<Self, Error> {
        let suffix = u128::from_be_bytes(suffix) & ((1 << 108) - 1);
        let value = decode_prefix(prefix)? | suffix;
        Ok(Self::from_bytes_unchecked(value.to_be_bytes()))
    }

    /// Create a new [`Id`] with the provided raw value.
    /// The value is not checked to be a valid [`Id`].
    pub fn from_bytes_unchecked(value: [u8; 16]) -> Self {
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

    /// Test to see if the provided string is a valid [`Id`].
    pub fn test(value: &str) -> bool {
        Self::parse(value).is_ok()
    }

    /// Attempt to parse the provided string into an [`Id`].
    pub fn parse(value: &str) -> Result<Self, Error> {
        decode_id(value).map(Self::from_bytes_unchecked)
    }

    /// Get the prefix of this identifier.
    pub fn prefix(self) -> String {
        encode_prefix(self.to_u128()).unwrap()
    }

    /// Get the suffix of this identifier.
    pub fn suffix(self) -> String {
        encode_suffix(self.to_u128()).unwrap()
    }

    /// Cast this [`Id`] into an [`Id`] with a different prefix.
    pub fn cast(self, prefix: &str) -> Result<Self, Error> {
        Self::from_parts(prefix, self.0)
    }
}

impl Debug for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            encode_id(self.0).expect("Id could not be serialized correctly")
        )
    }
}

impl FromStr for Id {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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

    fn try_from(value: u128) -> Result<Self, Error> {
        Self::new(value.to_be_bytes())
    }
}

impl TryFrom<i128> for Id {
    type Error = Error;

    fn try_from(value: i128) -> Result<Self, Error> {
        Self::new(value.to_be_bytes())
    }
}

impl TryFrom<IdBytes> for Id {
    type Error = Error;

    fn try_from(value: IdBytes) -> Result<Self, Error> {
        Self::new(value)
    }
}

impl TryFrom<&[u8]> for Id {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::new(value.try_into().map_err(|_| Error::InvalidData)?)
    }
}
