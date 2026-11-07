use crate::generator::RandomSource;
use crate::{Error, Result};

/// The operating system's CSPRNG, through the `getrandom` crate.
/// Needs the `getrandom` feature (on by default).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OsRandom;

impl RandomSource for OsRandom {
    fn fill(&mut self, buf: &mut [u8]) -> Result<()> {
        getrandom::fill(buf).map_err(|_| Error::RandomSourceFailure)
    }
}
