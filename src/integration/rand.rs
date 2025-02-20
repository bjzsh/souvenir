use crate::{Id, Type};
use rand::{random, Rng};

impl<T: Type> Id<T> {
    /// Generate an [`Id<T>`] with a random value.
    pub fn random() -> Self {
        Self::new(random())
    }

    /// Generate a random [`Id<T>`] with the provided RNG.
    pub fn random_with<R: Rng + ?Sized>(rng: &mut R) -> Self {
        Self::new(rng.random())
    }
}
