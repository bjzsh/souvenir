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
    pub const fn new(value: u128) -> Self {
        Self(value & Self::MASK)
    }

    /// Retrieve the inner [`u128`] value from this [`Suffix`].
    pub const fn to_u128(self) -> u128 {
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
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::Suffix;
    use crate::{id::Id, prefix::Prefix};

    #[test]
    fn conversion_masks_high_bits_and_preserves_prefix() {
        let prefix = Prefix::parse("user").unwrap();
        for value in [0, 1 << 108, u128::MAX] {
            let suffix = Suffix::from(value);
            assert_eq!(suffix.to_u128(), value & ((1 << 108) - 1));
            let id = Id::new(prefix, suffix);
            assert_eq!(id.prefix(), prefix);
            assert_eq!(Id::parse(&id.to_string()), Ok(id));
        }
    }
}
