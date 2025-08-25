#![forbid(unsafe_code)]

//! # souvenir
//!
//! A type-safe, tagged identifier library.
//!
//!
//! The crate primarily provides the [`Id`] struct, which stores a 128-bit
//! identifier with its corresponding type (tag).
//!
//! The String representation of an [`Id`] is the type's tag and the
//! random value encoded into a variant of
//! [Crockford Base 32](https://www.crockford.com/base32.html).
//!
//! Here is a simple example of how this crate can be used.
//!
//! ```
//! use souvenir::Id;
//!
//! let id: Id = Id::random("user".parse().unwrap());
//! println!("{}", id);
//!
//! let id2: Id = Id::parse("user_02v58c5a3fy30k560qrtg4").unwrap();
//! assert_eq!(id2.to_string(), "user_02v58c5a3fy30k560qrtg4");
//! ```
//!
//! Integrations for various libraries and databases are also (optionally)
//! available:
//! - (De)serialization with [`serde`](https://docs.rs/serde/latest/serde/)
//! - Random ID generation with [`rand`](https://docs.rs/rand/latest/rand/)
//! - Postgres, MySQL, and Sqlite support with
//!   [`sqlx`](https://docs.rs/sqlx/latest/sqlx/) and
//!   [`diesel`](https://docs.rs/diesel/latest/diesel/)

pub use souvenir_core::{
    encoding::ALPHABET, error::*, id::*, identifiable::*, prefix::*, suffix::*,
};

#[cfg(feature = "macros")]
pub use souvenir_macros::*;

/// Re-exports of the most common imports.
pub mod prelude {
    pub use crate::{Id, Identifiable, Prefix, Suffix};

    #[cfg(feature = "macros")]
    pub use crate::{id, prefix};
}
