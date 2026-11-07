use core::fmt;

/// A convenience alias for `Result<T, souvenir::Error>`.
pub type Result<T> = core::result::Result<T, Error>;

/// Every error this crate can produce.
///
/// Each variant has a [`code`](Error::code): a stable name such as
/// `"PrefixMismatch"`, defined by the Souvenir specification and reported
/// identically by every implementation (the JavaScript package throws errors
/// with the same codes). Match on the variant, or compare codes across
/// languages; don't parse [`message`](Error::message), whose wording may
/// change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Error {
    /// A prefix is empty, longer than 8 characters, or contains anything
    /// other than `a`-`z`.
    InvalidPrefix,
    /// Text has no `_` separator.
    MissingSeparator,
    /// Input has the wrong length (suffix, bytes, UUID, or text over 64 chars).
    InvalidLength,
    /// Text is not ASCII, or a suffix character is outside the alphabet.
    InvalidCharacter,
    /// The suffix encodes a value of 2^122 or more (first character above `3`).
    NonCanonical,
    /// The binary prefix field is zero. Zeroed bytes are never an ID.
    ZeroPrefix,
    /// The binary prefix field is in the range reserved for future formats.
    ReservedPrefix,
    /// The ID has a different prefix than the one expected.
    PrefixMismatch,
    /// A timestamp is 2^48 or more, including a monotonic generator running
    /// past the end of the range.
    TimestampOutOfRange,
    /// A random value is 2^74 or more.
    RandomOutOfRange,
    /// A UUID does not have version 7 and variant `0b10`.
    NotUuidV7,
    /// The random source failed.
    RandomSourceFailure,
    /// The clock is unavailable, or its reading is not a timestamp in
    /// `[0, 2^48)` (for example, before 1970).
    ClockFailure,
}

impl Error {
    /// The stable error code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidPrefix => "InvalidPrefix",
            Self::MissingSeparator => "MissingSeparator",
            Self::InvalidLength => "InvalidLength",
            Self::InvalidCharacter => "InvalidCharacter",
            Self::NonCanonical => "NonCanonical",
            Self::ZeroPrefix => "ZeroPrefix",
            Self::ReservedPrefix => "ReservedPrefix",
            Self::PrefixMismatch => "PrefixMismatch",
            Self::TimestampOutOfRange => "TimestampOutOfRange",
            Self::RandomOutOfRange => "RandomOutOfRange",
            Self::NotUuidV7 => "NotUuidV7",
            Self::RandomSourceFailure => "RandomSourceFailure",
            Self::ClockFailure => "ClockFailure",
        }
    }

    /// A human-readable description of the error.
    #[must_use]
    pub const fn message(&self) -> &'static str {
        match self {
            Self::InvalidPrefix => "prefix must be 1-8 characters of a-z",
            Self::MissingSeparator => "ID text has no '_' separator",
            Self::InvalidLength => "input has the wrong length",
            Self::InvalidCharacter => "input contains an invalid character",
            Self::NonCanonical => "suffix is not canonical (first character must be 0-3)",
            Self::ZeroPrefix => "prefix field is zero, which is not any prefix",
            Self::ReservedPrefix => "prefix field is in the range reserved for future formats",
            Self::PrefixMismatch => "ID has a different prefix than expected",
            Self::TimestampOutOfRange => "timestamp must fit in 48 bits of Unix milliseconds",
            Self::RandomOutOfRange => "random value must fit in 74 bits",
            Self::NotUuidV7 => "UUID is not version 7 with variant 0b10",
            Self::RandomSourceFailure => "the random number source failed",
            Self::ClockFailure => "the clock is unavailable or out of range",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for Error {}
