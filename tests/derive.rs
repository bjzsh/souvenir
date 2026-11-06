//! `#[derive(Tagged)]` on record types, enums and generic types.

use souvenir::{Id, Identifiable, Storage, Tagged};

#[derive(Tagged, Identifiable)]
#[souvenir(prefix = "user", storage = Bytes)]
struct User {
    id: Id<User>,
    #[allow(dead_code)]
    name: String,
}

#[derive(Tagged)]
#[souvenir(prefix = "page")]
struct Page<'a, T: Clone> {
    #[allow(dead_code)]
    items: &'a [T],
}

#[test]
fn record_type_is_its_own_tag() {
    let user = User {
        id: Id::from_parts(1_793_531_580_000, 0).unwrap(),
        name: "Ada".into(),
    };
    assert_eq!(user.id().to_string(), "user_00t35pg1sg000000000000000");
    assert_eq!(User::STORAGE, Storage::Bytes);
}

#[test]
fn generic_types_with_lifetimes() {
    let id = Id::<Page<'static, u8>>::from_parts(1_793_531_580_000, 0).unwrap();
    assert_eq!(id.prefix(), "page");
}

#[derive(Identifiable)]
struct Membership {
    #[souvenir(id)]
    member: Id<User>,
    // An unmarked `id` field is ignored when another field is marked.
    #[allow(dead_code)]
    id: u64,
}

#[test]
fn marked_field_wins_over_id() {
    let member = Id::from_parts(1_793_531_580_000, 1).unwrap();
    assert_eq!(Membership { member, id: 7 }.id(), member);
}

/// The ID field can be any `Identifiable` value, such as the record it
/// belongs to.
#[derive(Identifiable)]
struct Profile {
    #[souvenir(id)]
    user: User,
    #[allow(dead_code)]
    bio: String,
}

#[test]
fn id_field_can_be_another_identifiable() {
    let user = User {
        id: Id::from_parts(1_793_531_580_000, 9).unwrap(),
        name: "Ada".into(),
    };
    let expected = user.id;
    let profile = Profile {
        user,
        bio: String::new(),
    };
    assert_eq!(profile.id(), expected);
}
