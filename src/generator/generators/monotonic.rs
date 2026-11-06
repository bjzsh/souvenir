use crate::generator::{self, Clock, ClockExt, RandomSource, RandomSourceExt};
use crate::spec::{RANDOM_MAX, TIMESTAMP_MAX};
use crate::{Error, Generator, Prefix, Result, Souvenir};

/// Generates strictly increasing IDs, even when the clock stalls or goes
/// backwards:
///
/// - IDs created in the same millisecond (or after the clock went backwards)
///   reuse the last timestamp and add a random step of 1 to 2^32 to the
///   random field, so they stay hard to guess.
/// - If the random field would overflow, the timestamp moves forward by one
///   millisecond. Under bursts or clock rollback, timestamps can therefore
///   run slightly ahead of the wall clock.
/// - The guarantee holds per generator, across all prefixes. Across
///   generators, IDs from the same millisecond are in arbitrary order.
///
/// Its IDs are indistinguishable from a [`RandomGenerator`]'s. To share one
/// generator between threads, put it in a `Mutex`.
///
/// ```
/// # #[cfg(feature = "getrandom")] {
/// use souvenir::{Generator, MonotonicGenerator, prefix};
///
/// let mut generator = MonotonicGenerator::new();
/// let a = generator.generate_untyped(prefix!("event")).unwrap();
/// let b = generator.generate_untyped(prefix!("event")).unwrap();
/// assert!(a < b);
/// # }
/// ```
///
/// [`RandomGenerator`]: crate::RandomGenerator
// Not `Clone`: two copies would continue from the same state.
#[derive(Debug, Default)]
pub struct MonotonicGenerator<
    #[cfg(feature = "getrandom")] R = generator::OsRandom,
    #[cfg(not(feature = "getrandom"))] R,
    C = generator::SystemClock,
> {
    random: R,
    clock: C,
    last: Option<(u64, u128)>,
}

impl<R: RandomSource, C: Clock> MonotonicGenerator<R, C> {
    /// A generator drawing random bits from `random` and timestamps from
    /// `clock`. Closures work as either; see [`RandomSource`] and [`Clock`].
    #[must_use]
    pub const fn with_sources(random: R, clock: C) -> Self {
        Self {
            random,
            clock,
            last: None,
        }
    }
}

#[cfg(feature = "getrandom")]
impl MonotonicGenerator {
    /// A generator reading the system clock and the operating system's
    /// CSPRNG. Needs the `getrandom` feature (on by default).
    #[must_use]
    pub const fn new() -> Self {
        Self::with_sources(generator::OsRandom, generator::SystemClock)
    }
}

impl<R: RandomSource, C: Clock> Generator for MonotonicGenerator<R, C> {
    fn generate_untyped(&mut self, prefix: Prefix) -> Result<Souvenir> {
        let now = self.clock.now_checked()?;

        let (new_timestamp, new_random) = match self.last {
            Some((timestamp, random)) if now <= timestamp => {
                let new_random = random + self.random.random_step()?;

                if new_random <= RANDOM_MAX {
                    (timestamp, new_random)
                } else if timestamp < TIMESTAMP_MAX {
                    (timestamp + 1, self.random.random74()?)
                } else {
                    return Err(Error::TimestampOutOfRange);
                }
            }
            _ => (now, self.random.random74()?),
        };

        self.last = Some((new_timestamp, new_random));
        Souvenir::from_parts(prefix, new_timestamp, new_random)
    }
}
