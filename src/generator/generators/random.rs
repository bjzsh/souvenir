use crate::generator::{self, Clock, ClockExt, RandomSource, RandomSourceExt};
use crate::{Generator, Prefix, Result, Souvenir};

/// Generates each ID independently from the clock and random source.
///
/// It keeps no state between IDs: IDs with the same prefix created in the
/// same millisecond are in random order. Use a [`MonotonicGenerator`] when
/// they must be strictly increasing.
///
/// [`new`](RandomGenerator#method.new) reads the system clock and the
/// operating system's CSPRNG; [`with_sources`](Self::with_sources) takes your
/// own.
///
/// ```
/// # #[cfg(feature = "getrandom")] {
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
    #[cfg(feature = "getrandom")] R = generator::OsRandom,
    #[cfg(not(feature = "getrandom"))] R,
    C = generator::SystemClock,
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

#[cfg(feature = "getrandom")]
impl RandomGenerator {
    /// A generator reading the system clock and the operating system's
    /// CSPRNG. Needs the `getrandom` feature (on by default).
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
