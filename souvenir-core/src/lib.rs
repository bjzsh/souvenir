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

//! # souvenir_core
//!
//! This crate contains implementation details for
//! [`souvenir`](https://docs.rs/souvenir/latest/souvenir/).
//! This crate is not intended to be used directly.

pub mod encoding;
pub mod error;
pub mod id;
pub mod identifiable;
pub mod integration;
pub mod prefix;
pub mod tagged;
