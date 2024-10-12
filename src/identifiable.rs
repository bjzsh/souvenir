use crate::Id;

/// A type that can be used in an identifier.
pub trait Type {
    /// The prefix for this type
    const PREFIX: &'static str;
}

/// A type which can by identified with an `Id<Self>`.
pub trait Identifiable
where
    Self: Type,
{
    fn id(&self) -> Id<Self>;
}

impl<T: Identifiable> From<T> for Id<T> {
    fn from(value: T) -> Self {
        value.id()
    }
}
