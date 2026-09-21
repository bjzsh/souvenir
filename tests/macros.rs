#![cfg(feature = "macros")]

use souvenir::{Id, Identifiable, Tagged};

#[derive(Identifiable, Tagged)]
#[souvenir(tag = "user")]
struct User<T: Copy = u8>
where
    T: Default,
{
    #[souvenir(id)]
    id: Id,
    data: T,
}

#[derive(Identifiable)]
struct Wrapper(#[souvenir(id)] Id);

#[test]
fn derives_support_named_and_tuple_fields() {
    let id = Id::new([0; 16]);
    let user = User { id, data: 0u8 };
    assert_eq!(user.id(), id);
    assert_eq!(user.data, 0);
    assert_eq!(User::<u8>::PREFIX.to_string(), "user");
    assert_eq!(Wrapper(id).id(), id);
}
