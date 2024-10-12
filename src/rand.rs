use crate::{Id, Type};
use rand::random;

impl<T: Type> Id<T> {
    /// Generate an Id<T> with a random value
    pub fn random() -> Self {
        Self::new(random())
    }
}
