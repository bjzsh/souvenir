use crate::error::{Error, Result};
use crate::spec::{MAX_PREFIX_LEN, PREFIX_COUNTS, PREFIX_RANK_LIMIT};
use crate::util::ascii;
use core::borrow::Borrow;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::str::FromStr;

/// An ID prefix: 1-8 characters of `a`-`z`, such as `user` in
/// `user_00t35pg1sg9ech32tymrkdq6v`.
///
/// Construct a `Prefix` with [`Prefix::parse`] at run time or
/// [`prefix!`](crate::prefix!) at compile time. It is a small `Copy` value
/// (the letters are stored inline), compares with strings, and orders the
/// same way IDs do.
///
/// ```
/// use souvenir::{Prefix, prefix};
///
/// const USER: Prefix = prefix!("user");
///
/// let parsed = Prefix::parse("user").unwrap();
///
/// assert_eq!(parsed, USER);
/// assert_eq!(USER, "user");
/// assert_eq!(USER.as_str(), "user");
/// assert!(Prefix::parse("User").is_err());
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Prefix([u8; MAX_PREFIX_LEN]);

impl Prefix {
    /// Parse a prefix. Fails with [`Error::InvalidPrefix`] unless it is 1-8
    /// characters of `a`-`z`.
    pub const fn parse(prefix: &str) -> Result<Self> {
        Self::from_letters(prefix.as_bytes())
    }

    /// The length of this prefix, in characters (1-8).
    // A prefix is never empty, so an `is_empty` would always return false.
    #[allow(clippy::len_without_is_empty)]
    #[must_use]
    #[inline]
    pub const fn len(self) -> usize {
        // Zero padding fills the low bytes; each one adds 8 trailing zero bits.
        let v = u64::from_be_bytes(self.0);
        MAX_PREFIX_LEN - (v.trailing_zeros() / 8) as usize
    }

    /// The rank of this prefix: its position in sort order, and the value
    /// stored in an ID's prefix field. Ranks are defined by `SPEC.md` section 4 and
    /// are the same in every implementation.
    #[must_use]
    #[inline]
    pub const fn rank(self) -> u64 {
        // Each letter adds 1 for itself plus the number of strings that sort
        // before it at its position. Padding (zero bytes) ends the prefix.
        let mut rank = 0;
        let mut i = 0;
        while i < MAX_PREFIX_LEN && self.0[i] != 0 {
            rank += 1 + (self.0[i] - b'a') as u64 * PREFIX_COUNTS[MAX_PREFIX_LEN - 1 - i];
            i += 1;
        }
        rank
    }

    /// The prefix with a given rank: the inverse of [`rank`](Self::rank).
    ///
    /// Fails with [`Error::ZeroPrefix`] for 0, and [`Error::ReservedPrefix`]
    /// for ranks past `"zzzzzzzz"`.
    ///
    /// ```
    /// use souvenir::{Error, Prefix};
    ///
    /// let user = Prefix::parse("user").unwrap();
    /// assert_eq!(Prefix::from_rank(user.rank()), Ok(user));
    /// assert_eq!(Prefix::from_rank(0), Err(Error::ZeroPrefix));
    /// ```
    #[inline]
    pub const fn from_rank(rank: u64) -> Result<Self> {
        tri!(Self::check_rank(rank));
        Ok(Self::from_valid_rank(rank))
    }

    /// The prefix as a string slice.
    #[must_use]
    #[inline]
    pub const fn as_str(&self) -> &str {
        ascii(self.as_bytes())
    }

    /// Whether `prefix` is 1-8 characters of `a`-`z`.
    #[must_use]
    pub const fn is_valid(prefix: &str) -> bool {
        Self::parse(prefix).is_ok()
    }
}

// Crate-internal: every way of making a `Prefix` is in this file.
impl Prefix {
    #[inline]
    pub(crate) const fn from_letters(letters: &[u8]) -> Result<Self> {
        if letters.is_empty() || letters.len() > MAX_PREFIX_LEN {
            return Err(Error::InvalidPrefix);
        }
        let mut bytes = [0u8; MAX_PREFIX_LEN];
        let mut i = 0;
        while i < letters.len() {
            if !letters[i].is_ascii_lowercase() {
                return Err(Error::InvalidPrefix);
            }
            bytes[i] = letters[i];
            i += 1;
        }
        Ok(Self(bytes))
    }

    /// Check that some prefix has this rank, without building it.
    #[inline]
    pub(crate) const fn check_rank(rank: u64) -> Result<()> {
        if rank == 0 {
            Err(Error::ZeroPrefix)
        } else if rank >= PREFIX_RANK_LIMIT {
            Err(Error::ReservedPrefix)
        } else {
            Ok(())
        }
    }

    #[inline]
    pub(crate) const fn from_valid_rank(rank: u64) -> Self {
        Self::from_valid_rank_with_len(rank).0
    }

    /// Also returns the length, which the loop counts anyway; `encode` is
    /// about 20% faster than with a separate [`len`](Self::len).
    #[inline]
    pub(crate) const fn from_valid_rank_with_len(mut rank: u64) -> (Self, usize) {
        let mut bytes = [0u8; MAX_PREFIX_LEN];
        let mut i = 0;
        while rank > 0 && i < MAX_PREFIX_LEN {
            rank -= 1;
            let size = PREFIX_COUNTS[MAX_PREFIX_LEN - 1 - i];
            bytes[i] = b'a' + (rank / size) as u8;
            rank %= size;
            i += 1;
        }
        (Self(bytes), i)
    }

    #[inline]
    pub(crate) const fn padded(self) -> [u8; MAX_PREFIX_LEN] {
        self.0
    }

    #[inline]
    pub(crate) const fn as_bytes(&self) -> &[u8] {
        self.0.split_at(self.len()).0
    }
}

impl fmt::Display for Prefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(self.as_str())
    }
}

impl fmt::Debug for Prefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Prefix {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

impl TryFrom<&str> for Prefix {
    type Error = Error;
    fn try_from(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

impl Hash for Prefix {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl Borrow<str> for Prefix {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for Prefix {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq<str> for Prefix {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for Prefix {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<Prefix> for str {
    fn eq(&self, other: &Prefix) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<Prefix> for &str {
    fn eq(&self, other: &Prefix) -> bool {
        *self == other.as_str()
    }
}

/// A [`Prefix`] checked at compile time: an invalid literal fails to build.
///
/// ```
/// use souvenir::{Prefix, prefix};
///
/// const USER: Prefix = prefix!("user");
/// assert_eq!(USER, "user");
/// ```
///
/// ```compile_fail
/// let bad = souvenir::prefix!("User");
/// ```
#[macro_export]
macro_rules! prefix {
    ($prefix:expr) => {{
        // A constant item, so `cargo check` reports an invalid prefix.
        const PREFIX: $crate::Prefix = match $crate::Prefix::parse($prefix) {
            ::core::result::Result::Ok(prefix) => prefix,
            ::core::result::Result::Err(e) => ::core::panic!("{}", e.message()),
        };
        PREFIX
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_valid() {
        assert!(Prefix::is_valid("user"));
        assert!(!Prefix::is_valid("User"));
        assert!(!Prefix::is_valid(""));
    }

    #[test]
    fn str_keyed_lookups() {
        use std::collections::{BTreeSet, HashSet};
        let user = Prefix::parse("user").unwrap();
        let hashed = HashSet::from([user]);
        let ordered: BTreeSet<Prefix> = ["users", "u", "user", "org"]
            .iter()
            .map(|s| s.parse().unwrap())
            .collect();
        assert!(hashed.contains("user"));
        assert!(!hashed.contains("users"));
        assert!(ordered.contains("user"));
        assert!(!ordered.contains("us"));
    }

    #[test]
    fn len_counts_letters() {
        for s in ["a", "p", "ab", "user", "zzzzzzzz", "pppppppp"] {
            assert_eq!(Prefix::parse(s).unwrap().len(), s.len(), "{s}");
        }
    }

    #[test]
    fn formatting() {
        let p = Prefix::parse("user").unwrap();
        assert_eq!(p.to_string(), "user");
        assert_eq!(format!("{p:?}"), "user");
        assert_eq!(format!("{p:>6}"), "  user");
        assert_eq!("user".parse::<Prefix>(), Ok(p));
    }
}
