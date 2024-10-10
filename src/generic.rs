use crate::{Id, Identifiable};

pub struct Generic;

impl Identifiable for Generic {
    const PREFIX: &'static str = "flake";
}

impl Id<Generic> {
    pub fn cast<T: Identifiable>(self) -> Id<T> {
        Id::new(self.value())
    }
}
