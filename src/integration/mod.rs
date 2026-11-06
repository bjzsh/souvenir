// Each module holds everything its feature adds, so the type files need no
// feature gates.

#[cfg(feature = "diesel")]
mod diesel;
#[cfg(feature = "getrandom")]
pub(crate) mod getrandom;
#[cfg(feature = "serde")]
mod serde;
#[cfg(feature = "sqlx")]
mod sqlx;
#[cfg(feature = "uuid")]
mod uuid;

#[cfg(any(
    feature = "diesel",
    all(
        feature = "sqlx",
        any(feature = "postgres", feature = "mysql", feature = "sqlite")
    )
))]
mod stored;
