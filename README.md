# souvenir

A type-safe, tagged identifier library.

The crate primarily provides the `Id` struct, which stores a 128-bit
identifier with its corresponding type (tag).
The String representation of an `Id` is the type's tag and the
numeric suffix encoded into a variant of
[Crockford Base 32](https://www.crockford.com/base32.html).

Here is a simple example of how this crate can be used.

```rs
use souvenir::Id;

let id: Id = Id::random("user".parse().unwrap());
println!("{}", id);

let id2: Id = Id::parse("user_02v58c5a3fy30k560qrtg4").unwrap();
assert_eq!(id2.to_string(), "user_02v58c5a3fy30k560qrtg4");
```

Integrations for various libraries and databases are also (optionally)
available:

- (De)serialization with [`serde`](https://docs.rs/serde/latest/serde/)
- Random ID generation with [`rand`](https://docs.rs/rand/latest/rand/)
- Postgres, MySQL, and Sqlite support with
  [`sqlx`](https://docs.rs/sqlx/latest/sqlx/) and
  [`diesel`](https://docs.rs/diesel/latest/diesel/)

## Dense encoding migration (in development)

The dense format orders IDs by prefix, then numeric suffix, and accepts every
16-byte value. `Id::new(bytes)` and integer `From` conversions are infallible;
`Id::from_parts(prefix, suffix)` and `with_prefix(prefix)` check the prefix's
suffix bound.
Suffixes are plain `u128` values rather than a separate `Suffix` type.

This changes binary meanings. Migrate legacy bytes by decoding them with the old
version, exporting canonical text, and parsing that text with the new version.
Do not reinterpret old bytes directly: they will be accepted but generally mean
different IDs. Existing canonical text remains valid.
