/// A type that can be used in an identifier.
pub trait Identifiable {
    /// The prefix for this type
    const PREFIX: &'static str;
}
