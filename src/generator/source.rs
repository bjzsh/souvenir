use crate::Result;
use crate::spec::RANDOM_MAX;

/// Where a generator gets its random bits.
///
/// The random bits make IDs unguessable and keep them from colliding, so in
/// production this must be a cryptographically secure generator, such as
/// `OsRandom` (the default, with the `getrandom` feature).
///
/// Any closure taking `&mut [u8]` and returning `Result<()>` is a source too,
/// which lets tests fix the random bits:
///
/// ```
/// use souvenir::{Generator, RandomGenerator, prefix};
///
/// let zeros = |buf: &mut [u8]| {
///     buf.fill(0);
///     Ok(())
/// };
/// let mut generator = RandomGenerator::with_sources(zeros, || Ok(1_793_531_580_000));
/// let id = generator.generate_untyped(prefix!("user")).unwrap();
/// assert_eq!(id.random(), 0);
/// ```
pub trait RandomSource {
    /// Fill all of `buf` with random bytes.
    ///
    /// If that isn't possible, return
    /// [`Error::RandomSourceFailure`](crate::Error::RandomSourceFailure), and
    /// generation fails with it. Never leave `buf` partly filled or fall back
    /// to predictable bytes: the generator would turn them into IDs.
    fn fill(&mut self, buf: &mut [u8]) -> Result<()>;
}

impl<F: FnMut(&mut [u8]) -> Result<()>> RandomSource for F {
    fn fill(&mut self, buf: &mut [u8]) -> Result<()> {
        self(buf)
    }
}

pub(crate) trait RandomSourceExt {
    fn random74(&mut self) -> Result<u128>;
    fn random_step(&mut self) -> Result<u128>;
}

impl<T: RandomSource> RandomSourceExt for T {
    fn random74(&mut self) -> Result<u128> {
        let mut buf = [0u8; 16];
        self.fill(&mut buf[6..])?;
        Ok(u128::from_be_bytes(buf) & RANDOM_MAX)
    }

    fn random_step(&mut self) -> Result<u128> {
        let mut buf = [0u8; 4];
        self.fill(&mut buf)?;
        Ok(1 + u128::from(u32::from_be_bytes(buf)))
    }
}
