use crate::prefix::Prefix;
use crate::storage::Storage;

/// A type that fixes an [`Id`](crate::Id)'s prefix at compile time, so an
/// `Id<X>` can never be passed where an `Id<Y>` is expected.
///
/// Usually the record type the ID identifies, with `#[derive(Tagged)]` (with
/// the `macros` feature). For kinds of ID without a Rust type of their own,
/// declare a unit marker type with [`tagged!`](crate::tagged!):
///
/// ```
/// use souvenir::{Id, tagged};
///
/// tagged!(pub User = "user");
///
/// let id: Id<User> = "user_00t35pg1sg9ech32tymrkdq6v".parse().unwrap();
/// assert_eq!(id.prefix(), "user");
/// ```
///
/// By hand, build the prefix with [`prefix!`](crate::prefix!), which checks
/// it at compile time:
///
/// ```
/// use souvenir::{Prefix, Tagged, prefix};
///
/// struct User;
/// impl Tagged for User {
///     const PREFIX: Prefix = prefix!("user");
/// }
/// ```
pub trait Tagged {
    /// The prefix of IDs of this kind.
    const PREFIX: Prefix;

    /// How IDs of this kind are stored in a database. Defaults to
    /// [`Storage::Text`].
    const STORAGE: Storage = Storage::Text;
}

/// Declare one or more [`Tagged`] marker types.
///
/// Each is a unit struct. An invalid prefix fails to build. An optional
/// `storage = Text | Bytes | Uuid` sets [`Tagged::STORAGE`] (default
/// `Text`).
///
/// ```
/// use souvenir::{Id, Storage, Tagged, tagged};
///
/// tagged!(pub User = "user");
/// tagged! {
///     /// Doc comments and other attributes carry over to the type.
///     pub Org = "org";
///     Event = "event", storage = Bytes;
/// }
///
/// let id = Id::<Org>::from_parts(1_793_531_580_000, 0).unwrap();
/// assert_eq!(id.to_string(), "org_00t35pg1sg000000000000000");
/// assert_eq!(Event::STORAGE, Storage::Bytes);
/// ```
///
/// ```compile_fail
/// souvenir::tagged!(Bad = "Bad");
/// ```
#[macro_export]
macro_rules! tagged {
    ($(
        $(#[$meta:meta])* $vis:vis $name:ident = $prefix:literal $(, storage = $storage:ident)?
    );+ $(;)?) => {
        $(
            $(#[$meta])*
            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
            $vis struct $name;

            impl $crate::Tagged for $name {
                const PREFIX: $crate::Prefix = $crate::prefix!($prefix);
                $(const STORAGE: $crate::Storage = $crate::Storage::$storage;)?
            }
        )+
    };
}
