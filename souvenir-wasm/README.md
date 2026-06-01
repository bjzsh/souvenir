# souvenir.js

A type-safe prefixed identifier library implemented in Rust and WebAssembly.
The package initializes automatically when imported:

```js
import { Id } from 'souvenir.js';

const id = Id.random('user');
console.log(id.toString());
id.free();
```

Node CommonJS also works:

```js
const { Id } = require('souvenir.js');
console.log(Id.random('user').toString());
```

## Supported consumers

- Node.js 22.12+ (ESM or CommonJS).
- Vite 8+ in development and production, with no WASM plugin required.
- Webpack 5 in production, with no WASM experiment required.
- Native browser ES modules, using the generated embedded-WASM entry.

Browsers need WebAssembly, BigInt, and cryptographic randomness (use HTTPS or
localhost). The bundler entry uses top-level await internally; application code
needs no initialization call. It emits WASM as an asset, so include the generated
assets in deployments and serve `.wasm` as `application/wasm`.

For native browsers, serve the complete package directory at `/vendor/souvenir/`:

```html
<script type="module">
  import { Id } from '/vendor/souvenir/lib/esm/web.js';
  console.log(Id.random('user').toString());
</script>
```

That entry embeds the WASM bytes and initializes synchronously on import.
Browsers resolve URLs or import maps, not npm export conditions.

ESM and CommonJS consumers are supported separately; sharing class identities or
WASM objects between those module formats is not supported.

## Working with identifiers

Identifiers are immutable. `prefix` and `suffix` are read-only properties;
`cast()` returns a new identifier with the same suffix:

```js
const user = Id.random('user');
const team = user.cast('team');
console.log(user.prefix); // 'user'
console.log(team.prefix); // 'team'
console.log(user.suffix === team.suffix); // true
```

Use `equals()` for value comparison. JavaScript `===`, `Map`, and `Set` still
compare object identity; use the canonical string as a value-based map key.

```js
const copy = Id.parse(user.toString());
user.equals(copy); // true
JSON.stringify({ id: user }); // {"id":"user_..."}
```

Conversion factories validate their inputs:

```js
Id.fromBytes(user.toBytes()); // exactly 16 bytes, big-endian
Id.fromBigInt(user.toBigInt()); // unsigned 128-bit bigint, valid encoded prefix
Id.tryParse('invalid'); // undefined, rather than throwing
```

`new Id(bytes)` remains available. `parse()` throws for invalid strings;
`tryParse()` accepts `unknown` and returns `undefined` for non-strings or invalid
identifiers; `test()` accepts `unknown` and returns `false` for those inputs.
Throwing string APIs reject non-strings with `InvalidType`. Byte factories accept
only `Uint8Array` (including Node `Buffer`), rejecting other arrays and views
with `InvalidType`.

Validation errors are standard JavaScript errors with `name: 'SouvenirError'` and a
stable `code`. Length errors include numeric `expected` and `found` fields;
character errors include the offending Unicode character in `found`. String
length errors count UTF-8 bytes.

```js
try {
  Id.fromBytes(new Uint8Array(3));
} catch (error) {
  console.log(error.code);     // 'InvalidLength'
  console.log(error.expected); // 16
  console.log(error.found);    // 3
}
```

Codes are `InvalidData`, `InvalidPrefix`, `InvalidChar`, `InvalidFormat`,
`InvalidLength`, `InvalidType`, `InvalidRange`, and `RandomnessUnavailable`.
`SouvenirError` and `SouvenirErrorCode` are exported TypeScript types, not runtime constructors.
Negative or oversized BigInts are rejected without truncation. An in-range
BigInt must also encode a valid identifier.
