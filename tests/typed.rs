//! Typed IDs, `tagged!` and `Prefix` values.

mod support;

use souvenir::{
    Error, Generator, Id, MonotonicGenerator, Prefix, Souvenir, Tagged, prefix, tagged,
};
use support::random;

tagged! {
    /// Users.
    pub User = "user";
    Org = "org";
    Users = "users";
}

const TEXT: &str = "user_00t35pg1sg9ech32tymrkdq6v";

#[test]
fn parse_and_format() {
    let id: Id<User> = TEXT.parse().unwrap();
    assert_eq!(id.to_string(), TEXT);
    assert_eq!(format!("{id:?}"), TEXT);
    assert_eq!(format!("{:?}", id.to_untyped()), TEXT);
    assert_eq!(id.prefix(), "user");
    assert_eq!(id.timestamp_ms(), 1_793_531_580_000);
    assert_eq!(Id::<User>::try_from(TEXT), Ok(id));
    assert!(Id::<User>::is_valid(TEXT));
    assert!(!Id::<Org>::is_valid(TEXT));
    assert!(!Id::<User>::is_valid("user_"));
}

#[test]
fn wrong_prefix_is_rejected() {
    assert_eq!(Id::<Org>::parse(TEXT), Err(Error::PrefixMismatch));
    assert_eq!(
        Id::<Users>::parse(TEXT),
        Err(Error::PrefixMismatch),
        "users is not user"
    );
    let untyped = Souvenir::parse(TEXT).unwrap();
    assert_eq!(Id::<Org>::try_from(untyped), Err(Error::PrefixMismatch));
    assert_eq!(
        Id::<Org>::from_bytes(untyped.to_bytes()),
        Err(Error::PrefixMismatch)
    );
    // Parse errors still come first.
    assert_eq!(Id::<Org>::parse("user"), Err(Error::MissingSeparator));
}

#[test]
fn conversions() {
    let id = Id::<User>::from_parts(1_793_531_580_000, 42).unwrap();
    let untyped: Souvenir = id.into();
    assert_eq!(untyped, id);
    assert_eq!(id, untyped);
    assert_eq!(Id::<User>::try_from(untyped), Ok(id));
    assert_eq!(Id::<User>::from_bytes(id.to_bytes()), Ok(id));
    assert_eq!(Id::<User>::from_slice(id.as_bytes()), Ok(id));
    #[cfg(feature = "uuid")]
    {
        assert_eq!(Id::<User>::from_uuid(id.to_uuid()), Ok(id));
        assert_eq!(untyped.to_uuid(), id.to_uuid());
    }
    assert_eq!(id.to_untyped().prefix(), id.prefix());
    assert_eq!(
        Id::<User>::from_parts(1 << 48, 0),
        Err(Error::TimestampOutOfRange)
    );
}

#[test]
fn generation() {
    #[cfg(feature = "getrandom")]
    assert_eq!(Id::<User>::new().unwrap().prefix(), "user");
    let mut generator = MonotonicGenerator::with_sources(random(), || Ok(5));
    let b: Id<Org> = generator.generate().unwrap();
    let c: Id<Org> = generator.generate().unwrap();
    assert!(b < c);
    assert_eq!(b.timestamp_ms(), 5);
}

#[test]
fn ordering_and_hashing_match_untyped() {
    use std::collections::HashSet;
    let mut ids: Vec<Id<User>> = (0..100)
        .map(|i| Id::from_parts(1000 - i, i as u128).unwrap())
        .collect();
    ids.sort();
    let untyped: Vec<Souvenir> = ids.iter().map(|&id| id.into()).collect();
    assert!(untyped.windows(2).all(|w| w[0] < w[1]));
    let set: HashSet<Id<User>> = ids.iter().copied().collect();
    assert_eq!(set.len(), 100);
}

#[test]
fn const_construction() {
    const ID: Id<User> = match Id::<User>::parse(TEXT) {
        Ok(id) => id,
        Err(_) => panic!(),
    };
    assert_eq!(ID.to_string(), TEXT);
}

#[test]
fn id_is_send_sync_for_any_marker() {
    // A marker type that is neither Send nor Sync.
    struct Local(#[allow(dead_code)] std::rc::Rc<()>);
    impl Tagged for Local {
        const PREFIX: Prefix = prefix!("local");
    }
    fn assert_send_sync<T: Send + Sync + Copy>() {}
    assert_send_sync::<Id<Local>>();
    assert_eq!(size_of::<Id<Local>>(), 20);
}

#[test]
fn prefix_values() {
    const USER: Prefix = prefix!("user");
    let id: Id<User> = TEXT.parse().unwrap();
    assert_eq!(id.prefix(), USER);
    assert_eq!(id.to_untyped().prefix(), USER);
    assert_eq!(User::PREFIX, USER);
    // Prefix constants work as match patterns.
    match id.to_untyped().prefix() {
        USER => {}
        other => panic!("unexpected prefix {other}"),
    }
    let untyped = Souvenir::from_parts(USER, 1_793_531_580_000, 0).unwrap();
    assert_eq!(
        Souvenir::parse_expecting(USER, &untyped.to_string()),
        Ok(untyped)
    );
    assert_eq!(
        Souvenir::parse_expecting(prefix!("org"), &untyped.to_string()),
        Err(Error::PrefixMismatch)
    );
}

#[test]
fn timestamp_as_system_time() {
    use std::time::{Duration, UNIX_EPOCH};
    let id: Id<User> = TEXT.parse().unwrap();
    let expected = UNIX_EPOCH + Duration::from_millis(1_793_531_580_000);
    assert_eq!(id.timestamp(), expected);
    assert_eq!(id.to_untyped().timestamp(), expected);
}

#[test]
#[cfg(feature = "uuid")]
fn uuid_crate_conversions() {
    let id: Id<User> = TEXT.parse().unwrap();
    let uuid = uuid::Uuid::from(id);
    assert_eq!(uuid.get_version_num(), 7);
    assert_eq!(uuid, id.to_uuid());
    assert_eq!(Id::<User>::try_from(uuid), Ok(id));
    assert_eq!(
        Id::<User>::try_from(uuid::Uuid::nil()),
        Err(Error::NotUuidV7)
    );
}

#[test]
fn typed_maps_accept_untyped_lookups() {
    use std::collections::{BTreeMap, HashMap};
    let id: Id<User> = TEXT.parse().unwrap();
    let hashed = HashMap::from([(id, 1)]);
    let ordered = BTreeMap::from([(id, 2)]);
    let untyped = id.to_untyped();
    assert_eq!(hashed.get(&untyped), Some(&1));
    assert_eq!(ordered.get(&untyped), Some(&2));
    let other = Souvenir::from_parts(prefix!("org"), id.timestamp_ms(), id.random()).unwrap();
    assert_eq!(hashed.get(&other), None);
}
