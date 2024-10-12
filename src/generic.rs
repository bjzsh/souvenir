use crate::{Id, Type};

pub struct Generic;

impl Type for Generic {
    const PREFIX: &'static str = "flake";
}

impl Id<Generic> {
    pub fn cast<T: Type>(self) -> Id<T> {
        self.to_bytes().into()
    }
}
