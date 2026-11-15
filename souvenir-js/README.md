# souvenir.js

Type-safe, sortable, prefixed identifiers like
`user_00t35pg1sg9ech32tymrkdq6v`.

IDs are plain strings at run time, so they work as `Map` keys, compare with
`===`, and serialize to JSON as themselves. In TypeScript they carry their
prefix in the type. The package is plain TypeScript with no dependencies and
nothing to initialize.

```ts
import { typed, SouvenirError, type SouvenirId } from 'souvenir.js';

const UserId = typed('user');
const OrgId = typed('org');
type UserId = SouvenirId<'user'>;

const id: UserId = UserId.new();         // 'user_00t35pg1sg...'
const parsed = UserId.parse(input);      // throws unless input is a user ID
if (UserId.is(value)) { /* value: UserId */ }

function loadUser(id: UserId) { /* ... */ }
loadUser(OrgId.new());                   // type error: not a user ID
```

`typed()` throws `InvalidPrefix` straight away for a bad prefix. Prefixes are
1-8 lowercase letters.

## Functions

Each function either returns a value or throws a `SouvenirError`.

| Function | Returns |
|---|---|
| `newId(prefix)` | a new ID |
| `newMonotonicId(prefix)` | a new ID greater than every earlier one with the same prefix, even within one millisecond |
| `parse(text)` | `text` as an ID, if valid |
| `parseExpecting(prefix, text)` | `text` as an ID, if valid and with that prefix |
| `isValid(value)` | whether `value` is a valid ID string; never throws |
| `fromParts(prefix, timestampMs, random)` | an ID from a safe-integer timestamp and a 74-bit `bigint` |
| `prefixOf(id)`, `timestampOf(id)`, `dateOf(id)`, `randomOf(id)` | an ID's fields: prefix, Unix milliseconds, `Date`, 74-bit `bigint` |
| `toBytes(id)`, `fromBytes(bytes)` | the 20-byte binary form (`Uint8Array`) |
| `toUuid(id)`, `fromUuid(prefix, uuid)` | the UUIDv7 form (text or 16 bytes); the prefix is dropped and must be passed back |
| `compare(a, b)` | plain string comparison, which is also binary order |
| `typed(prefix)` | `new`, `newMonotonic`, `parse`, `is`, `fromParts`, `fromBytes` and `fromUuid` for one prefix |
| `randomGenerator(options)`, `monotonicGenerator(options)` | a generator with its own clock and random source; see [Generators](#generators) |

IDs sort by prefix, then by creation time: plain `Array.prototype.sort()`
puts IDs of one prefix in creation order. IDs from `newId` in the same
millisecond are in random order; use `newMonotonicId` when that matters.

TypeScript does not allow `<` between IDs of different prefixes, because
they have different types. Use `compare`.

## Generators

`newId` and `newMonotonicId` read `Date.now()` and `crypto.getRandomValues`.
To supply either yourself, for example to make a test deterministic, make a
generator:

```ts
import { monotonicGenerator, randomGenerator } from 'souvenir.js';

const zeros = (bytes: Uint8Array) => { bytes.fill(0); };
const generate = randomGenerator({ now: () => 1_793_531_580_000, random: zeros });
generate('user'); // 'user_00t35pg1sg000000000000000'

const next = monotonicGenerator(); // its own state, the system's sources
```

Both return an `IdGenerator`: a function from a prefix to a new ID.

`now` returns Unix milliseconds. `random` fills the bytes it is given, like
`crypto.getRandomValues`, and outside tests must be cryptographically
secure. Each `monotonicGenerator()` keeps its own state; `newMonotonicId`
uses one shared by the whole module.

IDs are identifiers, not secrets: do not use one as a token or an
unguessable link.

## Errors

Errors are `SouvenirError`s, with a stable `code`:

```ts
try {
  UserId.parse('org_00t35pg1sgajz7jf1pz84mv4e');
} catch (error) {
  if (error instanceof SouvenirError) console.log(error.code); // 'PrefixMismatch'
}
```

The codes are the ones in the
[specification](https://github.com/bjzsh/souvenir/blob/main/SPEC.md#9-errors),
plus `InvalidType` for an argument of the wrong JavaScript type (a
non-string ID, or bytes that are not a `Uint8Array`). `SouvenirErrorCode` is
the type of `code`. Compare codes, not messages, whose wording may change.

## Requirements

- Node.js 20 or later (ES modules or CommonJS), Bun, Deno, or any current
  browser.
- `BigInt`, and `crypto.getRandomValues` for generating IDs. In browsers
  that means HTTPS or localhost.

The package ships both ES modules and CommonJS. If a program loads both
builds, each has its own `newMonotonicId` state and its own `SouvenirError`
class, so `instanceof` is false for an error thrown by the other build.
`error.code` works either way.

## Development

```sh
bun install
bun run build       # dist/esm and dist/cjs
bun run typecheck
bun run test        # the last tests load the built package
```

The tests run the conformance vectors in
[`tests/fixtures/souvenir_v2_vectors.json`](https://github.com/bjzsh/souvenir/blob/main/tests/fixtures/souvenir_v2_vectors.json),
which the Rust crate runs too.
