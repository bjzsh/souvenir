# Souvenir identifier format

A souvenir identifier is a 128-bit unsigned value partitioned into a prefix rank
and a numeric suffix. All 2^128 values are valid. The binary representation is
exactly 16 bytes in big-endian order.

## Canonical text

The text representation is `prefix_suffix`:

- `prefix`: 1–4 lowercase ASCII letters (`a` through `z`).
- `_`: exactly one separator.
- `suffix`: exactly 22 digits from `0123456789abcdefghjkmnpqrstvwxyz`.

The suffix is the numeric suffix written in base 32, most significant digit first,
with leading zeroes to reach 22 digits. Digits `i`, `l`, `o`, and `u` are excluded.
Parsing is case-sensitive and accepts neither whitespace nor character aliases.

A suffix's digits must decode to a value within the selected prefix's bound.
Twenty-two digits can represent 110 bits, but the usable suffix range is smaller.

Example: `user_02v58c5a3fy30k560qrtg4`.

## Prefix ranks

There are:

```text
N = 26 + 26^2 + 26^3 + 26^4 = 475254
```

Prefixes have zero-based ranks in ASCII dictionary order. A prefix sorts before
all of its extensions:

```text
rank 0: a
rank 1: aa
rank 2: aaa
rank 3: aaaa
rank 4: aaab
...
rank 475253: zzzz
```

Let `d[i]` be a letter's zero-based value (`a = 0`, ..., `z = 25`), and let `L`
be the prefix length. Its rank is:

```text
S = [18279, 703, 27, 1]
rank = (L - 1) + sum(d[i] * S[i], i = 0 .. L - 1)
```

To recover a prefix, start with its rank and visit the subtree sizes in `S`.
Append the letter indexed by `rank / S[i]`. If `rank % S[i]` is zero, stop;
otherwise replace `rank` with `(rank % S[i]) - 1` and continue.

`Prefix::to_u32()` returns this rank. Valid ranks are `0..475254`.

## Suffix intervals

Divide the complete unsigned 128-bit space into contiguous prefix intervals:

```text
Q = floor(2^128 / N) = 716001058215056503392658678163188
R = 2^128 mod N     = 461704
```

The first `R` prefixes each receive `Q + 1` values. The remaining prefixes each
receive `Q` values. For prefix rank `p`:

```text
size(p)  = Q + 1                  if p < R
           Q                      otherwise
start(p) = p * Q + min(p, R)
0 <= suffix < size(p)
value    = start(p) + suffix
```

Thus the maximum suffix is `Q` for ranks below `R`, and `Q - 1` for the rest.
The last prefix with the larger interval is `zgsv` (rank 461703); the first with
the smaller interval is `zgsw` (rank 461704).

| Prefix | Maximum numeric suffix | Maximum encoded suffix |
| --- | --- | --- |
| `a` through `zgsv` | `Q` | `hmtd1hxh33gedanpsmt9qm` |
| `zgsw` through `zzzz` | `Q - 1` | `hmtd1hxh33gedanpsmt9qk` |

`Id::from_parts` and `Id::with_prefix` reject suffixes outside the destination
prefix's interval. Replacing a prefix preserves the suffix, but changes the full
128-bit value.

## Decoding the binary value

Set `split = R * (Q + 1)`. Given the unsigned value `v`:

```text
if v < split:
    prefix_rank = v / (Q + 1)
    suffix      = v mod (Q + 1)
else:
    offset      = v - split
    prefix_rank = R + offset / Q
    suffix      = offset mod Q
```

All divisions are integer divisions. These formulas are inverse to encoding and
cover the full value range without gaps or overlapping intervals.

Binary bytes are the big-endian encoding of `v`; they do not contain independent
prefix and suffix bit fields. `Id::new`, integer conversions, and byte-array
conversions preserve these bytes. Signed `i128` conversions reinterpret the same
bits using two's-complement representation.

## Ordering

The following orders agree:

- Prefix dictionary order, then numeric suffix.
- `Id::Ord`.
- The unsigned 128-bit value.
- Lexicographic comparison of the 16 binary bytes.
- Bytewise lexicographic comparison of canonical text.

The separator `_` sorts before lowercase ASCII letters, so a shorter prefix's
IDs sort before IDs with an extension of that prefix. The suffix alphabet is in
ASCII order, and suffixes have fixed width.

Locale-aware text sorting and signed integer sorting are not equivalent to these
orders. Database text ordering requires an appropriate binary collation.

## Random generation

Generation selects each suffix in the requested prefix's interval with equal
probability. The entropy for a fixed prefix is `log2(size(p))`, approximately
109.142 bits.

Rust uses unbiased bounded sampling from the prefix's full numeric interval.
JavaScript bindings sample 110 random bits and reject suffixes outside the bound.
Taking an unrestricted random value modulo the interval size would introduce bias.

## Serialization and storage

| Representation | Encoding |
| --- | --- |
| Rust `[u8; 16]`, JavaScript `Uint8Array` | Full value in big-endian order |
| Rust `u128`, JavaScript `bigint` | Full unsigned value |
| Rust `i128` | Full value's bits interpreted as a signed integer |
| Text, human-readable Serde, JavaScript JSON | Canonical `prefix_suffix` string |
| Binary Serde `Id` | Array of 16 bytes |
| Human-readable Serde `Prefix` | Prefix string |
| Binary Serde `Prefix` | Unsigned 32-bit prefix rank |
| PostgreSQL | UUID containing the 16 bytes |
| MySQL | Binary containing the 16 bytes |
| SQLite | Canonical text |

A PostgreSQL UUID's textual rendering is a hexadecimal rendering of the binary
bytes, not the canonical souvenir text. Binary serializers may add their own
framing around arrays and integers.

## Fixed vectors

Hex values below are both the full unsigned value and its 16 big-endian bytes.

| Canonical text | Full value in hexadecimal |
| --- | --- |
| `a_0000000000000000000000` | `00000000000000000000000000000000` |
| `user_02v58c5a3fy30k560qrtg4` | `cbcee7b695fe097e9d18ec46bc97fe46` |
| `zzzz_hmtd1hxh33gedanpsmt9qk` | `ffffffffffffffffffffffffffffffff` |

`zgsv_hmtd1hxh33gedanpsmt9qm` is valid. The same suffix with prefix `zgsw` is
invalid; `zgsw_hmtd1hxh33gedanpsmt9qk` is valid.
