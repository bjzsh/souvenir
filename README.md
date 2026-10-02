# souvenir

A prefixed identifier library for Rust.

An `Id` stores a 128-bit value with a 1–4 letter lowercase ASCII prefix and a
numeric suffix. Its canonical text is the prefix, an underscore, and 22 digits
from a lowercase variant of [Crockford Base 32](https://www.crockford.com/base32.html).

```rs
use souvenir::{Id, prefix};

let user = Id::random(prefix!("user"));
println!("{user}");

let id = Id::parse("user_02v58c5a3fy30k560qrtg4").unwrap();
let team = id.with_prefix(prefix!("team")).unwrap();
assert_eq!(id.suffix(), team.suffix());
assert_eq!(team.prefix(), prefix!("team"));
```

## Representation

Every 16-byte value represents a valid identifier. Bytes use big-endian order,
and IDs sort by prefix, then numeric suffix. Unsigned integers, byte arrays, and
canonical text under bytewise comparison have the same ordering.

`Id::new(bytes)` and `From` conversions from `[u8; 16]`, `u128`, and `i128` are
infallible. `Id::from_parts(prefix, suffix)` and `with_prefix(prefix)` validate
the prefix's suffix bound. Suffixes are `u128` values with approximately 109.142
bits of entropy when sampled uniformly for a fixed prefix.

```rs
use souvenir::{Id, prefix};

let id = Id::from_parts(prefix!("user"), 42).unwrap();
assert_eq!(id.suffix(), 42);
assert_eq!(Id::new(id.to_bytes()), id);
assert_eq!(Id::from(id.to_u128()), id);
```

Signed integer conversions preserve the underlying bits; signed integer ordering
does not match ID ordering across the sign boundary.

See the [format specification](docs/format.md) for prefix ranks, suffix bounds,
binary encoding, and fixed test vectors.

## Features and integrations

The default features are `macros`, `rand`, and `serde`.

- `macros`: `id!`, `prefix!`, and the `Tagged` and `Identifiable` derives.
- `rand`: random generation, including `Id::random_with` for a supplied RNG.
- `serde`: canonical strings for human-readable serializers and 16-byte arrays
  for binary serializers. A `Prefix` serializes as text or its numeric rank.
- `sqlx` or `diesel`, together with `postgres`, `mysql`, or `sqlite`: database
  support. PostgreSQL stores UUID bytes, MySQL stores binary bytes, and SQLite
  stores canonical text.

Use `default-features = false` for only the core identifier API, or select the
features needed by your application.
