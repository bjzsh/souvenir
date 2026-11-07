# souvenir

Type-safe, sortable, prefixed identifiers for Rust and JavaScript.

```text
user_00t35pg1sg9ech32tymrkdq6v
```

A Souvenir ID says what it identifies (`user`), when it was created, and
carries 74 random bits. The prefix is part of the binary form too, so an ID
stored as bytes never loses its type.

**Text order equals binary order: IDs sort by type, then by creation time.**

## Rust

```toml
[dependencies]
souvenir = "0.7"
```

### Typed IDs

Most code should use `Id<T>`, where `T` is the record the ID identifies.
Deriving `Tagged` on the record gives its IDs their prefix:

```rust
use souvenir::{Id, Identifiable, Tagged};

#[derive(Tagged, Identifiable)]
#[souvenir(prefix = "user")]
struct User {
    id: Id<User>,
    name: String,
}

#[derive(Tagged)]
#[souvenir(prefix = "org")]
struct Org {
    id: Id<Org>,
}

fn main() -> Result<(), souvenir::Error> {
    let user = User { id: Id::new()?, name: "Ada".into() };
    println!("{}", user.id()); // user_00t35pg1sg...

    let parsed: Id<User> = user.id.to_string().parse()?;
    assert_eq!(parsed, user.id);

    // An org ID is not a user ID: `let org: Id<Org> = user.id;` doesn't
    // compile, and parsing one as the other fails at run time.
    assert!(Id::<Org>::parse(&user.id.to_string()).is_err());
    Ok(())
}
```

Invalid prefixes are compile errors. For kinds of ID without a Rust type of
their own, such as references to another system's records, declare a marker
type with `tagged!`:

```rust
use souvenir::{Id, tagged};

tagged!(pub Invoice = "invoice");

let invoice: Id<Invoice> = Id::from_parts(1_793_531_580_000, 0).unwrap();
```

`id!` parses a literal at compile time:

```rust
use souvenir::{Id, Souvenir, id, tagged};

tagged!(User = "user");

const ADMIN: Id<User> = id!(User, "user_00t35pg1sg9ech32tymrkdq6v");
const ANY: Souvenir = id!("org_00t35pg1sgajz7jf1pz84mv4e");
```

### Untyped IDs

`Souvenir` holds an ID with any prefix. Use it for storage that mixes kinds
of ID, such as event logs, audit trails or polymorphic references. Its prefix
is a `Prefix`: a small, always-valid value made with `prefix!` (checked at
compile time) or `Prefix::parse` (at run time):

```rust
use souvenir::{Prefix, Souvenir, prefix};

const USER: Prefix = prefix!("user");
const ORG: Prefix = prefix!("org");

fn route(text: &str) -> Result<(), souvenir::Error> {
    let id: Souvenir = text.parse()?;
    match id.prefix() {
        USER => { /* ... */ }
        ORG => { /* ... */ }
        other => println!("unknown kind {other}"),
    }
    Ok(())
}

fn make(kind: &str) -> Result<Souvenir, souvenir::Error> {
    Souvenir::new(Prefix::parse(kind)?)
}
```

### Generators

`Souvenir::new` and `Id::new` are stateless; IDs created in the same
millisecond are in random order. When order matters within one process, use
a `MonotonicGenerator`:

```rust
use souvenir::{Generator, Id, MonotonicGenerator, tagged};

tagged!(Event = "event");

fn main() -> Result<(), souvenir::Error> {
    let mut generator = MonotonicGenerator::new();
    let a: Id<Event> = generator.generate()?;
    let b: Id<Event> = generator.generate()?;
    assert!(a < b);
    Ok(())
}
```

It stays strictly increasing even if the clock stalls or goes backwards, and
it may stamp IDs slightly ahead of the wall clock to do so. To share one
between threads, put it in a `Mutex`.

Code that needs new IDs can accept any `impl Generator`, so its caller
chooses: a `RandomGenerator` (what `Souvenir::new` uses), a
`MonotonicGenerator`, or a generator of its own. Both built-in generators
take any `RandomSource` and `Clock`, which makes tests deterministic:

```rust
use souvenir::{RandomGenerator, Generator, prefix};

let zeros = |buf: &mut [u8]| {
    buf.fill(0);
    Ok(())
};
let mut generator = RandomGenerator::with_sources(zeros, || Ok(1_793_531_580_000));
let id = generator.generate_untyped(prefix!("user")).unwrap();
assert_eq!(id.to_string(), "user_00t35pg1sg000000000000000");
```

### Features

| Feature  | Default | Provides |
|----------|---------|----------|
| `getrandom` | yes  | generating IDs from the OS random source (`Souvenir::new`, `Id::new`, `RandomGenerator::new`, `MonotonicGenerator::new`). Turn it off for parse-only builds, such as WebAssembly without a `getrandom` backend; generators can still take any random source. |
| `macros` | yes     | `#[derive(Tagged)]`, `#[derive(Identifiable)]`, `id!` |
| `serde`  | yes     | text form in human-readable formats, 20 bytes otherwise |
| `sqlx`   |         | with `postgres`, `mysql`, `sqlite`; see [Storage](#storage) |
| `diesel` |         | with `postgres`, `mysql`, `sqlite`; see [Storage](#storage) |
| `uuid`   |         | `to_uuid`/`from_uuid` with `uuid::Uuid` (UUIDv7), and conversions between `Id<T>` and `Uuid`; turned on by `sqlx` and `diesel` |

## JavaScript and TypeScript

`souvenir.js` is the same Rust implementation compiled to WebAssembly. IDs
are plain strings at run time and branded types in TypeScript:

```ts
import { typed, type SouvenirId } from 'souvenir.js';

const UserId = typed('user');
type UserId = SouvenirId<'user'>;

const id: UserId = UserId.new();
UserId.parse(input); // throws SouvenirError with code 'PrefixMismatch' for other prefixes
UserId.is(value);    // type guard
```

See [`souvenir-wasm/README.md`](https://github.com/bjzsh/souvenir/blob/main/souvenir-wasm/README.md).

## Format

160 bits, big-endian, 20 bytes:

```text
 bit 159                                                                   bit 0
 +----------------------+--------------------------+---------------------------+
 | prefix rank (38)     | unix_ts_ms (48)          | random (74)               |
 +----------------------+--------------------------+---------------------------+
```

- **Prefix:** 1-8 characters of `a`-`z`, stored as its rank in trie order
  (so `u < us < user < users < v` in both forms).
- **Timestamp:** Unix milliseconds, valid until the year 10889.
- **Random:** 74 bits from a CSPRNG.
- **Text:** `prefix` + `_` + 25 characters of lowercase Crockford Base32
  (`0123456789abcdefghjkmnpqrstvwxyz`) encoding the timestamp and random bits.
  Parsing is strict: one ID has exactly one text form.
- **UUIDv7:** the timestamp and random bits map bit-for-bit onto a UUIDv7.

The full specification is in [SPEC.md](https://github.com/bjzsh/souvenir/blob/main/SPEC.md).

## Storage

IDs are stored as **text by default**: the same string you see in JSON,
URLs and logs. Every table holds the same form, so any two ID columns join
with `=`, and the database stays readable without this library. The prefix
is part of the value, so reading an ID through the wrong `Id<T>` fails with
`PrefixMismatch`.

Declare text columns with a byte-order collation, so the database sorts
IDs the same way Rust does, and a check that rejects anything that isn't an
ID:

```sql
-- Postgres: create once, then use `souvenir` as the column type
CREATE DOMAIN souvenir AS text COLLATE "C"
  CHECK (VALUE ~ '^[a-z]{1,8}_[0-3][0-9a-hjkmnp-tv-z]{24}$');

CREATE TABLE users (id souvenir PRIMARY KEY, ...);
```

```sql
-- MySQL
CREATE TABLE users (
  id VARCHAR(34) CHARACTER SET ascii COLLATE ascii_bin PRIMARY KEY
     CHECK (REGEXP_LIKE(id, '^[a-z]{1,8}_[0-3][0-9a-hjkmnp-tv-z]{24}$', 'c')),
  ...
);
```

SQLite's default `TEXT` collation already sorts by bytes. Never use a
case-insensitive or locale-aware collation: some locales sort IDs out of
creation order.

Each `Tagged` type can declare a different form. Every column that holds that
kind of ID (its table's key and every foreign key to it) then uses the same
form, and code just uses `Id<T>`:

```rust
use souvenir::{Id, Tagged, tagged};

#[derive(Tagged)]
#[souvenir(prefix = "user")] // text, the default
struct User {
    id: Id<User>,
}

#[derive(Tagged)]
#[souvenir(prefix = "acct", storage = Uuid)] // native uuid
struct Account {
    id: Id<Account>,
}

tagged!(pub Event = "event", storage = Bytes); // 20 bytes
```

| Storage | Postgres | MySQL | SQLite | Use for |
|---|---|---|---|---|
| `Text` (default) | `text` / `souvenir` | `VARCHAR(34)` | `TEXT` | almost everything |
| `Bytes` | `bytea` | `BINARY(20)` | `BLOB` (20 B) | very large tables, where the smaller index matters. Still self-describing. |
| `Uuid` | `uuid` | `BINARY(16)` | `BLOB` (16 B) | columns that must be native UUIDs. The prefix is not stored: only the Rust type supplies it, and nothing detects a column read through the wrong `Id<T>`. |

- **sqlx:** the declared form is used for both writing and reading; nothing
  is guessed from a value.
- **diesel:** declare the column's SQL type to match (`Text`, `Binary`, or
  `Uuid` on Postgres). A mismatch is a compile error when the code is built.
- **Untyped `Souvenir`** is stored as text. For bytes, bind
  `&id.as_bytes()[..]` and read with `Souvenir::from_slice` (diesel also maps
  `Souvenir` to `Binary` columns directly).

In JSON and HTTP, use the string; "no ID" is `null` or an absent field. In
URLs, use the string as-is: every character is URL-safe.

## Things to know

- **Prefixes are part of each ID's identity.** Renaming a prefix changes every
  ID that uses it. Choose prefixes like public API names.
- **Prefixes are type tags, not namespaces.** Two applications can both use
  `user`; uniqueness comes from the timestamp and random bits.
- **Ordering is per prefix,** not global across kinds. Use `timestamp_ms()` to
  order across kinds.
- **IDs reveal their creation time,** like UUIDv7 and ULID.
- **Uniqueness is probabilistic.** Two IDs can only collide if they have the
  same prefix and millisecond. Generating 1 million IDs per second with one
  prefix, expect about 8 * 10^-7 collisions per year; at 100 million per second,
  about 8 * 10^-3. These are the same odds as UUIDv7.
- **There is no nil ID.** Represent "no ID" with `Option` or `null`. Zeroed
  bytes never decode as an ID.

## License

MIT
