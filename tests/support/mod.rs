//! Shared test helpers.

use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};

/// A CSPRNG as a random source, so tests run with or without the
/// `getrandom` feature.
pub fn random() -> impl FnMut(&mut [u8]) -> souvenir::Result<()> + Send {
    let mut rng = StdRng::from_os_rng();
    move |buf| {
        rng.fill_bytes(buf);
        Ok(())
    }
}
