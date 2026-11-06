use crate::{Error, Result, generator::Clock};

/// The system's wall clock.
///
/// `wasm32-unknown-unknown` has no system clock, so there every reading
/// fails with [`Error::ClockFailure`]. Give generators a clock of your own on
/// that target, for example one reading JavaScript's `Date.now()`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SystemClock;

impl Clock for SystemClock {
    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
    fn now_ms(&mut self) -> Result<u64> {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Error::ClockFailure)?
            .as_millis()
            .try_into()
            .map_err(|_| Error::ClockFailure)
    }

    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    fn now_ms(&mut self) -> Result<u64> {
        Err(Error::ClockFailure)
    }
}
