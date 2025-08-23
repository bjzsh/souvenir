#![forbid(unsafe_code)]

//! # souvenir_macros
//!
//! This crate contains procedural macros for
//! [`souvenir`](https://docs.rs/souvenir/latest/souvenir/).
//! This crate is not intended to be used directly.

extern crate proc_macro;

mod id;

use proc_macro::TokenStream;

/// Create an `Id` based on some literal input.
/// All inputs are verified at compile time to ensure that the `Id` is valid.
///
/// If the full string representation of an `Id` is provided, it is parsed.
/// If only a prefix is provided, a random `Id` with the provided prefix will
/// be generated at runtime.
///
/// ```
/// # use souvenir_core::id::Id;
/// # use souvenir_macros::id;
/// let id: Id = id!("user_02v58c5a3fy30k560qrtg4");
/// assert_eq!(id, "user_02v58c5a3fy30k560qrtg4".parse().unwrap());
///
/// let id2: Id = id!("user");
/// assert_eq!(id.prefix(), "user");
/// ```
#[proc_macro]
pub fn id(input: TokenStream) -> TokenStream {
    id::id(input)
}
