#[cfg(feature = "getrandom")]
mod os_random;
#[cfg(feature = "std")]
mod system_clock;

#[cfg(feature = "getrandom")]
pub use os_random::OsRandom;
#[cfg(feature = "std")]
pub use system_clock::SystemClock;
