use crate::{id::Id, prefix::Prefix, suffix::Suffix};
use rand::{Rng, random};

impl Id {
    /// Generate an [`Id`] with a random value.
    pub fn random(prefix: Prefix) -> Self {
        Self::new(prefix, Suffix::random())
    }

    /// Generate a random [`Id`] with the provided RNG.
    pub fn random_with<R: Rng + ?Sized>(prefix: Prefix, rng: &mut R) -> Self {
        Self::new(prefix, Suffix::random_with(rng))
    }
}

impl Suffix {
    /// Generate a [`Suffix`] with a random value.
    pub fn random() -> Self {
        Self::new(random())
    }

    /// Generate a random [`Suffix`] with the provided RNG.
    pub fn random_with<R: Rng + ?Sized>(rng: &mut R) -> Self {
        Self::new(rng.random())
    }
}
