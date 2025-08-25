use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use crate::{
    encoding::{decode_prefix, encode_prefix, validate_prefix},
    error::{Error, Result},
};

/// A valid [`Id`](crate::id::Id) prefix.
#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Prefix(u32);

impl Prefix {
    /// Create a [`Prefix`] from its inner [`u32`] value.
    /// If the provided value is not valid, this will error.
    pub fn new(value: u32) -> Result<Self> {
        validate_prefix(value)
    }

    /// Create a [`Prefix`] from its inner [`u32`] value.
    ///
    /// # Safety
    /// This is potentially unsafe as the rest of the API assumes that the
    /// inner value is valid.
    pub unsafe fn new_unchecked(value: u32) -> Self {
        Self(value)
    }

    /// Retrieve the inner [`u32`] value from this [`Prefix`].
    pub fn to_u32(self) -> u32 {
        self.0
    }

    /// Attempt to parse the provided string into a [`Prefix`]
    pub fn parse(prefix: &str) -> Result<Self> {
        decode_prefix(prefix)
    }
}

impl Default for Prefix {
    fn default() -> Self {
        Self(0b01001_00100_00000_00000_00000)
    }
}

impl Debug for Prefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl Display for Prefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", encode_prefix(*self))
    }
}

impl FromStr for Prefix {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl From<Prefix> for u32 {
    fn from(value: Prefix) -> Self {
        value.0
    }
}

impl TryFrom<u32> for Prefix {
    type Error = Error;

    fn try_from(value: u32) -> std::result::Result<Self, Self::Error> {
        Self::new(value)
    }
}
