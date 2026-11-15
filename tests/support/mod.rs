//! Shared test helpers.

use rand::Rng;
use rand::rngs::StdRng;

/// A CSPRNG as a random source, so tests run with or without the
/// `getrandom` feature.
pub fn random() -> impl FnMut(&mut [u8]) -> souvenir::Result<()> + Send {
    let mut rng = rand::make_rng::<StdRng>();
    move |buf| {
        rng.fill_bytes(buf);
        Ok(())
    }
}
