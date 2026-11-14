#[cfg(feature = "std")]
use crate::generator;
use crate::generator::{Clock, ClockExt, RandomSource, RandomSourceExt};
use crate::{Generator, Prefix, Result, Souvenir};

/// Generates each ID independently from the clock and random source.
///
/// It keeps no state between IDs: IDs with the same prefix created in the
/// same millisecond are in random order. Use a [`MonotonicGenerator`] when
/// they must be in the order they were created.
///
/// [`new`](RandomGenerator#method.new) reads the system clock and the
/// operating system's CSPRNG; [`with_sources`](Self::with_sources) takes your
/// own.
///
/// ```
/// # #[cfg(all(feature = "getrandom", feature = "std"))] {
/// use souvenir::{RandomGenerator, Generator, Id, tagged};
///
/// tagged!(User = "user");
///
/// let mut generator = RandomGenerator::new();
/// let id: Id<User> = generator.generate().unwrap();
/// # }
/// ```
///
/// [`MonotonicGenerator`]: crate::MonotonicGenerator
#[derive(Debug, Clone, Default)]
pub struct RandomGenerator<
    #[cfg(all(feature = "getrandom", feature = "std"))] R = generator::OsRandom,
    #[cfg(not(all(feature = "getrandom", feature = "std")))] R,
    #[cfg(feature = "std")] C = generator::SystemClock,
    #[cfg(not(feature = "std"))] C,
> {
    random: R,
    clock: C,
}

impl<R: RandomSource, C: Clock> RandomGenerator<R, C> {
    /// A generator drawing random bits from `random` and timestamps from
    /// `clock`. Closures work as either; see [`RandomSource`] and [`Clock`].
    #[must_use]
    pub const fn with_sources(random: R, clock: C) -> Self {
        Self { random, clock }
    }
}

#[cfg(all(feature = "getrandom", feature = "std"))]
impl RandomGenerator {
    /// A generator reading the system clock and the operating system's
    /// CSPRNG. Needs the `std` and `getrandom` features (on by default).
    #[must_use]
    pub const fn new() -> Self {
        Self::with_sources(generator::OsRandom, generator::SystemClock)
    }
}

impl<R: RandomSource, C: Clock> Generator for RandomGenerator<R, C> {
    fn generate_untyped(&mut self, prefix: Prefix) -> Result<Souvenir> {
        let timestamp = self.clock.now_checked()?;
        let random = self.random.random74()?;
        Souvenir::from_parts(prefix, timestamp, random)
    }
}
