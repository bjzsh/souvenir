# souvenir-postgres

A native Postgres type for [Souvenir IDs](../README.md), built with
[pgrx](https://github.com/pgcentralfoundation/pgrx). Postgres 14-18.

```sql
CREATE EXTENSION souvenir;

CREATE TABLE users (
  id souvenir PRIMARY KEY DEFAULT gen_souvenir('user'),
  name text
);

INSERT INTO users (name) VALUES ('Ada') RETURNING id;
--               id
-- --------------------------------
--  user_00t3scv2dpmramfjfjqpsaykm
```

## Why

| | `text` domain | `bytea` | `souvenir` |
|---|---|---|---|
| Size on disk and in indexes | ~31 B | 21 B | 20 B |
| Readable in `psql`, logs, dumps | yes | no | yes |
| Sorts in ID order | with `COLLATE "C"` | yes | yes |
| Rejects non-IDs | with a `CHECK` | length only | always |
| Generate in SQL | no | no | `gen_souvenir('user')` |

Values are the 20-byte binary form from the spec: fixed length, no varlena
header, compared byte by byte. Text input and output use the canonical string,
and the binary wire format (`COPY BINARY`, binary query results) is the same
20 bytes, so every client can read it, as text if nothing else.

## Running it

**Docker.** The official Postgres image with the extension installed and
created in the default database and in `template1`, so every database has it:

```bash
docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=postgres ghcr.io/bjzsh/souvenir-postgres:18
```

Tags: `14` to `18` (amd64 and arm64), tracking `main`; `<pg>-<version>` for
releases, such as `18-0.7.0`; `latest` is `18`. Mount a volume at
`/var/lib/postgresql` (Postgres 18) or `/var/lib/postgresql/data` (older) as
with the official image.

To add the extension to your own Postgres image, build
[`Dockerfile`](Dockerfile) with a different base, or copy its two stages.

**From source.** Needs Rust, `cargo-pgrx` and the server headers for your
Postgres (`postgresql-server-dev-18` on Debian and Ubuntu):

```bash
cargo install cargo-pgrx --version 0.19.4 --locked
```

```bash
cargo pgrx init --pg18 "$(which pg_config)"
```

```bash
cd souvenir-postgres && cargo pgrx install --release --no-default-features --features pg18
```

Then `CREATE EXTENSION souvenir;` in each database (or in `template1`).

Hosted services (RDS, Cloud SQL, Supabase, Neon, ...) only allow extensions
from their own lists; there, store IDs as text or bytes instead.

## SQL reference

### Type

`souvenir`: 20 bytes, `STORAGE = plain`. B-tree and hash operator classes are
the defaults, so primary keys, unique constraints, `ORDER BY`, merge and hash
joins, `DISTINCT` and hash indexes all work.

### Casts

| From | To | |
|---|---|---|
| `text`, `varchar` | `souvenir` | on assignment (`INSERT ... VALUES ($1)` with a text parameter works); elsewhere write `$1::souvenir` |
| `souvenir` | `text` | on assignment |
| `souvenir` | `bytea` | explicit: the 20 bytes |
| `bytea` | `souvenir` | explicit; fails unless 20 valid bytes |

### Functions

| Function | Returns | |
|---|---|---|
| `gen_souvenir(prefix text)` | `souvenir` | a new ID, from the current time and the OS random source |
| `souvenir_prefix(id)` | `text` | `'user'` |
| `souvenir_has_prefix(id, prefix text)` | `boolean` | |
| `souvenir_timestamp(id)` | `timestamptz` | when the ID was created (millisecond precision) |
| `souvenir_is_valid(text)` | `boolean` | whether the text is a valid ID |
| `souvenir_min(prefix text)` | `souvenir` | the smallest ID with this prefix |
| `souvenir_max(prefix text)` | `souvenir` | the largest ID with this prefix |
| `souvenir_min(prefix text, at timestamptz)` | `souvenir` | the smallest ID with this prefix created at or after `at` |
| `souvenir_max(prefix text, at timestamptz)` | `souvenir` | the largest ID with this prefix created at or before `at` |
| `souvenir_to_uuid(id)` | `uuid` | the timestamp and random bits as a UUIDv7 (the prefix is dropped) |
| `souvenir_from_uuid(prefix text, uuid)` | `souvenir` | the inverse |
| `souvenir_to_bytea(id)`, `souvenir_from_bytea(bytea)` | | the casts above, as functions |

Invalid input fails with SQLSTATE `22P02` (text), `22P03` (binary) or `22023`
(an invalid prefix argument).

### Patterns

Constrain a column to one kind of ID:

```sql
user_id souvenir NOT NULL REFERENCES users CHECK (souvenir_has_prefix(user_id, 'user'))
```

Select by kind, or by kind and creation time, with an index range scan on the
primary key (IDs sort by prefix, then time):

```sql
SELECT * FROM events WHERE id BETWEEN souvenir_min('click') AND souvenir_max('click');

SELECT * FROM events
WHERE id BETWEEN souvenir_min('click', now() - interval '1 hour') AND souvenir_max('click');
```

## From Rust

Declare `storage = Native` on the `Tagged` type, and `Id<T>` reads and writes
the type in its binary form with sqlx or diesel:

```rust
use souvenir::tagged;

tagged!(pub Order = "order", storage = Native);
```

With diesel, use `souvenir::sql_types::Souvenir` as the column's SQL type. `diesel
print-schema` generates its own type for `souvenir` columns; to use this one,
add it to `import_types` in `diesel.toml`:

```toml
[print_schema]
import_types = ["diesel::sql_types::*", "souvenir::sql_types::*"]
```

An untyped `Souvenir` reads from these columns and is written as text, which
converts on assignment. In a `WHERE` clause, cast it: `id = $1::souvenir`.

## Development

Tests run inside Postgres with `cargo pgrx test pg18` (as root, add
`--runas postgres --pgdata /tmp/pgrx`). The database integration tests in
`integration-tests/` exercise the type through sqlx and diesel when
`SOUVENIR_TEST_PG_NATIVE_URL` points at a server with the extension, such as
the Docker image.
