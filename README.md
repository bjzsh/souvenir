# souvenir

A type-safe, tagged identifier library.

The crate primarily provides the `Id` struct, which stores a 128-bit
identifier with its corresponding type (tag).
The String representation of an `Id` is the type's tag and the
128-bit value encoded into a variant of
[Crockford Base 32](https://www.crockford.com/base32.html).

Here is a simple example of how this crate can be used.

```rs
use souvenir::{Type, Id};

struct User;

impl Type for User {
// Specify a prefix for all `Id<User>`
const PREFIX: &'static str = "user";
}

let id: Id<User> = Id::random();
println!("{}", id);

let id2: Id<User> = Id::parse("user_02v58c5a3fy30k560qrtg4rb2k").unwrap();
assert_eq!(id2.to_string(), "user_02v58c5a3fy30k560qrtg4rb2k");
```

Integrations for various libraries and databases are also (optionally)
available:

- (De)serialization with [`serde`](https://docs.rs/serde/latest/serde/)
- Random ID generation with [`rand`](https://docs.rs/rand/latest/rand/)
- Postgres, MySQL, and Sqlite support with
  [`sqlx`](https://docs.rs/sqlx/latest/sqlx/) and
  [`diesel`](https://docs.rs/diesel/latest/diesel/)

