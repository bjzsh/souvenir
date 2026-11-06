use crate::base32;
use crate::error::{Error, Result};
use crate::prefix::Prefix;
use crate::spec::{
    BODY_BITS, BODY_MASK, BYTE_LEN, MAX_TEXT_LEN, PREFIX_BITS, RANDOM_BITS, RANDOM_MAX, SUFFIX_LEN,
    TIMESTAMP_BITS, TIMESTAMP_MAX,
};
use crate::util::ascii;
use core::fmt;
use core::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// A Souvenir ID whose prefix is only known at run time.
///
/// Holds exactly 20 bytes: a 38-bit prefix rank, a 48-bit Unix-millisecond
/// timestamp, and 74 random bits, big-endian. Every value is a valid ID:
/// there is no nil value, and zeroed bytes never decode.
///
/// Ordering compares the bytes, which is the same as comparing the text forms:
/// IDs sort by prefix, then by creation time.
///
/// Prefer the typed [`Id<T>`](crate::Id) wherever the prefix is known at
/// compile time. Use `Souvenir` for storage that mixes several kinds of ID,
/// such as event logs or polymorphic references.
///
/// ```
/// use souvenir::Souvenir;
///
/// let id: Souvenir = "user_00t35pg1sg9ech32tymrkdq6v".parse().unwrap();
/// assert_eq!(id.prefix(), "user");
/// assert_eq!(id.timestamp_ms(), 1_793_531_580_000);
/// assert_eq!(id.to_string(), "user_00t35pg1sg9ech32tymrkdq6v");
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "diesel",
    derive(::diesel::AsExpression, ::diesel::FromSqlRow),
    diesel(sql_type = ::diesel::sql_types::Text),
    diesel(sql_type = ::diesel::sql_types::Binary)
)]
pub struct Souvenir([u8; BYTE_LEN]);

impl Souvenir {
    /// Length of the binary form, in bytes.
    pub const BYTE_LEN: usize = BYTE_LEN;

    /// Length of the longest text form, in bytes: an 8-character prefix,
    /// `_`, and 25 characters. [`encode`](Self::encode) writes into a buffer
    /// of this size.
    pub const MAX_TEXT_LEN: usize = MAX_TEXT_LEN;

    /// The largest timestamp an ID can hold: 2^48 - 1 Unix milliseconds, in
    /// the year 10889.
    pub const TIMESTAMP_MAX: u64 = TIMESTAMP_MAX;

    /// The largest random field an ID can hold: 2^74 - 1.
    pub const RANDOM_MAX: u128 = RANDOM_MAX;

    /// Build an ID from its parts. No randomness or clock is involved.
    ///
    /// Fails with [`Error::TimestampOutOfRange`] (`timestamp_ms >= 2^48`) or
    /// [`Error::RandomOutOfRange`] (`random >= 2^74`).
    #[inline]
    pub const fn from_parts(prefix: Prefix, timestamp_ms: u64, random: u128) -> Result<Self> {
        if timestamp_ms > TIMESTAMP_MAX {
            return Err(Error::TimestampOutOfRange);
        }
        if random > RANDOM_MAX {
            return Err(Error::RandomOutOfRange);
        }
        Ok(Self::pack(
            prefix,
            ((timestamp_ms as u128) << RANDOM_BITS) | random,
        ))
    }

    /// Parse the canonical text form (`prefix_suffix`).
    ///
    /// Parsing is strict: there is exactly one text form per ID, and no case
    /// folding, trimming or character aliasing. Checks run in the order of
    /// SPEC.md section 6.3, so every implementation reports the same error.
    pub const fn parse(text: &str) -> Result<Self> {
        let text = text.as_bytes();
        // 1. ASCII
        if !text.is_ascii() {
            return Err(Error::InvalidCharacter);
        }
        // 2. Cheap length guard
        if text.len() > Self::PARSE_GUARD_LEN {
            return Err(Error::InvalidLength);
        }
        // 3. Last separator: drop bytes from the end until one is `_`.
        let mut letters = text;
        loop {
            match letters {
                [] => return Err(Error::MissingSeparator),
                [before @ .., b'_'] => {
                    letters = before;
                    break;
                }
                [before @ .., _] => letters = before,
            }
        }
        let (_, suffix) = text.split_at(letters.len() + 1);
        // 4. Prefix
        let prefix = tri!(Prefix::from_letters(letters));
        // 5. Suffix
        let body = tri!(base32::decode(suffix));
        Ok(Self::pack(prefix, body))
    }

    /// Parse text and require a particular prefix. A valid ID with another
    /// prefix fails with [`Error::PrefixMismatch`].
    pub const fn parse_expecting(prefix: Prefix, text: &str) -> Result<Self> {
        let id = tri!(Self::parse(text));
        if id.rank() == prefix.rank() {
            Ok(id)
        } else {
            Err(Error::PrefixMismatch)
        }
    }

    /// Whether `text` is a valid ID.
    #[must_use]
    pub const fn is_valid(text: &str) -> bool {
        Self::parse(text).is_ok()
    }

    /// Decode the 20-byte binary form.
    ///
    /// Fails with [`Error::ZeroPrefix`] when the prefix field is zero
    /// (including all-zero bytes), or [`Error::ReservedPrefix`] when it is in
    /// the range reserved for future formats.
    #[inline]
    pub const fn from_bytes(bytes: [u8; BYTE_LEN]) -> Result<Self> {
        // Any 20 bytes lay out correctly; only the prefix field can be invalid.
        let id = Self(bytes);
        tri!(Prefix::check_rank(id.rank()));
        Ok(id)
    }

    /// Decode the binary form from a slice, which must be exactly 20 bytes.
    #[inline]
    pub const fn from_slice(bytes: &[u8]) -> Result<Self> {
        match bytes.first_chunk::<BYTE_LEN>() {
            Some(array) if bytes.len() == BYTE_LEN => Self::from_bytes(*array),
            _ => Err(Error::InvalidLength),
        }
    }

    /// The 20-byte big-endian binary form.
    #[must_use]
    #[inline]
    pub const fn to_bytes(self) -> [u8; BYTE_LEN] {
        self.0
    }

    /// A reference to the 20-byte big-endian binary form.
    #[must_use]
    #[inline]
    pub const fn as_bytes(&self) -> &[u8; BYTE_LEN] {
        &self.0
    }

    /// The prefix.
    #[must_use]
    #[inline]
    pub const fn prefix(&self) -> Prefix {
        Prefix::from_valid_rank(self.rank())
    }

    /// Creation time in milliseconds since the Unix epoch.
    #[must_use]
    #[inline]
    pub const fn timestamp_ms(&self) -> u64 {
        let [_, _, _, _, w0, w1, w2, w3, w4, w5, w6, w7, ..] = self.0;
        let window = u64::from_be_bytes([w0, w1, w2, w3, w4, w5, w6, w7]);
        (window >> Self::TIMESTAMP_SHIFT) & TIMESTAMP_MAX
    }

    /// The 74-bit random field.
    #[must_use]
    #[inline]
    pub const fn random(&self) -> u128 {
        self.tail() & RANDOM_MAX
    }

    /// Creation time as a [`SystemTime`]. For Unix milliseconds, use
    /// [`timestamp_ms`](Self::timestamp_ms).
    #[must_use]
    #[inline]
    pub fn timestamp(&self) -> SystemTime {
        // Can't overflow: timestamps end before the year 10890, well inside
        // `SystemTime`'s range on every platform std supports.
        UNIX_EPOCH + Duration::from_millis(self.timestamp_ms())
    }

    /// Write the text form into `buf` without allocating, and return it.
    pub const fn encode<'b>(&self, buf: &'b mut [u8; MAX_TEXT_LEN]) -> &'b str {
        let (prefix, len) = Prefix::from_valid_rank_with_len(self.rank());
        // `len` is 1-8, so the text always fits in `buf`.
        let (text, _) = buf.split_at_mut(len + 1 + SUFFIX_LEN);
        let (letters, rest) = text.split_at_mut(len);
        let (separator, suffix) = rest.split_at_mut(1);
        letters.copy_from_slice(prefix.padded().split_at(len).0);
        separator[0] = b'_';
        base32::encode(self.tail() & BODY_MASK, suffix);
        ascii(text)
    }
}

// Crate-internal: the 160-bit layout (SPEC.md section 3).
//
// `rank << 122 | timestamp << 74 | random`, big-endian in 20 bytes. Each
// field is read from a fixed window of the bytes as one big-endian integer,
// so no accessor decodes more than its own field:
//
// ```text
// bits    0        38            86                  160
//         [ rank   | timestamp   | random            ]
// [0..8)  u64  -> rank      = v >> 26
// [4..12) u64  -> timestamp = (v >> 10) & (2^48 - 1)
// [4..20) u128 -> random    = v & (2^74 - 1);  body = v & (2^122 - 1)
// ```
impl Souvenir {
    /// Bytes 0-3 hold the rank's top 32 bits; its low bits share byte 4
    /// onward with the body.
    const SPLIT: usize = 4;
    const RANK_LOW_BITS: u32 = PREFIX_BITS - 8 * Self::SPLIT as u32;

    /// Shift that brings the timestamp to the bottom of the `u64` read at
    /// byte 4.
    const TIMESTAMP_SHIFT: u32 = 64 - Self::RANK_LOW_BITS - TIMESTAMP_BITS;

    /// Longest text accepted before precise checks run (SPEC.md section 6.3 step 2).
    const PARSE_GUARD_LEN: usize = 64;

    #[inline]
    const fn pack(prefix: Prefix, body: u128) -> Self {
        Self(Self::layout(prefix.rank(), body))
    }

    /// The 20 bytes for a rank and body. Takes a raw rank so tests can reach
    /// every bit pattern.
    #[inline]
    const fn layout(rank: u64, body: u128) -> [u8; BYTE_LEN] {
        let head = ((rank >> Self::RANK_LOW_BITS) as u32).to_be_bytes();
        // The shift keeps only the rank's low bits; the high ones are in `head`.
        let tail = ((rank as u128) << BODY_BITS) | (body & BODY_MASK);
        let mut bytes = [0; BYTE_LEN];
        let (h, t) = bytes.split_at_mut(Self::SPLIT);
        h.copy_from_slice(&head);
        t.copy_from_slice(&tail.to_be_bytes());
        bytes
    }

    /// The prefix field: the prefix's rank.
    #[inline]
    pub(crate) const fn rank(&self) -> u64 {
        let [w0, w1, w2, w3, w4, w5, w6, w7, ..] = self.0;
        u64::from_be_bytes([w0, w1, w2, w3, w4, w5, w6, w7]) >> (64 - PREFIX_BITS)
    }

    /// Bytes 4..20 as a big-endian `u128`: the rank's low bits, then the
    /// body.
    #[inline]
    const fn tail(&self) -> u128 {
        let [_, _, _, _, tail @ ..] = self.0;
        u128::from_be_bytes(tail)
    }
}

// The layout's windows only work for these widths. (The array patterns in
// `rank`, `timestamp_ms` and `tail` hard-code the 4-byte split; the compiler
// checks their lengths.)
const _: () = {
    assert!(PREFIX_BITS <= 64);
    assert!(Souvenir::RANK_LOW_BITS + TIMESTAMP_BITS <= 64);
    assert!(Souvenir::RANK_LOW_BITS + BODY_BITS == 128);
    assert!(Souvenir::SPLIT + 16 == BYTE_LEN);
};

impl fmt::Display for Souvenir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(self.encode(&mut [0; MAX_TEXT_LEN]))
    }
}

impl fmt::Debug for Souvenir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.encode(&mut [0; MAX_TEXT_LEN]))
    }
}

impl FromStr for Souvenir {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

impl TryFrom<&str> for Souvenir {
    type Error = Error;
    fn try_from(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

impl TryFrom<[u8; BYTE_LEN]> for Souvenir {
    type Error = Error;
    fn try_from(bytes: [u8; BYTE_LEN]) -> Result<Self> {
        Self::from_bytes(bytes)
    }
}

impl TryFrom<&[u8]> for Souvenir {
    type Error = Error;
    fn try_from(bytes: &[u8]) -> Result<Self> {
        Self::from_slice(bytes)
    }
}

impl From<Souvenir> for [u8; BYTE_LEN] {
    fn from(id: Souvenir) -> Self {
        id.0
    }
}

impl AsRef<[u8]> for Souvenir {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_round_trip_at_the_edges() {
        let ranks = [1u64, 2, 63, 64, 65, (1 << PREFIX_BITS) - 1];
        let timestamps = [0u64, 1, 1 << 47, TIMESTAMP_MAX];
        let randoms = [0u128, 1, 1 << 73, RANDOM_MAX];
        for rank in ranks {
            for t in timestamps {
                for r in randoms {
                    let id = Souvenir(Souvenir::layout(rank, ((t as u128) << RANDOM_BITS) | r));
                    assert_eq!(id.rank(), rank);
                    assert_eq!(id.timestamp_ms(), t);
                    assert_eq!(id.random(), r);
                }
            }
        }
    }

    #[test]
    fn byte_positions() {
        // The rank's lowest bit lands in byte 4's top 6 bits.
        assert_eq!(Souvenir::layout(1, 0)[4], 0x04);
        assert_eq!(Souvenir::layout(1 << Souvenir::RANK_LOW_BITS, 0)[3], 0x01);
        // The random field's lowest bit is the last byte's lowest bit.
        assert_eq!(Souvenir::layout(0, 1)[BYTE_LEN - 1], 0x01);
        assert_eq!(
            Souvenir::layout((1 << PREFIX_BITS) - 1, BODY_MASK),
            [0xFF; BYTE_LEN]
        );
    }
}
