use crate::{encoding::id_interval, id::Id, prefix::Prefix};
use rand::Rng;

impl Id {
    /// Generate an identifier uniformly within the selected prefix's interval.
    pub fn random(prefix: Prefix) -> Self {
        Self::random_with(prefix, &mut rand::rng())
    }

    /// Generate an identifier with the provided RNG using unbiased bounded sampling.
    pub fn random_with<R: Rng + ?Sized>(prefix: Prefix, rng: &mut R) -> Self {
        Self::from(rng.random_range(id_interval(prefix)))
    }
}
