use crate::{Id, Identifiable};
use rand::random;

impl<T: Identifiable> Id<T> {
    /// Generate an Id<T> with a random value
    pub fn random() -> Self {
        Self::new(random())
    }
}
