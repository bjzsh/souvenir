use crate::id::Id;
use crate::tagged::Tagged;

/// A value identified by a typed [`Id`].
///
/// Usually derived with `#[derive(Identifiable)]` (the `macros` feature),
/// which uses the field named `id`, or the field marked `#[souvenir(id)]`.
pub trait Identifiable {
    /// The [`Tagged`] type of this value's ID: `T` in `Id<T>`.
    type Tag: Tagged;

    /// This value's ID.
    fn id(&self) -> Id<Self::Tag>;
}

impl<T: Tagged> Identifiable for Id<T> {
    type Tag = T;

    fn id(&self) -> Id<T> {
        *self
    }
}
