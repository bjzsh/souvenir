use crate::{Generator, Id, Prefix, RandomGenerator, Result, Souvenir, Tagged};

impl Souvenir {
    /// Generate a new ID with the given prefix, using the system clock and
    /// the operating system's CSPRNG.
    ///
    /// This is stateless and safe to call from any thread. IDs with the same
    /// prefix created in the same millisecond are in random order; use a
    /// [`MonotonicGenerator`](crate::MonotonicGenerator) if that matters.
    ///
    /// Needs the `getrandom` feature (on by default).
    ///
    /// ```
    /// use souvenir::{Souvenir, prefix};
    ///
    /// let id = Souvenir::new(prefix!("user")).unwrap();
    /// assert_eq!(id.prefix(), "user");
    /// ```
    pub fn new(prefix: Prefix) -> Result<Self> {
        RandomGenerator::new().generate_untyped(prefix)
    }
}

impl<T: Tagged> Id<T> {
    /// Generate a new ID using the system clock and the operating system's
    /// CSPRNG. See [`Souvenir::new`]. Needs the `getrandom` feature (on by
    /// default).
    pub fn new() -> Result<Self> {
        Souvenir::new(T::PREFIX).map(Self::from_untyped_unchecked)
    }
}
