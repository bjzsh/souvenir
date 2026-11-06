//! Generator tests. Clocks are always injected, except in the
//! smoke tests of the system-clock entry points.

mod support;

use souvenir::{
    Error, Generator, Id, MonotonicGenerator, RandomGenerator, Souvenir, prefix, tagged,
};
use support::random;

tagged!(User = "user");

/// A clock that returns `values` in order, then repeats the last one.
fn sequence_clock(values: Vec<u64>) -> impl FnMut() -> souvenir::Result<u64> {
    let mut index = 0;
    move || {
        let t = values[index.min(values.len() - 1)];
        index += 1;
        Ok(t)
    }
}

/// A random source that fills every buffer with `byte`.
fn constant(byte: u8) -> impl FnMut(&mut [u8]) -> souvenir::Result<()> {
    move |buf| {
        buf.fill(byte);
        Ok(())
    }
}

const FIXED_MS: u64 = 1_793_531_580_000;

#[test]
fn monotonic_survives_stalls_and_rollback() {
    let clock = vec![1000, 1000, 1000, 999, 998, 1001, 1001, 1001, 500, 1002];
    let mut generator = MonotonicGenerator::with_sources(random(), sequence_clock(clock));
    let ids: Vec<Souvenir> = (0..1_010)
        .map(|_| generator.generate_untyped(prefix!("user")).unwrap())
        .collect();
    for pair in ids.windows(2) {
        assert!(pair[0] < pair[1], "{:?} !< {:?}", pair[0], pair[1]);
        assert!(pair[0].timestamp_ms() <= pair[1].timestamp_ms());
    }
    // Rolled-back readings keep the last emitted timestamp.
    assert_eq!(ids[3].timestamp_ms(), 1000);
    assert_eq!(ids[4].timestamp_ms(), 1000);
    assert_eq!(ids[5].timestamp_ms(), 1001);
    assert_eq!(ids[8].timestamp_ms(), 1001);
}

#[test]
fn monotonic_overflow_borrows_next_millisecond() {
    // A source of all-ones makes every 74-bit draw `RANDOM_MAX`, so the second
    // ID in a millisecond must overflow.
    let mut generator = MonotonicGenerator::with_sources(constant(0xFF), || Ok(1000));
    let a = generator.generate_untyped(prefix!("user")).unwrap();
    let b = generator.generate_untyped(prefix!("user")).unwrap();
    let c = generator.generate_untyped(prefix!("user")).unwrap();
    assert_eq!((a.timestamp_ms(), a.random()), (1000, Souvenir::RANDOM_MAX));
    assert_eq!((b.timestamp_ms(), b.random()), (1001, Souvenir::RANDOM_MAX));
    assert_eq!((c.timestamp_ms(), c.random()), (1002, Souvenir::RANDOM_MAX));
    assert!(a < b && b < c);
}

#[test]
fn monotonic_overflow_at_end_of_time_fails() {
    let mut generator =
        MonotonicGenerator::with_sources(constant(0xFF), || Ok(Souvenir::TIMESTAMP_MAX));
    generator.generate_untyped(prefix!("user")).unwrap();
    assert_eq!(
        generator.generate_untyped(prefix!("user")),
        Err(Error::TimestampOutOfRange)
    );
}

#[test]
fn random_source_failure_is_reported_never_hidden() {
    let failing = |_: &mut [u8]| Err(Error::RandomSourceFailure);
    assert_eq!(
        RandomGenerator::with_sources(failing, || Ok(1000)).generate_untyped(prefix!("user")),
        Err(Error::RandomSourceFailure)
    );
    assert_eq!(
        MonotonicGenerator::with_sources(failing, || Ok(1000)).generate_untyped(prefix!("user")),
        Err(Error::RandomSourceFailure)
    );
}

#[test]
fn random_source_failure_on_monotonic_step_is_reported() {
    let mut calls = 0;
    let fails_after_first = move |buf: &mut [u8]| {
        calls += 1;
        if calls == 1 {
            buf.fill(0);
            Ok(())
        } else {
            Err(Error::RandomSourceFailure)
        }
    };
    let mut generator = MonotonicGenerator::with_sources(fails_after_first, || Ok(1000));
    generator.generate_untyped(prefix!("user")).unwrap();
    assert_eq!(
        generator.generate_untyped(prefix!("user")),
        Err(Error::RandomSourceFailure)
    );
}

#[test]
fn clock_failures() {
    let user = prefix!("user");
    assert_eq!(
        RandomGenerator::with_sources(random(), || Ok(Souvenir::TIMESTAMP_MAX + 1))
            .generate_untyped(user),
        Err(Error::ClockFailure)
    );
    assert_eq!(
        MonotonicGenerator::with_sources(random(), || Ok(Souvenir::TIMESTAMP_MAX + 1))
            .generate_untyped(user),
        Err(Error::ClockFailure)
    );
    assert_eq!(
        RandomGenerator::with_sources(random(), || Err(Error::ClockFailure)).generate_untyped(user),
        Err(Error::ClockFailure)
    );
    assert_eq!(
        MonotonicGenerator::with_sources(random(), || Err(Error::ClockFailure))
            .generate_untyped(user),
        Err(Error::ClockFailure)
    );
    let id = RandomGenerator::with_sources(random(), || Ok(Souvenir::TIMESTAMP_MAX))
        .generate_untyped(user)
        .unwrap();
    assert_eq!(id.timestamp_ms(), Souvenir::TIMESTAMP_MAX);
}

/// Code that needs IDs can take any generator, including a borrowed one,
/// a boxed one, a trait object, or one of its own.
#[test]
fn the_generator_trait_abstracts_over_generators() {
    fn two<G: Generator>(generator: &mut G) -> (Id<User>, Id<User>) {
        (generator.generate().unwrap(), generator.generate().unwrap())
    }

    let mut monotonic = MonotonicGenerator::with_sources(random(), || Ok(FIXED_MS));
    let (a, b) = two(&mut monotonic);
    assert!(a < b);
    let (c, d) = two(&mut &mut monotonic);
    assert!(b < c && c < d);

    let mut boxed: Box<dyn Generator> = Box::new(RandomGenerator::with_sources(random(), || Ok(7)));
    assert_eq!(
        boxed
            .generate_untyped(prefix!("user"))
            .unwrap()
            .timestamp_ms(),
        7
    );
    let (e, _) = two(&mut boxed);
    assert_eq!(e.timestamp_ms(), 7);

    /// Scripted IDs, as a test might use.
    struct Scripted(Vec<Souvenir>);
    impl Generator for Scripted {
        fn generate_untyped(&mut self, prefix: souvenir::Prefix) -> souvenir::Result<Souvenir> {
            let next = self.0.remove(0);
            Souvenir::from_parts(prefix, next.timestamp_ms(), next.random())
        }
    }
    let expected = Souvenir::from_parts(prefix!("user"), 5, 6).unwrap();
    let (f, _) = two(&mut Scripted(vec![expected, expected]));
    assert_eq!(f, expected);
}

#[test]
#[cfg(feature = "getrandom")]
fn system_entry_points_smoke() {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = || {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    };
    let before = now();
    let ids = [
        Souvenir::new(prefix!("user")).unwrap(),
        Id::<User>::new().unwrap().to_untyped(),
        RandomGenerator::new()
            .generate_untyped(prefix!("user"))
            .unwrap(),
        MonotonicGenerator::new()
            .generate_untyped(prefix!("user"))
            .unwrap(),
    ];
    let after = now();
    for id in ids {
        assert_eq!(id.prefix(), "user");
        assert!((before..=after).contains(&id.timestamp_ms()));
    }
}
