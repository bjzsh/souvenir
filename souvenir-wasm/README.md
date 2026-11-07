# souvenir.js

Type-safe, sortable, prefixed identifiers like
`user_00t35pg1sg9ech32tymrkdq6v`, implemented in Rust and WebAssembly.

IDs are plain strings at run time, so they work as `Map` keys, compare with
`===`, and serialize to JSON as themselves. In TypeScript they carry their
prefix in the type. The package initializes automatically when imported.

```ts
import { typed, type SouvenirError, type SouvenirId } from 'souvenir.js';

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
| `newMonotonicId(prefix)` | a new ID greater than every earlier result of this function, even within one millisecond |
| `parse(text)` | `text` as an ID, if valid |
| `parseExpecting(prefix, text)` | `text` as an ID, if valid and with that prefix |
| `isValid(value)` | whether `value` is a valid ID string; never throws |
| `fromParts(prefix, timestampMs, random)` | an ID from a safe-integer timestamp and a 74-bit `bigint` |
| `prefixOf(id)`, `timestampOf(id)`, `dateOf(id)`, `randomOf(id)` | an ID's fields: prefix, Unix milliseconds, `Date`, 74-bit `bigint` |
| `toBytes(id)`, `fromBytes(bytes)` | the 20-byte binary form (`Uint8Array`) |
| `toUuid(id)`, `fromUuid(prefix, uuid)` | the UUIDv7 form (text or 16 bytes); the prefix is dropped and must be passed back |
| `compare(a, b)` | plain string comparison, which is also binary order |
| `typed(prefix)` | `new`, `newMonotonic`, `parse`, `is`, `fromParts`, `fromBytes` and `fromUuid` for one prefix |

IDs sort by prefix, then by creation time: plain `Array.prototype.sort()`
puts IDs of one prefix in creation order. IDs from `newId` in the same
millisecond are in random order; use `newMonotonicId` when that matters.

## Errors

Errors are standard `Error`s with `name: 'SouvenirError'` and a stable
`code`:

```ts
try {
  UserId.parse('org_00t35pg1sgajz7jf1pz84mv4e');
} catch (error) {
  console.log((error as SouvenirError).code); // 'PrefixMismatch'
}
```

The codes are the ones in the [specification](../SPEC.md#9-errors), plus
`InvalidType` for an argument of the wrong JavaScript type (a non-string ID,
or bytes that are not a `Uint8Array`). `SouvenirError` and
`SouvenirErrorCode` are exported TypeScript types.

## Supported consumers

- Node.js 22.12+ (ESM or CommonJS).
- Vite 8+ in development and production, with no WASM plugin required.
- Webpack 5 in production, with no WASM experiment required.
- Native browser ES modules, using the generated embedded-WASM entry.

Browsers need WebAssembly, BigInt and cryptographic randomness (use HTTPS or
localhost). The bundler entry uses top-level await internally; application
code needs no initialization call. It emits WASM as an asset, so include the
generated assets in deployments and serve `.wasm` as `application/wasm`.

For native browsers, serve the complete package directory at
`/vendor/souvenir/` and import the embedded-WASM entry:

```html
<script type="module">
  import { newId } from '/vendor/souvenir/lib/wrapper/esm/web.js';
  console.log(newId('user'));
</script>
```

## Development

```sh
bun install
bun run build                 # needs wasm-opt (Binaryen) on PATH
bun run build --no-wasm-opt   # local build without Binaryen
bun run test
```
