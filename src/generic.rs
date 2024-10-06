use crate::{Id, Identifiable};

pub struct Generic;

impl Identifiable for Generic {
    fn prefix() -> &'static str {
        "flake"
    }
}

impl Id<Generic> {
    pub fn cast<T: Identifiable>(self) -> Id<T> {
        Id::new(self.value)
    }
}
