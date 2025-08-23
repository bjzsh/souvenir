use crate::id::Id;

/// A type which can by identified with an [`Id`].
pub trait Identifiable {
    /// Retrieve an [`Id`] from this [`Identifiable`].
    fn id(&self) -> Id;
}

impl<T: Identifiable> From<T> for Id {
    fn from(value: T) -> Self {
        value.id()
    }
}
