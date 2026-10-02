#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unimplemented,
        clippy::unreachable
    )
)]

//! JavaScript bindings for Souvenir identifiers.

mod error;
mod id;
mod interop;

pub use id::Id;
pub use interop::{BigIntInput, BytesInput, StringInput, UnknownInput};
