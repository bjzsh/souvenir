use souvenir_core::{
    encoding::{
        PREFIX_COUNT, Q, R, SPLIT, decode_suffix, encode_suffix, id_interval, split_id,
        suffix_interval,
    },
    error::Error,
    id::Id,
    prefix::Prefix,
};

#[test]
fn every_interval_is_adjacent_and_round_trips_at_its_boundaries() {
    let mut next = Some(0);
    for rank in 0..PREFIX_COUNT {
        let prefix = Prefix::new(rank).unwrap();
        let interval = id_interval(prefix);
        let max = *suffix_interval(prefix).end();
        assert_eq!(Some(*interval.start()), next);
        assert_eq!(interval.end() - interval.start(), max);
        for suffix in [0, 1, max / 2, max - 1, max] {
            let id = Id::from_parts(prefix, suffix).unwrap();
            assert_eq!(id.to_u128(), interval.start() + suffix);
            assert_eq!(id.prefix(), prefix);
            assert_eq!(id.suffix(), suffix);
            assert_eq!(Id::parse(&id.to_string()), Ok(id));
        }
        assert!(Id::from_parts(prefix, max + 1).is_err());
        assert!(Id::from_parts(prefix, u128::MAX).is_err());
        if let Some(previous) = interval.start().checked_sub(1) {
            let previous = Id::from(previous);
            let first = Id::from(*interval.start());
            assert!(previous < first);
            assert!(previous.to_bytes() < first.to_bytes());
            assert!(previous.to_string() < first.to_string());
        }
        next = interval.end().checked_add(1);
    }
    assert_eq!(next, None);
}

#[test]
fn fixed_vectors_and_interval_size_transition() {
    assert_eq!(Q, 716_001_058_215_056_503_392_658_678_163_188);
    assert_eq!(R, 461_704);
    assert_eq!(split_id(0), (0, 0));
    assert_eq!(split_id(u128::MAX), (475_253, Q - 1));
    assert_eq!(split_id(SPLIT - 1), (R as u32 - 1, Q));
    assert_eq!(split_id(SPLIT), (R as u32, 0));
    assert_eq!(split_id(SPLIT + 1), (R as u32, 1));
    let vectors = [
        ("a_0000000000000000000000", 0u128),
        (
            "user_02v58c5a3fy30k560qrtg4",
            0xcbcee7b695fe097e9d18ec46bc97fe46,
        ),
        ("zzzz_hmtd1hxh33gedanpsmt9qk", u128::MAX),
    ];
    for (text, value) in vectors {
        assert_eq!(Id::parse(text).unwrap().to_u128(), value);
        assert_eq!(Id::from(value).to_string(), text);
        assert_eq!(Id::new(value.to_be_bytes()), Id::from(value));
    }
}

#[test]
fn checked_with_prefix_and_text_bounds() {
    let large = Prefix::new(R as u32 - 1).unwrap();
    let small = Prefix::new(R as u32).unwrap();
    let largest = Id::from_parts(large, Q).unwrap();
    assert_eq!(largest.with_prefix(small), Err(Error::InvalidData));
    let shared = Id::from_parts(large, Q - 1).unwrap();
    assert_eq!(shared.with_prefix(small).unwrap().suffix(), Q - 1);
    let text = format!("{small}_{}", encode_suffix(Q).unwrap());
    assert_eq!(Id::parse(&text), Err(Error::InvalidData));
    assert_eq!(
        Id::parse("a_zzzzzzzzzzzzzzzzzzzzzz"),
        Err(Error::InvalidData)
    );
    for value in [1 << 108, 1 << 109, Q, (1 << 110) - 1] {
        assert_eq!(decode_suffix(&encode_suffix(value).unwrap()), Ok(value));
    }
    for value in [1 << 110, u128::MAX] {
        assert_eq!(encode_suffix(value), Err(Error::InvalidData));
    }
}

#[test]
fn arbitrary_bits_and_slice_lengths() {
    let mut value = 1u128;
    for _ in 0..10_000 {
        value ^= value << 23;
        value ^= value >> 17;
        value ^= value << 26;
        let id = Id::from(value);
        assert_eq!(Id::from_parts(id.prefix(), id.suffix()), Ok(id));
        assert_eq!(Id::parse(&id.to_string()), Ok(id));
        assert_eq!(Id::from(id.to_i128()), id);
        assert_eq!(Id::try_from(id.as_bytes().as_slice()), Ok(id));
    }
    for len in [0, 15, 17] {
        assert_eq!(
            Id::try_from(vec![0; len].as_slice()),
            Err(Error::InvalidLength {
                expected: 16,
                found: len
            })
        );
    }
}

#[cfg(feature = "rand")]
#[test]
fn seeded_random_generation_uses_the_expanded_range() {
    use rand::{SeedableRng, rngs::StdRng};
    let mut rng = StdRng::seed_from_u64(7);
    for rank in [0, R as u32 - 1, R as u32, PREFIX_COUNT - 1] {
        let prefix = Prefix::new(rank).unwrap();
        let mut expanded = false;
        for _ in 0..1000 {
            let id = Id::random_with(prefix, &mut rng);
            assert_eq!(id.prefix(), prefix);
            assert!(suffix_interval(prefix).contains(&id.suffix()));
            expanded |= id.suffix() >= 1 << 109;
        }
        assert!(expanded);
    }
}

#[cfg(feature = "serde")]
#[test]
fn serde_preserves_text_and_raw_binary_values() {
    use serde_test::{Configure, Token, assert_tokens};
    for (value, byte, text) in [
        (0, 0, "a_0000000000000000000000"),
        (u128::MAX, 255, "zzzz_hmtd1hxh33gedanpsmt9qk"),
    ] {
        let id = Id::from(value);
        assert_tokens(&id.readable(), &[Token::Str(text)]);
        let mut tokens = vec![Token::Tuple { len: 16 }];
        tokens.extend([Token::U8(byte); 16]);
        tokens.push(Token::TupleEnd);
        assert_tokens(&id.compact(), &tokens);
    }
}
