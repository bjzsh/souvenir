//! Benchmarks. Targets: parse and format under 100 ns,
//! `new` under 200 ns excluding the cost of the CSPRNG.

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use souvenir::{Generator, MonotonicGenerator, RandomGenerator, Souvenir, prefix};
use std::hint::black_box;

const TEXT: &str = "invoice_00t35pg1sgananananananana";

fn codec(c: &mut Criterion) {
    let id = Souvenir::parse(TEXT).unwrap();

    c.bench_function("parse", |b| b.iter(|| Souvenir::parse(black_box(TEXT))));
    c.bench_function("format (encode into buffer)", |b| {
        let mut buf = [0u8; Souvenir::MAX_TEXT_LEN];
        b.iter(|| black_box(black_box(id).encode(&mut buf)).len())
    });
    c.bench_function("to_string", |b| b.iter(|| black_box(id).to_string()));
    #[cfg(feature = "uuid")]
    c.bench_function("to_uuid", |b| b.iter(|| black_box(id).to_uuid()));
}

/// Field accessors and byte conversions, over IDs already in memory. Timing
/// a single `black_box`ed ID would mostly measure the CPU stalling while it
/// reads back the 20 bytes it has just written.
fn accessors(c: &mut Criterion) {
    let p = prefix!("invoice");
    let ids: Vec<Souvenir> = (0..1024u64)
        .map(|i| Souvenir::from_parts(p, 1_793_531_580_000 + i, u128::from(i) * 7919).unwrap())
        .collect();
    let bytes: Vec<[u8; Souvenir::BYTE_LEN]> = ids.iter().map(|id| id.to_bytes()).collect();
    let mut group = c.benchmark_group("per ID");
    group.throughput(Throughput::Elements(ids.len() as u64));
    group.bench_function("from_bytes", |b| {
        b.iter(|| {
            black_box(&bytes)
                .iter()
                .filter(|b| Souvenir::from_bytes(**b).is_ok())
                .count()
        })
    });
    group.bench_function("prefix", |b| {
        b.iter(|| {
            black_box(&ids)
                .iter()
                .map(|id| id.prefix().len())
                .sum::<usize>()
        })
    });
    group.bench_function("timestamp_ms", |b| {
        b.iter(|| {
            black_box(&ids)
                .iter()
                .map(|id| id.timestamp_ms())
                .fold(0, u64::wrapping_add)
        })
    });
    group.bench_function("random", |b| {
        b.iter(|| {
            black_box(&ids)
                .iter()
                .map(|id| id.random())
                .fold(0, u128::wrapping_add)
        })
    });
    group.finish();
}

fn generation(c: &mut Criterion) {
    let fixed_clock = || Ok(1_793_531_580_000);
    let mut no_rng = RandomGenerator::with_sources(
        |buf: &mut [u8]| {
            buf.fill(0x5A);
            Ok(())
        },
        fixed_clock,
    );
    let mut monotonic_no_rng = MonotonicGenerator::with_sources(
        |buf: &mut [u8]| {
            buf.fill(0);
            Ok(())
        },
        fixed_clock,
    );
    c.bench_function("generate (no CSPRNG, fixed clock)", |b| {
        b.iter(|| no_rng.generate_untyped(black_box(prefix!("invoice"))))
    });
    c.bench_function("generate monotonic (no CSPRNG, fixed clock)", |b| {
        b.iter(|| monotonic_no_rng.generate_untyped(black_box(prefix!("invoice"))))
    });
    #[cfg(feature = "getrandom")]
    c.bench_function("Souvenir::new (system clock + getrandom)", |b| {
        b.iter(|| Souvenir::new(black_box(prefix!("invoice"))))
    });
}

criterion_group!(benches, codec, accessors, generation);
criterion_main!(benches);
