# Souvenir ID format, version 2

This document specifies the Souvenir ID format. The conformance vectors in
[`tests/fixtures/souvenir_v2_vectors.json`](tests/fixtures/souvenir_v2_vectors.json)
are normative alongside it: every implementation must pass all of them, with
exactly the error codes they give.

## 1. Overview

A Souvenir ID is an unsigned 160-bit integer made of three fields:

```text
 bit 159                                                                   bit 0
 +----------------------+--------------------------+---------------------------+
 | prefix rank (38)     | unix_ts_ms (48)          | random (74)               |
 +----------------------+--------------------------+---------------------------+
   V    = (rank << 122) | (ts << 74) | random
   body = V & (2^122 - 1)
```

Its text form is the prefix, an underscore, and 25 Base32 characters encoding
the 122-bit body:

```text
user_00t35pg1sg9ech32tymrkdq6v
```

**Core invariant:** comparing two IDs' binary forms byte by byte gives the
same result as comparing their text forms byte by byte, for any two IDs. IDs
therefore sort by prefix, then by timestamp, then by random bits.

## 2. Constants

```text
MAX_PREFIX_LEN    = 8
PREFIX_BITS       = 38
TIMESTAMP_BITS    = 48
RANDOM_BITS       = 74
BODY_BITS         = 122
TOTAL_BITS        = 160
BYTE_LEN          = 20
SUFFIX_LEN        = 25
SEPARATOR         = '_'   (0x5F)
ALPHABET          = "0123456789abcdefghjkmnpqrstvwxyz"

S[m]              = sum of 26^k for k = 0..m
S                 = [1, 27, 703, 18279, 475255, 12356631, 321272407,
                     8353082583, 217180147159]
PREFIX_RANK_LIMIT = S[8] = 217180147159   (exclusive)
```

## 3. Binary form

The ID is serialized big-endian to exactly 20 bytes. Fields do not fall on
byte boundaries (the prefix ends inside byte 4).

Comparing the 20-byte arrays lexicographically is the same as comparing the
integers `V`. Equality and hashing are defined on the bytes.

### 3.1 Prefix field

| Value `p` | Meaning |
|---|---|
| `0` | **Invalid** (`ZeroPrefix`), whatever the body. |
| `1 ... 217180147158` | A valid prefix, `unrank(p)`. |
| `217180147159 ... 2^38 - 1` | **Reserved** for future formats. Decoders reject these (`ReservedPrefix`); generators never produce them. |

The reserved range is the extension mechanism for future versions of the
format. There are no version bits.

### 3.2 No nil ID

There is no nil or zero ID. A nil would be a value of the ID type that is
not an ID. Represent absence outside the type: `Option`, `null`, a nullable
column, an absent field.

All-zero bytes never decode as an ID, because prefix rank 0 is not any
prefix. Zeroed memory, a defaulted column or a dropped field therefore fails
with `ZeroPrefix` instead of becoming a valid-looking ID.

Bindings for languages where a zero value cannot be avoided (such as a Go
struct) may expose an `IsZero()` check, and must make formatting or encoding
the zero value an error. No other API may accept or produce it.

### 3.3 Timestamp

Unsigned milliseconds since the Unix epoch (UTC), 48 bits, valid until about
the year 10889. Decoders accept every 48-bit value.

### 3.4 Random

74 bits. Every value is valid on decode.

## 4. Prefixes

A prefix is 1 to 8 characters, each in `a`-`z`. Nothing else is allowed: no
digits, underscores or uppercase letters. That restriction is what keeps text
order and binary order identical.

A prefix is stored as its **rank in trie preorder**: lexicographic order,
where a string comes before every longer string that starts with it
(`u < us < user < users < v`).

With `c_i = s[i] - 'a'` and positions counted from 1:

```text
rank(s) = sum over i = 1..len(s) of (1 + c_i * S[8 - i])
```

```text
function rank(s):
    require 1 <= len(s) <= 8 and every char in 'a'..'z'   else InvalidPrefix
    r = 0
    for i in 1..len(s):
        r += 1 + (s[i] - 'a') * S[8 - i]
    return r

function unrank(r):
    if r == 0:                  error ZeroPrefix
    if r >= PREFIX_RANK_LIMIT:  error ReservedPrefix
    out = ""
    i = 1
    while r > 0:
        r -= 1
        size = S[8 - i]
        out += char('a' + r / size)     # integer division
        r = r mod size
        i += 1
    return out
```

All values fit in 64 bits. Examples: `a` = 1, `aa` = 2, `b` = 8353082584,
`user` = 172902060849, `users` = 172902389872, `zzzzzzzz` = 217180147158.

## 5. Suffix

The suffix encodes the 122-bit body as 25 symbols of 5 bits each, most
significant first. The top 3 of those 125 bits are always zero, so the first
suffix character is always `0`, `1`, `2` or `3`.

```text
encode(body):
    for i in 0..24:
        out[i] = ALPHABET[(body >> (5 * (24 - i))) & 31]

decode(s):
    if len(s) != 25:          error InvalidLength
    v = 0
    for ch in s:
        if ch not in ALPHABET: error InvalidCharacter
        v = (v << 5) | index(ch)
    if v >> 122 != 0:         error NonCanonical
    return v
```

Decoding is strict:

- Lowercase only. `A`-`Z` are invalid characters.
- `i`, `l`, `o` and `u` are not in the alphabet and are not aliased.
- No hyphens, padding, whitespace or check symbols.
- Every ID has exactly one text form: `format(parse(t)) == t` for every
  accepted `t`.

The alphabet is in ascending ASCII order, so suffix order equals numeric
order.

## 6. Text form

### 6.1 Grammar

```text
id      = prefix "_" suffix
prefix  = 1*8( %x61-7A )                  ; a-z
suffix  = %x30-33 24( alphabet-char )     ; first char 0-3
```

As an anchored regular expression:
`^[a-z]{1,8}_[0-3][0-9a-hjkmnp-tv-z]{24}$`. Text is 27 to 34 ASCII
characters.

### 6.2 Formatting

`format(id) = unrank(rank) + "_" + encode(body)`. Formatting never fails.

### 6.3 Parsing

Checks run in exactly this order, and the first failure is reported, so every
implementation returns the same error for the same input:

1. Input is not ASCII -> `InvalidCharacter`.
2. Input is longer than 64 characters -> `InvalidLength`.
3. Find the **last** `_`. None -> `MissingSeparator`.
4. Validate the prefix (left of the separator) -> `InvalidPrefix`.
5. Decode the suffix (right of the separator) per section 5 -> `InvalidLength`,
   `InvalidCharacter` or `NonCanonical`.
6. Return `(rank(prefix) << 122) | body`.

Splitting on the last `_` is safe because `_` is not in the suffix alphabet.
It also means `user_auth_...` is reported as an invalid prefix.

### 6.4 Parsing with an expected prefix

`parse_expecting(prefix, text)`:

1. Validate the **expected** prefix first. If it is invalid -> `InvalidPrefix`,
   whatever `text` is. An invalid expected prefix is a programming error and
   is never reported as a problem with the input.
2. Parse `text` per section 6.3.
3. If the prefixes differ -> `PrefixMismatch`.

Typed APIs should make step 1 impossible to fail where the language allows
it.

## 7. Generation

### 7.1 Default

```text
new(prefix):
    validate prefix                       else InvalidPrefix
    t = wall-clock Unix milliseconds
    if t < 0 or t >= 2^48:                error ClockFailure
    (clock unavailable:                   error ClockFailure)
    r = 74 bits from a CSPRNG             failure: RandomSourceFailure
    return (rank(prefix) << 122) | (t << 74) | r
```

The default generator is stateless and safe across threads, processes and
machines. A failing random source is reported as an error; implementations
never fall back to a weaker source.

### 7.2 Monotonic

A monotonic generator guarantees that each ID it returns is greater than the
previous one, including when the clock stalls or goes backwards. Its state,
`(last_t, last_r)`, is shared across all prefixes.

```text
new(prefix):
    validate prefix; read and range-check t as above
    lock:
        if t > last_t:
            r = csprng_bits(74)
        else:                                  # same ms, or clock went back
            t = last_t
            r = last_r + 1 + csprng_bits(32)   # random step in [1, 2^32]
            if r >= 2^74:                      # overflow: borrow the next ms
                t = last_t + 1
                if t >= 2^48: error TimestampOutOfRange
                r = csprng_bits(74)
        last_t, last_r = t, r
    return (rank(prefix) << 122) | (t << 74) | r
```

- Timestamps may run slightly ahead of the wall clock under bursts or clock
  rollback.
- The guarantee holds per generator instance. Across instances, IDs from the
  same millisecond are in arbitrary order.
- The random step keeps successive IDs hard to guess.
- IDs from the two generators are indistinguishable.

### 7.3 Collisions

Two IDs can collide only if they have the same prefix and the same
millisecond. With 74 random bits, one prefix generating 1 million IDs per
second is expected to see about 8 * 10^-7 collisions per year; at 100 million
per second, about 8 * 10^-3. These are the same figures as UUIDv7.

## 8. UUIDv7 bridge

The 122-bit body maps bit-for-bit onto a UUIDv7. This lets a column whose
type already implies the prefix store IDs in a native 16-byte `uuid`.

```text
to_uuid(id):
    ts = timestamp, r = random
    uuid = (ts << 80)              # unix_ts_ms (48)
         | (0x7 << 76)             # version 7  (4)
         | ((r >> 62) << 64)       # rand_a = top 12 bits of r
         | (0b10 << 62)            # variant    (2)
         | (r & (2^62 - 1))        # rand_b = low 62 bits of r

from_uuid(prefix, uuid):
    require ((uuid >> 76) & 0xF) == 7        else NotUuidV7
    require ((uuid >> 62) & 0b11) == 0b10    else NotUuidV7
    ts = uuid >> 80
    r  = (((uuid >> 64) & 0xFFF) << 62) | (uuid & (2^62 - 1))
    return from_parts(prefix, ts, r)
```

The round trip is lossless. Within one prefix, UUID byte order equals ID
order.

## 9. Errors

Every implementation reports errors with these codes. Messages may differ
between implementations; codes may not.

| Code | When |
|---|---|
| `InvalidPrefix` | A prefix (including the expected prefix of `parse_expecting`) is empty, longer than 8 characters, or contains anything outside `a`-`z` |
| `MissingSeparator` | Text has no `_` |
| `InvalidLength` | Suffix is not 25 characters; bytes are not 20; a UUID is not 16 bytes; text is over 64 characters |
| `InvalidCharacter` | Text is not ASCII, or a suffix character is outside the alphabet |
| `NonCanonical` | The suffix's first character is `4`-`z` (value >= 2^122) |
| `ZeroPrefix` | The prefix field is 0, including all-zero bytes |
| `ReservedPrefix` | The prefix field is >= 217180147159 |
| `PrefixMismatch` | A valid ID has a different prefix than expected |
| `TimestampOutOfRange` | A timestamp is >= 2^48, including a monotonic generator running past the end of the range |
| `RandomOutOfRange` | A random value is >= 2^74 |
| `NotUuidV7` | A UUID does not have version 7 and variant `0b10` |
| `RandomSourceFailure` | The random source failed |
| `ClockFailure` | The clock is unavailable, or reads outside `[0, 2^48)` |

Bindings may add codes for errors specific to their language boundary (for
example `InvalidType` in JavaScript for a non-string argument).

## 10. Storage guidance

Non-normative. The recommended default everywhere is the **canonical text
form**: it is the same in every table, API and log, joins with plain
equality, and stays readable and self-describing without a Souvenir
library.

| Representation | When |
|---|---|
| Text (default) | Any column. Use a byte-order collation (Postgres `COLLATE "C"`, MySQL `ascii_bin`, SQLite's default) so the database sorts IDs in ID order, and a check against the section 6.1 regular expression. Never a case-insensitive or locale-aware collation. |
| 20-byte binary | Large tables where index size matters (Postgres `bytea` with a length check, MySQL `BINARY(20)`). Still self-describing. |
| UUIDv7 (section 8) | Only for columns that must be native UUIDs. The prefix is not stored, so whatever reads the column has to supply it, and nothing detects a wrong one. |
| JSON and HTTP | The canonical string; absence is `null` or an absent field |
| URLs | The canonical string as-is |

A Postgres domain for text IDs:

```sql
CREATE DOMAIN souvenir AS text COLLATE "C"
  CHECK (VALUE ~ '^[a-z]{1,8}_[0-3][0-9a-hjkmnp-tv-z]{24}$');
```
