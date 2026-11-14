# Souvenir ID format, version 2

This document specifies the Souvenir ID format. The conformance vectors in
[`tests/fixtures/souvenir_v2_vectors.json`](tests/fixtures/souvenir_v2_vectors.json)
are normative alongside it: every implementation must pass all of them, with
exactly the error codes they give. If this document and the vectors disagree,
the vectors win and this document is corrected.

Sections 1 to 9 are normative. Sections 10 to 12 are not.

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
format. There are no version bits. Reserved values have no text form, so
only decoding bytes can fail with `ReservedPrefix`.

### 3.2 No nil ID

There is no nil or zero ID. A nil would be a value of the ID type that is
not an ID. Represent absence outside the type: `Option`, `null`, a nullable
column, an absent field.

All-zero bytes never decode as an ID, because prefix rank 0 is not any
prefix. Zeroed memory, a defaulted column or a dropped field therefore fails
with `ZeroPrefix` instead of becoming a valid-looking ID.

### 3.3 Timestamp

Unsigned milliseconds since the Unix epoch (UTC), 48 bits, valid until about
the year 10889. Decoders accept every 48-bit value.

### 3.4 Random

74 bits. Every value is valid on decode.

### 3.5 Building from parts

Checks run in this order, and the first failure is reported:

```text
from_parts(prefix, ts, random):
    validate prefix                       else InvalidPrefix
    if ts >= 2^48:                        error TimestampOutOfRange
    if random >= 2^74:                    error RandomOutOfRange
    return (rank(prefix) << 122) | (ts << 74) | random
```

### 3.6 Decoding bytes

Checks run in this order, and the first failure is reported:

```text
from_bytes(b):
    if len(b) != 20:                      error InvalidLength
    p = the top 38 bits of b
    if p == 0:                            error ZeroPrefix
    if p >= PREFIX_RANK_LIMIT:            error ReservedPrefix
    return b
```

Nothing else can fail: every timestamp and every random value is valid.

## 4. Prefixes

A prefix is 1 to 8 characters, each in `a`-`z`. Nothing else is allowed: no
digits, underscores or uppercase letters. That restriction is what keeps text
order and binary order identical (section 12.2).

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

The checks run in that order: a suffix of the wrong length is
`InvalidLength` even if it also holds invalid characters, and an invalid
character anywhere is reported before `NonCanonical`.

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
2. Find the **last** `_`. None -> `MissingSeparator`.
3. Validate the prefix (left of the separator) -> `InvalidPrefix`.
4. Decode the suffix (right of the separator) per section 5 -> `InvalidLength`,
   `InvalidCharacter` or `NonCanonical`.
5. Return `(rank(prefix) << 122) | body`.

The ASCII check comes first so that every later length means the same thing
in every language. Languages count the length of a non-ASCII string
differently (bytes, UTF-16 units, code points); for ASCII they all agree.

Splitting on the last `_` is safe because `_` is not in the suffix alphabet.
It also means `user_auth_...` is reported as an invalid prefix.

There is no separate limit on the length of the input. Text that is too long
fails at whichever step it reaches: with no `_` it is `MissingSeparator`,
with a long prefix `InvalidPrefix`, with a long suffix `InvalidLength`.

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

A monotonic generator guarantees that the body (timestamp, then random bits)
of each ID it returns is greater than the body of the one before, including
when the clock stalls or goes backwards. IDs with the same prefix are
therefore strictly increasing.

IDs with different prefixes are not: an ID's prefix sorts before its body, so
an `org` ID is less than every `user` ID, whenever each was made. To order
IDs of different prefixes by creation, compare their bodies.

The generator's state is the body of the last ID it returned, shared across
all prefixes. It starts empty.

```text
state: last = none                             # else (last_t, last_r)

next(prefix):
    validate prefix; read and range-check t as in section 7.1
    lock:
        if last is none or t > last_t:
            r = csprng_bits(74)
        else:                                  # same ms, or clock went back
            t = last_t
            r = last_r + 1 + csprng_bits(32)   # random step in [1, 2^32]
            if r >= 2^74:                      # overflow: borrow the next ms
                if last_t == 2^48 - 1:         error TimestampOutOfRange
                t = last_t + 1
                r = csprng_bits(74)
        last = (t, r)
    return (rank(prefix) << 122) | (t << 74) | r
```

- The state changes only when an ID is returned. A failing clock or random
  source, or running past the end of the timestamp range, leaves it as it
  was.
- Timestamps may run slightly ahead of the wall clock under bursts or clock
  rollback.
- The guarantee holds per generator instance. Across instances, IDs from the
  same millisecond are in arbitrary order.
- IDs from the two generators are indistinguishable.
- The random step makes the next ID one of 2^32 values, not one of 2^74
  (section 10).

The `monotonic` vectors script a clock and the values `csprng_bits` returns,
and give the ID each call must produce.

### 7.3 Collisions

Two IDs can collide only if they have the same prefix and the same
millisecond. With 74 random bits, one prefix generating 1 million IDs per
second is expected to see about 8 * 10^-7 collisions per year; at 100 million
per second, about 8 * 10^-3. These are the same figures as UUIDv7.

## 8. UUIDv7 bridge

The 122-bit body maps bit-for-bit onto a UUIDv7 (RFC 9562). This lets a
column whose type already implies the prefix store IDs in a native 16-byte
`uuid`.

```text
to_uuid(id):
    ts = timestamp, r = random
    uuid = (ts << 80)              # unix_ts_ms (48)
         | (0x7 << 76)             # version 7  (4)
         | ((r >> 62) << 64)       # rand_a = top 12 bits of r
         | (0b10 << 62)            # variant    (2)
         | (r & (2^62 - 1))        # rand_b = low 62 bits of r

from_uuid(prefix, uuid):
    validate prefix                          else InvalidPrefix
    require ((uuid >> 76) & 0xF) == 7        else NotUuidV7
    require ((uuid >> 62) & 0b11) == 0b10    else NotUuidV7
    ts = uuid >> 80
    r  = (((uuid >> 64) & 0xFFF) << 62) | (uuid & (2^62 - 1))
    return from_parts(prefix, ts, r)
```

The round trip is lossless in both directions: every ID has one UUIDv7, and
every UUIDv7 with variant `0b10`, whatever generated it, is the body of one
ID per prefix. An existing UUIDv7 column can therefore be read as Souvenir
IDs without rewriting it. Within one prefix, UUID byte order equals ID order.

A UUID given as bytes must be exactly 16 bytes (`InvalidLength`). Where an
implementation accepts a UUID as text, it accepts only the 36-character
hyphenated form (`xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`), with hex digits in
either case: any other length is `InvalidLength`, and a wrong character is
`InvalidCharacter`. `to_uuid` text is lowercase.

## 9. Errors

Every implementation reports errors with these codes. Messages may differ
between implementations; codes may not.

| Code | When |
|---|---|
| `InvalidPrefix` | A prefix (including the expected prefix of `parse_expecting`) is empty, longer than 8 characters, or contains anything outside `a`-`z` |
| `MissingSeparator` | Text has no `_` |
| `InvalidLength` | Suffix is not 25 characters; bytes are not 20; a UUID is not 16 bytes (or, as text, not 36 characters) |
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

## 10. Security considerations

- **IDs are identifiers, not secrets.** Do not use one as a password, a
  session token or an unguessable link.
- **An ID reveals when it was created,** to the millisecond, and its prefix
  reveals what kind of thing it identifies.
- **Default IDs are hard to guess:** 74 bits from a CSPRNG per millisecond.
- **Monotonic IDs are easier to guess.** Within one millisecond, each ID is
  the one before plus a step of at most 2^32, so someone who has seen one ID
  can find the next in about 2^32 attempts. Authorize every access to a
  record; never rely on its ID being unknown.
- **Never substitute a weaker random source.** A predictable source makes
  IDs both guessable and likely to collide.

## 11. Storage guidance

The recommended default everywhere is the **canonical text form**: it is the
same in every table, API and log, joins with plain equality, and stays
readable and self-describing without a Souvenir library.

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

## 12. Rationale

### 12.1 Field sizes

- **48 + 74 bits** are the timestamp and random bits of a UUIDv7, which is
  what makes the bridge in section 8 lossless. Together they are 122 bits,
  which fit in 25 Base32 characters.
- **38 bits** is the smallest field that holds every prefix of up to 8
  letters (`S[8] < 2^38`), and brings the total to 160 bits: exactly 20
  bytes.
- **8 letters** is the longest prefix that fits. A ninth letter needs 43
  bits, which no longer fits 20 bytes.

### 12.2 Why text order equals binary order

Take two IDs with prefixes `p` and `q`.

- **Same prefix.** The texts differ only in their suffixes, which have the
  same length and an alphabet in ascending order, so they compare as the
  bodies do. The binary forms share their top 38 bits and compare by the
  rest, which is the body.
- **`p` is the start of a longer `q`.** Where `p`'s text has `_`, `q`'s has a
  letter, and `_` (0x5F) is less than every letter (0x61 and up), so `p`'s ID
  is first. In binary, trie preorder gives `p` the smaller rank.
- **Otherwise** the prefixes first differ at some letter, which decides the
  order of the texts and, through the rank, of the binary forms.

This is why prefixes hold only `a`-`z`: an uppercase letter, a digit or an
underscore sorts at or below the separator, and would break the second case.
