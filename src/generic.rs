use std::{
    fmt::{Debug, Display, Formatter},
    str::FromStr,
};

use crate::{Error, Id, Type, encoding::parse_base32};

impl Type for () {
    const PREFIX: &'static str = "";
}

/// A runtime-tagged identifier.
#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Generic {
    prefix: String,
    value: Id<()>,
}

impl Generic {
    /// Create a new `Id<T>` with the following underlying value.
    pub fn new(prefix: impl Into<String>, value: [u8; 16]) -> Self {
        Self {
            prefix: prefix.into(),
            value: Id::new(value),
        }
    }

    /// Get the data value of the identifier.
    pub fn as_bytes(&self) -> &[u8; 16] {
        self.value.as_bytes()
    }

    /// Get the data value of the identifier.
    pub fn to_bytes(self) -> [u8; 16] {
        self.value.to_bytes()
    }

    /// Get the data value of the identifier as a [`u64`].
    pub fn to_u128(&self) -> u128 {
        self.value.to_u128()
    }

    /// Get the data value of the identifier as an [`i64`].
    pub fn to_i128(&self) -> i128 {
        self.value.to_i128()
    }

    /// Test to see if the provided string is a valid [`Generic`].
    pub fn test(value: &str) -> bool {
        Self::parse(value).is_ok()
    }

    /// Attempt to parse the provided string into a [`Generic`].
    pub fn parse(value: &str) -> Result<Self, Error> {
        let (prefix, value) = value.split_once('_').ok_or(Error::InvalidData)?;

        Ok(Self {
            prefix: prefix.to_owned(),
            value: Id::new(parse_base32(value)?),
        })
    }

    /// Get the prefix of this identifier.
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    /// Cast this [`Generic`] into an [`Id`] of a different type,
    /// failing if the prefix does not match the prefix of the target type.
    pub fn cast<U: Type + ?Sized>(&self) -> Result<Id<U>, Error> {
        if self.prefix != U::PREFIX {
            return Err(Error::PrefixMismatch {
                expected: U::PREFIX,
                actual: self.prefix.clone(),
            });
        }

        Ok(self.value.cast())
    }

    /// Cast this [`Generic`] into an [`Id<U>`] regardless of the prefix.
    pub const fn cast_unchecked<U: Type + ?Sized>(&self) -> Id<U> {
        self.value.cast()
    }
}

impl Debug for Generic {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl Display for Generic {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.prefix, self.value)
    }
}

impl FromStr for Generic {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl From<Generic> for u128 {
    fn from(value: Generic) -> Self {
        value.to_u128()
    }
}

impl From<Generic> for i128 {
    fn from(value: Generic) -> Self {
        value.to_i128()
    }
}

#[cfg(test)]
mod test {
    use super::Generic;

    #[test]
    fn generic_display() {
        let id = Generic::parse("test_7zzzzzzzzzzzzzzzzzzzzzzzzz").unwrap();
        assert_eq!(id.to_string(), "test_7zzzzzzzzzzzzzzzzzzzzzzzzz");
    }
}
