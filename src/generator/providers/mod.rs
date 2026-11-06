#[cfg(feature = "getrandom")]
mod os_random;
mod system_clock;

#[cfg(feature = "getrandom")]
pub use os_random::OsRandom;
pub use system_clock::SystemClock;
