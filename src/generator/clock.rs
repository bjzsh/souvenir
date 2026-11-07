use crate::error::{Error, Result};
use crate::spec::TIMESTAMP_MAX;

/// Where a generator gets each ID's timestamp, in Unix milliseconds.
///
/// [`SystemClock`](super::SystemClock) is the default. Any closure returning
/// `Result<u64>` is a clock too, which lets tests fix or step the time:
///
/// ```
/// use souvenir::{Generator, MonotonicGenerator, prefix};
///
/// let mut ticks = 1_793_531_580_000;
/// let clock = move || {
///     ticks += 1;
///     Ok(ticks)
/// };
/// let zeros = |buf: &mut [u8]| {
///     buf.fill(0);
///     Ok(())
/// };
/// let mut generator = MonotonicGenerator::with_sources(zeros, clock);
/// let id = generator.generate_untyped(prefix!("user")).unwrap();
/// assert_eq!(id.timestamp_ms(), 1_793_531_580_001);
/// ```
pub trait Clock {
    /// The current time in milliseconds since the Unix epoch.
    ///
    /// Return [`Error::ClockFailure`] if the clock can't be read. Generation
    /// also fails with it when a reading doesn't fit the 48-bit timestamp
    /// field (2^48 ms or more, past the year 10889).
    fn now_ms(&mut self) -> Result<u64>;
}

impl<F: FnMut() -> Result<u64>> Clock for F {
    fn now_ms(&mut self) -> Result<u64> {
        self()
    }
}

pub(crate) trait ClockExt {
    fn now_checked(&mut self) -> Result<u64>;
}

impl<T: Clock> ClockExt for T {
    fn now_checked(&mut self) -> Result<u64> {
        let t = self.now_ms()?;
        if t > TIMESTAMP_MAX {
            return Err(Error::ClockFailure);
        }
        Ok(t)
    }
}
