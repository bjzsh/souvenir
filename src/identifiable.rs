use crate::Id;

/// A type that can be used in an identifier.
pub trait Type {
    const PREFIX: &'static str;
}

/// A type which can by identified with an `Id<Self>`.
pub trait Identifiable
where
    Self::Output: Type,
{
    type Output;
    fn id(&self) -> Id<Self::Output>;
}

impl<T: Type, U: Identifiable<Output = T>> From<U> for Id<T> {
    fn from(value: U) -> Self {
        value.id()
    }
}
