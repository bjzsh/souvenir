use crate::interop;
use souvenir::generator::{Clock, OsRandom};
use souvenir::{Error, MonotonicGenerator, RandomGenerator};
use std::sync::Mutex;

/// `Date.now()`, rejecting values that aren't a valid timestamp.
#[derive(Debug, Clone, Copy)]
pub(crate) struct JsClock;

impl Clock for JsClock {
    fn now_ms(&mut self) -> souvenir::Result<u64> {
        let ms = interop::now();
        if ms.is_finite() && ms >= 0.0 {
            Ok(ms as u64)
        } else {
            Err(Error::ClockFailure)
        }
    }
}

/// Backs `newId`. It keeps no state, so each call gets its own.
pub(crate) const fn random_generator() -> RandomGenerator<OsRandom, JsClock> {
    RandomGenerator::with_sources(OsRandom, JsClock)
}

/// Backs `newMonotonicId`: one generator for the whole module.
pub(crate) static MONOTONIC: Mutex<MonotonicGenerator<OsRandom, JsClock>> =
    Mutex::new(MonotonicGenerator::with_sources(OsRandom, JsClock));
