#![forbid(unsafe_code)]

//! # souvenir_macros
//!
//! This crate contains procedural macros for
//! [`souvenir`](https://docs.rs/souvenir/latest/souvenir/).
//! This crate is not intended to be used directly.

extern crate proc_macro;

mod id;
mod identifiable;
mod prefix;

use proc_macro::TokenStream;

/// Create an `Id` based on some literal input.
/// All inputs are verified at compile time to ensure that the `Id` is valid.
///
/// If the full string representation of an `Id` is provided, it is parsed.
/// If only a prefix is provided, a random `Id` with the provided prefix will
/// be generated at runtime.
///
/// ```
/// # use souvenir::{id, Id};
/// let id: Id = id!("user_02v58c5a3fy30k560qrtg4");
/// assert_eq!(id, "user_02v58c5a3fy30k560qrtg4".parse().unwrap());
///
/// let id2: Id = id!("user");
/// assert_eq!(id.prefix().to_string(), "user");
/// ```
#[proc_macro]
pub fn id(input: TokenStream) -> TokenStream {
    id::id(input)
}

/// Create a `Prefix` based on some literal input.
/// All inputs are verified at compile time to ensure that the `Prefix` is
/// valid.
///
/// ```
/// # use souvenir::{prefix, Id, Prefix};
/// let prefix: Prefix = prefix!("hi");
/// assert_eq!(prefix.to_string(), "hi");
///
/// let id: Id = Id::random(prefix);
/// assert_eq!(id.prefix(), prefix);
/// ```
#[proc_macro]
pub fn prefix(input: TokenStream) -> TokenStream {
    prefix::prefix(input)
}

/// Automatically implement `Identifiable`.
///
/// ```
/// # use souvenir::{id, Id, Identifiable};
/// #[derive(Identifiable)]
/// struct User {
///     #[souvenir(id)]
///     id: Id,
/// }
///
/// let user = User { id: id!("user") };
/// assert_eq!(user.id, user.id());
/// ```
#[proc_macro_derive(Identifiable, attributes(souvenir))]
pub fn identifiable(input: TokenStream) -> TokenStream {
    identifiable::identifiable(input)
}
