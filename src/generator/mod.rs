//! The clocks and random sources that [`RandomGenerator`] and
//! [`MonotonicGenerator`] read from.
//!
//! By default, generators read the [`SystemClock`] and, with the `getrandom`
//! feature, `OsRandom`. Both traits are implemented for closures, so a test
//! can pass fixed values to `with_sources` without writing a type.
//!
//! [`RandomGenerator`]: crate::RandomGenerator
//! [`MonotonicGenerator`]: crate::MonotonicGenerator

#[allow(clippy::module_inception)]
pub(crate) mod generator;
pub(crate) mod generators;

mod clock;
mod providers;
mod source;

pub use clock::Clock;
pub(crate) use clock::ClockExt;

#[cfg(feature = "getrandom")]
pub use providers::OsRandom;
pub use providers::SystemClock;

pub use source::RandomSource;
pub(crate) use source::RandomSourceExt;
