//! Property tests, 10 000 cases each.

use proptest::prelude::*;
use souvenir::{Error, Prefix, Souvenir};

const CASES: u32 = 10_000;

/// One past the rank of `"zzzzzzzz"`; ranks from here on are reserved.
const PREFIX_RANK_LIMIT: u64 = 217_180_147_159;

fn rank(prefix: &str) -> Result<u64, Error> {
    Prefix::parse(prefix).map(Prefix::rank)
}

fn unrank(rank: u64) -> Result<Prefix, Error> {
    Prefix::from_rank(rank)
}

/// Any valid prefix, biased toward chains like `u/us/use/user/users` so that
/// prefixes of one another are compared often.
fn prefix() -> impl Strategy<Value = String> {
    prop_oneof![
        2 => "[a-z]{1,8}",
        2 => prop::sample::select(vec!["u", "us", "use", "user", "users", "usera", "uz", "v", "a", "zzzzzzzz"])
            .prop_map(str::to_owned),
        1 => ("[a-z]{1,4}", 0usize..=4).prop_map(|(base, extra)| {
            let mut s = base.clone();
            s.extend(std::iter::repeat_n('a', extra));
            s
        }),
    ]
}

fn souvenir() -> impl Strategy<Value = Souvenir> {
    (
        prefix(),
        0..=Souvenir::TIMESTAMP_MAX,
        0..=Souvenir::RANDOM_MAX,
    )
        .prop_map(|(p, t, r)| Souvenir::from_parts(Prefix::parse(&p).unwrap(), t, r).unwrap())
}

const CODES: &[Error] = &[
    Error::InvalidPrefix,
    Error::MissingSeparator,
    Error::InvalidLength,
    Error::InvalidCharacter,
    Error::NonCanonical,
    Error::ZeroPrefix,
    Error::ReservedPrefix,
];

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    // 1. rank/unrank bijection
    #[test]
    fn unrank_rank(p in prefix()) {
        prop_assert_eq!(unrank(rank(&p).unwrap()).unwrap(), p.as_str());
    }

    #[test]
    fn rank_unrank(r in 1..PREFIX_RANK_LIMIT) {
        prop_assert_eq!(rank(unrank(r).unwrap().as_str()), Ok(r));
    }

    // 2. rank preserves string order
    #[test]
    fn rank_order(a in prefix(), b in prefix()) {
        prop_assert_eq!(a.cmp(&b), rank(&a).unwrap().cmp(&rank(&b).unwrap()));
    }

    // 3. round trips
    #[test]
    fn round_trips(x in souvenir()) {
        prop_assert_eq!(Souvenir::parse(&x.to_string()), Ok(x));
        prop_assert_eq!(Souvenir::from_bytes(x.to_bytes()), Ok(x));
        #[cfg(feature = "uuid")]
        prop_assert_eq!(Souvenir::from_uuid(x.prefix(), x.to_uuid()), Ok(x));
    }

    // 4. core invariant: byte order == text order, across mixed prefixes
    #[test]
    fn byte_order_is_text_order(x in souvenir(), y in souvenir()) {
        prop_assert_eq!(x.to_bytes().cmp(&y.to_bytes()), x.to_string().cmp(&y.to_string()));
        prop_assert_eq!(x.cmp(&y), x.to_string().cmp(&y.to_string()));
    }

    // 5. within one prefix, UUID order == ID order
    #[test]
    #[cfg(feature = "uuid")]
    fn uuid_order_within_prefix(p in prefix(), t1 in 0..=Souvenir::TIMESTAMP_MAX, r1 in 0..=Souvenir::RANDOM_MAX,
                                t2 in 0..=Souvenir::TIMESTAMP_MAX, r2 in 0..=Souvenir::RANDOM_MAX) {
        let p = Prefix::parse(&p).unwrap();
        let x = Souvenir::from_parts(p, t1, r1).unwrap();
        let y = Souvenir::from_parts(p, t2, r2).unwrap();
        prop_assert_eq!(x.to_uuid().cmp(&y.to_uuid()), x.cmp(&y));
    }

    // 6. canonical text: anything accepted formats back identically; no panics
    #[test]
    fn parse_is_canonical(s in "\\PC{0,40}") {
        match Souvenir::parse(&s) {
            Ok(id) => prop_assert_eq!(id.to_string(), s),
            Err(e) => prop_assert!(CODES.contains(&e)),
        }
    }

    #[test]
    fn parse_near_valid(s in "[a-z_]{0,9}_[0-9a-z]{23,27}") {
        match Souvenir::parse(&s) {
            Ok(id) => prop_assert_eq!(id.to_string(), s),
            Err(e) => prop_assert!(CODES.contains(&e)),
        }
    }

    // 7. any 20 bytes either error with a defined code or round-trip exactly
    #[test]
    fn from_bytes_total(bytes in prop::array::uniform20(any::<u8>())) {
        match Souvenir::from_bytes(bytes) {
            Ok(id) => prop_assert_eq!(id.to_bytes(), bytes),
            Err(e) => prop_assert!(matches!(e, Error::ZeroPrefix | Error::ReservedPrefix)),
        }
    }

    #[test]
    #[cfg(feature = "uuid")]
    fn from_uuid_total(bytes in prop::array::uniform16(any::<u8>())) {
        let bytes: [u8; 16] = bytes;
        let uuid = uuid::Uuid::from_bytes(bytes);
        match Souvenir::from_uuid(souvenir::prefix!("user"), uuid) {
            Ok(id) => prop_assert_eq!(id.to_uuid(), uuid),
            Err(e) => prop_assert_eq!(e, Error::NotUuidV7),
        }
    }
}
