use crate::{error::Error, id::Id};
use rand::{Rng, random};

impl Id {
    /// Generate an [`Id`] with a random value.
    pub fn random(prefix: &str) -> Result<Self, Error> {
        Self::from_parts(prefix, random())
    }

    /// Generate a random [`Id`] with the provided RNG.
    pub fn random_with<R: Rng + ?Sized>(prefix: &str, rng: &mut R) -> Result<Self, Error> {
        Self::from_parts(prefix, rng.random())
    }
}
