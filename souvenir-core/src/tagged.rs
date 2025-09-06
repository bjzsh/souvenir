use crate::prefix::Prefix;

/// A constant, tagged prefix.
pub trait Tagged {
    /// The prefix associated with this type.
    const PREFIX: Prefix;
}
