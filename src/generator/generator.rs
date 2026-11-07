use crate::{Id, Prefix, Result, Souvenir, Tagged};

/// Something that generates IDs.
///
/// Accept `impl Generator` (or `&mut dyn Generator`) where code needs new IDs
/// but the caller should decide how they are made: [`RandomGenerator`] for
/// independent IDs, [`MonotonicGenerator`] for strictly increasing ones, or a
/// generator of your own.
///
/// ```
/// use souvenir::{Generator, Id, MonotonicGenerator, tagged};
///
/// tagged!(Event = "event");
///
/// fn record(ids: &mut impl Generator) -> Result<Id<Event>, souvenir::Error> {
///     ids.generate()
/// }
///
/// # #[cfg(feature = "getrandom")] {
/// let mut ids = MonotonicGenerator::new();
/// let a = record(&mut ids).unwrap();
/// let b = record(&mut ids).unwrap();
/// assert!(a < b);
/// # }
/// ```
///
/// Implementing it gives exact control over IDs, for example in tests:
///
/// ```
/// use souvenir::{Generator, Prefix, Souvenir};
///
/// /// Hands out IDs from a fixed sequence of timestamps.
/// struct Scripted(std::vec::IntoIter<u64>);
///
/// impl Generator for Scripted {
///     fn generate_untyped(&mut self, prefix: Prefix) -> souvenir::Result<Souvenir> {
///         let timestamp_ms = self.0.next().expect("script ran out");
///         Souvenir::from_parts(prefix, timestamp_ms, 0)
///     }
/// }
///
/// let mut ids = Scripted(vec![1, 2].into_iter());
/// let first = ids.generate_untyped(souvenir::prefix!("user")).unwrap();
/// assert_eq!(first.timestamp_ms(), 1);
/// ```
///
/// [`RandomGenerator`]: crate::RandomGenerator
/// [`MonotonicGenerator`]: crate::MonotonicGenerator
pub trait Generator {
    /// Generate an untyped ID with the given prefix.
    fn generate_untyped(&mut self, prefix: Prefix) -> Result<Souvenir>;

    /// Generate a typed ID. `T` is usually inferred:
    /// `let id: Id<User> = generator.generate()?;`.
    fn generate<T: Tagged>(&mut self) -> Result<Id<T>>
    where
        Self: Sized,
    {
        self.generate_untyped(T::PREFIX)
            .map(Id::from_untyped_unchecked)
    }
}

impl<G: Generator + ?Sized> Generator for &mut G {
    fn generate_untyped(&mut self, prefix: Prefix) -> Result<Souvenir> {
        (**self).generate_untyped(prefix)
    }
}

impl<G: Generator + ?Sized> Generator for Box<G> {
    fn generate_untyped(&mut self, prefix: Prefix) -> Result<Souvenir> {
        (**self).generate_untyped(prefix)
    }
}
