/// How a kind of typed ID is stored in a database.
///
/// Each [`Tagged`](crate::Tagged) type declares a form, so every column holding that kind of
/// ID (its table's key and every foreign key to it) uses the same form, and
/// joins between them work with plain equality. Code just uses `Id<T>`.
///
/// ```
/// use souvenir::tagged;
///
/// tagged!(pub User = "user");                    // text, the default
/// tagged!(pub Event = "event", storage = Bytes); // 20 bytes
/// tagged!(pub Account = "acct", storage = Uuid); // native uuid
/// ```
///
/// | `Storage` | Postgres | MySQL         | SQLite        |
/// |-----------|----------|---------------|---------------|
/// | `Text`    | `text`   | `VARCHAR(34)` | `TEXT`        |
/// | `Bytes`   | `bytea`  | `BINARY(20)`  | `BLOB` (20 B) |
/// | `Uuid`    | `uuid`   | `BINARY(16)`  | `BLOB` (16 B) |
///
/// [`Souvenir`](crate::Souvenir) has no `Tagged` type, so it is stored as
/// text. To store one in another form, bind `id.as_bytes()` and read with
/// [`Souvenir::from_slice`](crate::Souvenir::from_slice) (diesel also maps
/// `Souvenir` to `Binary` columns directly).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Storage {
    /// The ID string, the same as in JSON, URLs and logs. Readable,
    /// self-describing, and identical in every table. Use a byte-order
    /// collation so the database sorts IDs in ID order.
    Text,
    /// The 20-byte binary form. Smaller than text and still
    /// self-describing; for large tables where index size matters.
    Bytes,
    /// A UUIDv7 (16 bytes). Only for columns that must be native UUIDs:
    /// the prefix is not stored, so anyone reading the column directly sees
    /// a bare UUID, and reading it through another `Id<U>` silently gives the
    /// IDs `U`'s prefix.
    Uuid,
}
