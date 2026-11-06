use crate::error::{Error, Result};
use crate::prefix::Prefix;
use crate::souvenir::Souvenir;
use crate::spec::{BYTE_LEN, MAX_TEXT_LEN};
use crate::tagged::Tagged;
use core::borrow::Borrow;
use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::str::FromStr;
use std::time::SystemTime;

/// An ID whose prefix is fixed by the type `T`, usually the record it
/// identifies.
///
/// `Id<T>` is a zero-cost wrapper around [`Souvenir`] that can only hold IDs
/// with prefix `T::PREFIX`, so an `Id<User>` can never be passed where an
/// `Id<Org>` is expected. `T` implements [`Tagged`], usually with
/// `#[derive(Tagged)]` on the record type, or is declared with
/// [`tagged!`](crate::tagged!).
///
/// ```
/// use souvenir::{Id, tagged};
///
/// tagged!(User = "user");
///
/// let id: Id<User> = "user_00t35pg1sg9ech32tymrkdq6v".parse().unwrap();
/// assert_eq!(id.prefix(), "user");
/// assert!("org_00t35pg1sg9ech32tymrkdq6v".parse::<Id<User>>().is_err());
/// ```
#[repr(transparent)]
#[cfg_attr(
    feature = "diesel",
    derive(::diesel::AsExpression, ::diesel::FromSqlRow),
    diesel(sql_type = ::diesel::sql_types::Text),
    diesel(sql_type = ::diesel::sql_types::Binary)
)]
#[cfg_attr(
    all(feature = "diesel", feature = "postgres"),
    diesel(sql_type = ::diesel::sql_types::Uuid)
)]
pub struct Id<T: Tagged> {
    inner: Souvenir,
    marker: PhantomData<fn() -> T>,
}

impl<T: Tagged> Id<T> {
    /// Build an ID from its parts. See [`Souvenir::from_parts`].
    pub const fn from_parts(timestamp_ms: u64, random: u128) -> Result<Self> {
        let inner = tri!(Souvenir::from_parts(T::PREFIX, timestamp_ms, random));
        Ok(Self::from_untyped_unchecked(inner))
    }

    /// Parse the text form. Fails with [`Error::PrefixMismatch`] if the ID
    /// is valid but has another prefix.
    pub const fn parse(text: &str) -> Result<Self> {
        Self::check(tri!(Souvenir::parse(text)))
    }

    /// Whether `text` is a valid ID with prefix `T::PREFIX`.
    #[must_use]
    pub const fn is_valid(text: &str) -> bool {
        Self::parse(text).is_ok()
    }

    /// Decode the 20-byte binary form. See [`Souvenir::from_bytes`].
    #[inline]
    pub const fn from_bytes(bytes: [u8; BYTE_LEN]) -> Result<Self> {
        Self::check(tri!(Souvenir::from_bytes(bytes)))
    }

    /// Decode the binary form from a slice of exactly 20 bytes.
    #[inline]
    pub const fn from_slice(bytes: &[u8]) -> Result<Self> {
        Self::check(tri!(Souvenir::from_slice(bytes)))
    }

    /// The 20-byte big-endian binary form.
    #[must_use]
    #[inline]
    pub const fn to_bytes(self) -> [u8; BYTE_LEN] {
        self.inner.to_bytes()
    }

    /// A reference to the 20-byte big-endian binary form.
    #[must_use]
    #[inline]
    pub const fn as_bytes(&self) -> &[u8; BYTE_LEN] {
        self.inner.as_bytes()
    }

    /// The untyped ID.
    #[must_use]
    #[inline]
    pub const fn to_untyped(self) -> Souvenir {
        self.inner
    }

    /// The prefix, `T::PREFIX`.
    #[must_use]
    #[inline]
    pub const fn prefix(&self) -> Prefix {
        T::PREFIX
    }

    /// Creation time in milliseconds since the Unix epoch.
    #[must_use]
    #[inline]
    pub const fn timestamp_ms(&self) -> u64 {
        self.inner.timestamp_ms()
    }

    /// The 74-bit random field.
    #[must_use]
    #[inline]
    pub const fn random(&self) -> u128 {
        self.inner.random()
    }

    /// Creation time as a [`SystemTime`]. For Unix milliseconds, use
    /// [`timestamp_ms`](Self::timestamp_ms).
    #[must_use]
    #[inline]
    pub fn timestamp(&self) -> SystemTime {
        self.inner.timestamp()
    }

    /// Write the text form into `buf` without allocating, and return it.
    pub const fn encode<'b>(&self, buf: &'b mut [u8; MAX_TEXT_LEN]) -> &'b str {
        self.inner.encode(buf)
    }
}

// Crate-internal
impl<T: Tagged> Id<T> {
    /// Wrap an untyped ID already known to have prefix `T::PREFIX`.
    #[inline]
    pub(crate) const fn from_untyped_unchecked(inner: Souvenir) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
    }

    /// Wrap an untyped ID, failing with [`Error::PrefixMismatch`] unless its
    /// prefix is `T::PREFIX`.
    const fn check(inner: Souvenir) -> Result<Self> {
        if inner.rank() == T::PREFIX.rank() {
            Ok(Self::from_untyped_unchecked(inner))
        } else {
            Err(Error::PrefixMismatch)
        }
    }
}

// Hand-written so that `T` (often a record type) needs no extra bounds.
impl<T: Tagged> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Tagged> Copy for Id<T> {}

impl<T: Tagged> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T: Tagged> Eq for Id<T> {}

impl<T: Tagged> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Tagged> Ord for Id<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl<T: Tagged> Hash for Id<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl<T: Tagged> PartialEq<Souvenir> for Id<T> {
    fn eq(&self, other: &Souvenir) -> bool {
        self.inner == *other
    }
}

impl<T: Tagged> PartialEq<Id<T>> for Souvenir {
    fn eq(&self, other: &Id<T>) -> bool {
        *self == other.inner
    }
}

impl<T: Tagged> fmt::Display for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl<T: Tagged> fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.encode(&mut [0; MAX_TEXT_LEN]))
    }
}

impl<T: Tagged> FromStr for Id<T> {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

impl<T: Tagged> TryFrom<&str> for Id<T> {
    type Error = Error;
    fn try_from(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

impl<T: Tagged> TryFrom<Souvenir> for Id<T> {
    type Error = Error;
    fn try_from(id: Souvenir) -> Result<Self> {
        Self::check(id)
    }
}

impl<T: Tagged> TryFrom<[u8; BYTE_LEN]> for Id<T> {
    type Error = Error;
    fn try_from(bytes: [u8; BYTE_LEN]) -> Result<Self> {
        Self::from_bytes(bytes)
    }
}

impl<T: Tagged> TryFrom<&[u8]> for Id<T> {
    type Error = Error;
    fn try_from(bytes: &[u8]) -> Result<Self> {
        Self::from_slice(bytes)
    }
}

impl<T: Tagged> From<Id<T>> for Souvenir {
    fn from(id: Id<T>) -> Self {
        id.inner
    }
}

impl<T: Tagged> From<Id<T>> for [u8; BYTE_LEN] {
    fn from(id: Id<T>) -> Self {
        id.to_bytes()
    }
}

impl<T: Tagged> AsRef<[u8]> for Id<T> {
    fn as_ref(&self) -> &[u8] {
        self.inner.as_ref()
    }
}

/// Look up maps and sets keyed by `Id<T>` with an untyped [`Souvenir`], such
/// as one read from a mixed-kind event log. Equality, ordering and hashing
/// are the same for both types; an ID with another prefix is simply absent.
impl<T: Tagged> Borrow<Souvenir> for Id<T> {
    fn borrow(&self) -> &Souvenir {
        &self.inner
    }
}
