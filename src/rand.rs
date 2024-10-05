use crate::{Id, Identifiable};
use rand::random;

impl<T: Identifiable> Id<T> {
    pub fn random() -> Self {
        Self::new(random())
    }
}
