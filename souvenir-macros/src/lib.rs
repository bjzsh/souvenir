#![forbid(unsafe_code)]
// Library code never panics at run time: no explicit panics, and no
// `unwrap`/`expect`. (Panics in `const` evaluation, such as an invalid
// `prefix!` literal, are compile errors and are fine.)
#![cfg_attr(
    not(test),
    deny(
        clippy::panic,
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::unreachable,
        clippy::todo,
        clippy::unimplemented
    )
)]
#![warn(missing_docs)]

//! # souvenir_macros
//!
//! Procedural macros for [`souvenir`](https://docs.rs/souvenir/latest/souvenir/).
//! Use them through that crate rather than depending on this one directly.

extern crate proc_macro;

mod attr;
mod id;
mod identifiable;
mod tagged;

use proc_macro::TokenStream;

/// Parse an ID literal at compile time.
///
/// - `id!("user_...")` produces a `Souvenir`.
/// - `id!(User, "user_...")` produces an `Id<User>`. A literal with another
///   prefix is a compile error.
///
/// Invalid literals are compile errors, and the value is built at compile
/// time.
///
/// ```
/// use souvenir::{Id, Souvenir, id, tagged};
///
/// tagged!(User = "user");
///
/// let any: Souvenir = id!("user_00t35pg1sg9ech32tymrkdq6v");
/// let user: Id<User> = id!(User, "user_00t35pg1sg9ech32tymrkdq6v");
/// assert_eq!(any, user);
/// ```
///
/// ```compile_fail
/// let bad = souvenir::id!("user_00t35pg1sg9ech32tymrkdq6");
/// ```
///
/// A literal with the wrong prefix for the type is rejected when the code is
/// built. (`cargo check` does not report it, because the check runs during
/// const evaluation.)
///
/// ```compile_fail
/// use souvenir::{id, tagged};
///
/// tagged!(Org = "org");
/// let org = id!(Org, "user_00t35pg1sg9ech32tymrkdq6v");
/// ```
#[proc_macro]
pub fn id(input: TokenStream) -> TokenStream {
    id::id(input.into()).into()
}

/// Implement `Tagged`, so `Id<Self>` is the ID type of this kind of record.
///
/// Usually derived on the record type itself. `storage = Text | Bytes |
/// Uuid` sets how IDs of this kind are stored in a database (default
/// `Text`).
///
/// ```
/// use souvenir::{Id, Identifiable, Storage, Tagged};
///
/// #[derive(Tagged, Identifiable)]
/// #[souvenir(prefix = "user")]
/// struct User {
///     id: Id<User>,
///     name: String,
/// }
///
/// #[derive(Tagged)]
/// #[souvenir(prefix = "acct", storage = Uuid)]
/// enum Account {
///     Personal,
///     Business,
/// }
///
/// let user = User { id: Id::from_parts(1_793_531_580_000, 0).unwrap(), name: "Ada".into() };
/// assert_eq!(user.id().to_string(), "user_00t35pg1sg000000000000000");
/// assert_eq!(User::STORAGE, Storage::Text);
/// assert_eq!(Account::STORAGE, Storage::Uuid);
/// ```
///
/// Invalid prefixes, a missing `prefix` and an unknown `storage` are compile
/// errors:
///
/// ```compile_fail
/// #[derive(souvenir::Tagged)]
/// #[souvenir(prefix = "User")]
/// struct User;
/// ```
///
/// ```compile_fail
/// #[derive(souvenir::Tagged)]
/// struct User;
/// ```
///
/// ```compile_fail
/// #[derive(souvenir::Tagged)]
/// #[souvenir(prefix = "user", storage = Binary)]
/// struct User;
/// ```
#[proc_macro_derive(Tagged, attributes(souvenir))]
pub fn tagged(input: TokenStream) -> TokenStream {
    tagged::derive(input.into()).into()
}

/// Implement `Identifiable` using the field named `id`, or the field marked
/// `#[souvenir(id)]` if another field holds the ID. The field must be
/// `Identifiable`: an `Id<T>`, or a value that has one.
///
/// ```
/// use souvenir::{Id, Identifiable, tagged};
///
/// tagged!(User = "user");
///
/// #[derive(Identifiable)]
/// struct Account {
///     id: Id<User>,
///     name: String,
/// }
///
/// #[derive(Identifiable)]
/// struct Membership {
///     #[souvenir(id)]
///     member: Id<User>,
///     role: String,
/// }
///
/// let account = Account { id: Id::new().unwrap(), name: "Ada".into() };
/// assert_eq!(account.id(), account.id);
/// let membership = Membership { member: account.id, role: "admin".into() };
/// assert_eq!(membership.id(), account.id);
/// ```
///
/// A struct with neither is a compile error:
///
/// ```compile_fail
/// use souvenir::{Id, Identifiable, tagged};
///
/// tagged!(User = "user");
///
/// #[derive(Identifiable)]
/// struct Account {
///     key: Id<User>,
/// }
/// ```
#[proc_macro_derive(Identifiable, attributes(souvenir))]
pub fn identifiable(input: TokenStream) -> TokenStream {
    identifiable::derive(input.into()).into()
}
