#![forbid(unsafe_code)]
// Library code never panics at run time: no explicit panics, and no
// `unwrap`/`expect`. (Panics in `const` evaluation, such as an invalid
// `prefix!` literal, are compile errors and are fine.)
#![cfg_attr(
    not(test),
    deny(
        clippy::panic,
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::unreachable,
        clippy::todo,
        clippy::unimplemented
    )
)]
#![warn(missing_docs)]
// The crate docs are the README, so docs.rs and GitHub show one guide and its
// examples run as doctests. The examples need the default features; without
// them, a one-line summary stands in.
#![cfg_attr(
    all(feature = "getrandom", feature = "macros", feature = "serde"),
    doc = include_str!("../README.md")
)]
#![cfg_attr(
    not(all(feature = "getrandom", feature = "macros", feature = "serde")),
    doc = "Type-safe, sortable, prefixed IDs. See the README for the guide."
)]

#[macro_use]
mod util;

mod base32;
mod error;
mod id;
mod identifiable;
mod integration;
mod prefix;
mod souvenir;
mod spec;
mod storage;
mod tagged;

pub mod generator;

pub use error::{Error, Result};
pub use generator::generator::Generator;
pub use generator::generators::{MonotonicGenerator, RandomGenerator};
pub use id::Id;
pub use identifiable::Identifiable;
pub use prefix::Prefix;
pub use souvenir::Souvenir;
pub use storage::Storage;
pub use tagged::Tagged;

#[cfg(feature = "macros")]
pub use souvenir_macros::{Identifiable, Tagged, id};

/// Diesel SQL types.
#[cfg(all(feature = "diesel", feature = "postgres"))]
pub mod sql_types {
    /// The `souvenir` type from the `souvenir-postgres` extension, for
    /// `table!` columns holding a [`Souvenir`](crate::Souvenir) or an
    /// [`Id<T>`](crate::Id) whose `T` has
    /// [`Storage::Native`](crate::Storage::Native).
    ///
    /// `diesel print-schema` generates its own type for `souvenir` columns;
    /// to use this one instead, add `"souvenir::sql_types::*"` to
    /// `import_types` in `diesel.toml`.
    #[derive(
        Debug, Clone, Copy, Default, ::diesel::sql_types::SqlType, ::diesel::query_builder::QueryId,
    )]
    #[diesel(postgres_type(name = "souvenir"))]
    pub struct Souvenir;
}

/// The `uuid` crate, re-exported so `to_uuid` and `from_uuid` can be used
/// without depending on it directly (and without a version mismatch).
#[cfg(feature = "uuid")]
pub use uuid;
