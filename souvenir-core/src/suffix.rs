use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use crate::{
    encoding::{decode_suffix, encode_suffix},
    error::{Error, Result},
};

/// A valid [`Id`](crate::id::Id) suffix.
#[derive(Copy, Clone, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Suffix(u128);

impl Suffix {
    const MASK: u128 = (1 << 108) - 1;

    /// Create a [`Suffix`] from its inner [`u128`] value.
    pub fn new(value: u128) -> Self {
        Self(value & Self::MASK)
    }

    /// Retrieve the inner [`u128`] value from this [`Suffix`].
    pub fn to_u128(self) -> u128 {
        self.0
    }

    /// Attempt to parse the provided strong into a [`Suffix`]
    pub fn parse(suffix: &str) -> Result<Self> {
        decode_suffix(suffix)
    }
}

impl Debug for Suffix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl Display for Suffix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", encode_suffix(*self))
    }
}

impl FromStr for Suffix {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl From<Suffix> for u128 {
    fn from(value: Suffix) -> Self {
        value.0
    }
}

impl From<u128> for Suffix {
    fn from(value: u128) -> Self {
        Self(value)
    }
}
