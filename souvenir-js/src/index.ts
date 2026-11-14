/**
 * Souvenir IDs for JavaScript: type-safe, sortable, prefixed identifiers
 * like `user_00t35pg1sg9ech32tymrkdq6v`.
 *
 * IDs are plain strings at run time. In TypeScript they are branded with
 * their prefix, so a `SouvenirId<"user">` can't be passed where a
 * `SouvenirId<"org">` is expected.
 *
 * This is an implementation of the Souvenir ID format, version 2 (SPEC.md),
 * checked against the same conformance vectors as the Rust crate.
 */

// The numbers SPEC.md section 2 defines.

const ALPHABET = "0123456789abcdefghjkmnpqrstvwxyz";
const SUFFIX_LEN = 25;
const BYTE_LEN = 20;
const UUID_LEN = 16;
const MAX_PREFIX_LEN = 8;

/** `PREFIX_COUNTS[m]` is the number of strings of length `0..=m` over a-z. */
const PREFIX_COUNTS = [
  1, 27, 703, 18_279, 475_255, 12_356_631, 321_272_407, 8_353_082_583, 217_180_147_159,
] as const;
const PREFIX_RANK_LIMIT = PREFIX_COUNTS[MAX_PREFIX_LEN];

const TIMESTAMP_MAX = 2 ** 48 - 1;
const RANDOM_BITS = 74n;
const RANDOM_MAX = (1n << RANDOM_BITS) - 1n;
const BODY_BITS = 122n;
const BODY_MASK = (1n << BODY_BITS) - 1n;
const RAND_B_MASK = (1n << 62n) - 1n;

const ASCII = /^[\x00-\x7f]*$/;
const PREFIX = /^[a-z]{1,8}$/;
const SUFFIX = /^[0-9a-hjkmnp-tv-z]*$/;
const UUID_TEXT = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

/** Maps a character code to its 5-bit suffix value. */
const DECODE = new Map<number, bigint>(
  Array.from(ALPHABET, (ch, i) => [ch.charCodeAt(0), BigInt(i)]),
);

// Errors

const MESSAGES = {
  InvalidPrefix: "prefix must be 1-8 characters of a-z",
  MissingSeparator: "ID text has no '_' separator",
  InvalidLength: "input has the wrong length",
  InvalidCharacter: "input contains an invalid character",
  NonCanonical: "suffix is not canonical (first character must be 0-3)",
  ZeroPrefix: "prefix field is zero, which is not any prefix",
  ReservedPrefix: "prefix field is in the range reserved for future formats",
  PrefixMismatch: "ID has a different prefix than expected",
  TimestampOutOfRange: "timestamp must be an integer in [0, 2^48)",
  RandomOutOfRange: "random value must be in [0, 2^74)",
  NotUuidV7: "UUID is not version 7 with variant 0b10",
  RandomSourceFailure: "the random number source failed",
  ClockFailure: "the clock is unavailable or out of range",
  InvalidType: "argument has the wrong type",
} as const;

/**
 * A stable error code. All but `InvalidType` are defined by the Souvenir
 * specification and are the same in every implementation.
 */
export type SouvenirErrorCode = keyof typeof MESSAGES;

/** The error every function in this package throws. `code` is stable. */
export class SouvenirError extends Error {
  /** The stable error code. Compare this, not `message`. */
  readonly code: SouvenirErrorCode;

  constructor(code: SouvenirErrorCode, detail?: string, options?: { cause?: unknown }) {
    super(detail === undefined ? MESSAGES[code] : `${MESSAGES[code]} (${detail})`, options);
    this.name = "SouvenirError";
    this.code = code;
  }
}

function fail(code: SouvenirErrorCode, detail?: string): never {
  throw new SouvenirError(code, detail);
}

// Types

declare const brand: unique symbol;

/** A validated Souvenir ID string with prefix `P`. */
export type SouvenirId<P extends string = string> = `${P}_${string}` & {
  readonly [brand]: P;
};

// Prefixes (SPEC.md section 4)

function checkString(value: unknown): string {
  return typeof value === "string" ? value : fail("InvalidType", "expected a string");
}

function checkPrefix(value: unknown): string {
  const prefix = checkString(value);
  return PREFIX.test(prefix) ? prefix : fail("InvalidPrefix");
}

function rank(prefix: string): number {
  let rank = 0;
  for (let i = 0; i < prefix.length; i++) {
    rank += 1 + (prefix.charCodeAt(i) - 97) * PREFIX_COUNTS[MAX_PREFIX_LEN - 1 - i]!;
  }
  return rank;
}

function unrank(rank: number): string {
  if (rank === 0) fail("ZeroPrefix");
  if (rank >= PREFIX_RANK_LIMIT) fail("ReservedPrefix");
  let prefix = "";
  for (let i = 0; rank > 0; i++) {
    rank -= 1;
    const size = PREFIX_COUNTS[MAX_PREFIX_LEN - 1 - i]!;
    prefix += String.fromCharCode(97 + Math.floor(rank / size));
    rank %= size;
  }
  return prefix;
}

// Text (SPEC.md sections 5 and 6)

/** Parse `text` (section 6.3) and return where its separator is. */
function separator(value: unknown): number {
  const text = checkString(value);
  if (!ASCII.test(text)) fail("InvalidCharacter");
  const at = text.lastIndexOf("_");
  if (at < 0) fail("MissingSeparator");
  if (!PREFIX.test(text.slice(0, at))) fail("InvalidPrefix");
  const suffix = text.slice(at + 1);
  if (suffix.length !== SUFFIX_LEN) fail("InvalidLength");
  if (!SUFFIX.test(suffix)) fail("InvalidCharacter");
  if (suffix.charCodeAt(0) > 0x33) fail("NonCanonical");
  return at;
}

/** The prefix and 122-bit body of a valid ID. */
function fields(text: unknown): { prefix: string; body: bigint } {
  const at = separator(text);
  const id = text as string;
  let body = 0n;
  for (let i = at + 1; i < id.length; i++) {
    body = (body << 5n) | DECODE.get(id.charCodeAt(i))!;
  }
  return { prefix: id.slice(0, at), body };
}

/** The text form of a valid prefix and a body. */
function format(prefix: string, body: bigint): string {
  let text = prefix + "_";
  for (let shift = 5n * BigInt(SUFFIX_LEN - 1); shift >= 0n; shift -= 5n) {
    text += ALPHABET[Number((body >> shift) & 31n)];
  }
  return text;
}

function expectPrefix(expected: string, actual: string): void {
  if (expected !== actual) {
    fail("PrefixMismatch", `expected \`${expected}\`, found \`${actual}\``);
  }
}

/** Validate an ID string, throwing a `SouvenirError` if it is invalid. */
export function parse(text: string): SouvenirId {
  separator(text);
  // Parsing is strict, so a valid ID is already in its only text form.
  return text as SouvenirId;
}

/**
 * Validate an ID string with the given prefix. An invalid `prefix` throws
 * `InvalidPrefix` before `text` is looked at; a valid ID with another prefix
 * throws `PrefixMismatch`.
 */
export function parseExpecting<const P extends string>(prefix: P, text: string): SouvenirId<P> {
  checkPrefix(prefix);
  const at = separator(text);
  expectPrefix(prefix, text.slice(0, at));
  return text as SouvenirId<P>;
}

/** Whether `value` is a valid ID string. Never throws. */
export function isValid(value: unknown): value is SouvenirId {
  try {
    separator(value);
    return true;
  } catch {
    return false;
  }
}

/** The prefix of an ID. */
export function prefixOf<P extends string>(id: SouvenirId<P> | string): P {
  const at = separator(id);
  return id.slice(0, at) as P;
}

/** Creation time in Unix milliseconds, like `Date.now()`. */
export function timestampOf(id: string): number {
  return Number(fields(id).body >> RANDOM_BITS);
}

/** Creation time as a `Date`. */
export function dateOf(id: string): Date {
  return new Date(timestampOf(id));
}

/** The 74-bit random field. */
export function randomOf(id: string): bigint {
  return fields(id).body & RANDOM_MAX;
}

/**
 * Compare two IDs. Text order is the same as binary order, so this is plain
 * string comparison: IDs sort by prefix, then by creation time.
 */
export function compare(a: string, b: string): -1 | 0 | 1 {
  return a < b ? -1 : a > b ? 1 : 0;
}

// Parts (SPEC.md section 3.5)

function checkTimestamp(value: unknown): number {
  if (typeof value !== "number") fail("InvalidType", "expected a number");
  if (!Number.isSafeInteger(value) || value < 0 || value > TIMESTAMP_MAX) {
    fail("TimestampOutOfRange");
  }
  return value;
}

function checkRandom(value: unknown): bigint {
  if (typeof value !== "bigint") fail("InvalidType", "expected a bigint");
  if (value < 0n || value > RANDOM_MAX) fail("RandomOutOfRange");
  return value;
}

function build(prefix: string, timestampMs: number, random: bigint): string {
  return format(prefix, (BigInt(timestampMs) << RANDOM_BITS) | random);
}

/** Build an ID from its parts: Unix milliseconds and a 74-bit random value. */
export function fromParts<const P extends string>(
  prefix: P,
  timestampMs: number,
  random: bigint,
): SouvenirId<P> {
  checkPrefix(prefix);
  return build(prefix, checkTimestamp(timestampMs), checkRandom(random)) as SouvenirId<P>;
}

// Bytes (SPEC.md section 3)

// Read through the original getter, so that neither a subclass nor an object
// from another realm (such as a Node.js `Buffer`) is misjudged.
const typedArrayTag = Object.getOwnPropertyDescriptor(
  Object.getPrototypeOf(Uint8Array.prototype),
  Symbol.toStringTag,
)!.get!;

function isUint8Array(value: unknown): value is Uint8Array {
  return ArrayBuffer.isView(value) && typedArrayTag.call(value) === "Uint8Array";
}

function fromBigEndian(bytes: Uint8Array): bigint {
  let value = 0n;
  for (const byte of bytes) value = (value << 8n) | BigInt(byte);
  return value;
}

function toBigEndian(value: bigint, length: number): Uint8Array {
  const bytes = new Uint8Array(length);
  for (let i = length - 1; i >= 0; i--) {
    bytes[i] = Number(value & 0xffn);
    value >>= 8n;
  }
  return bytes;
}

/** The 20-byte binary form. */
export function toBytes(id: string): Uint8Array {
  const { prefix, body } = fields(id);
  return toBigEndian((BigInt(rank(prefix)) << BODY_BITS) | body, BYTE_LEN);
}

/** Decode the 20-byte binary form. All-zero bytes throw `ZeroPrefix`. */
export function fromBytes(bytes: Uint8Array): SouvenirId {
  if (!isUint8Array(bytes)) fail("InvalidType", "expected a Uint8Array");
  if (bytes.length !== BYTE_LEN) fail("InvalidLength");
  const value = fromBigEndian(bytes);
  return format(unrank(Number(value >> BODY_BITS)), value & BODY_MASK) as SouvenirId;
}

// UUIDv7 (SPEC.md section 8)

/** The UUIDv7 form (lowercase, hyphenated). The prefix is dropped. */
export function toUuid(id: string): string {
  const { body } = fields(id);
  const random = body & RANDOM_MAX;
  const uuid =
    ((body >> RANDOM_BITS) << 80n) |
    (0x7n << 76n) |
    ((random >> 62n) << 64n) |
    (0b10n << 62n) |
    (random & RAND_B_MASK);
  const hex = uuid.toString(16).padStart(2 * UUID_LEN, "0");
  return [hex.slice(0, 8), hex.slice(8, 12), hex.slice(12, 16), hex.slice(16, 20), hex.slice(20)]
    .join("-");
}

/** Rebuild an ID from a prefix and a UUIDv7 (hyphenated text or 16 bytes). */
export function fromUuid<const P extends string>(
  prefix: P,
  uuid: string | Uint8Array,
): SouvenirId<P> {
  checkPrefix(prefix);
  let value: bigint;
  if (typeof uuid === "string") {
    if (uuid.length !== 36) fail("InvalidLength");
    if (!UUID_TEXT.test(uuid)) fail("InvalidCharacter");
    value = BigInt("0x" + uuid.replaceAll("-", ""));
  } else if (isUint8Array(uuid)) {
    if (uuid.length !== UUID_LEN) fail("InvalidLength");
    value = fromBigEndian(uuid);
  } else {
    fail("InvalidType", "expected a string or Uint8Array");
  }
  if (((value >> 76n) & 0xfn) !== 0x7n || ((value >> 62n) & 0b11n) !== 0b10n) {
    fail("NotUuidV7");
  }
  const random = (((value >> 64n) & 0xfffn) << 62n) | (value & RAND_B_MASK);
  return build(prefix, Number(value >> 80n), random) as SouvenirId<P>;
}

// Generation (SPEC.md section 7)

/** Where a generator gets each ID's timestamp: Unix milliseconds, like `Date.now`. */
export type Clock = () => number;

/**
 * Where a generator gets its random bits: a function that fills all of
 * `bytes`, like `crypto.getRandomValues`. In production this must be
 * cryptographically secure. If it throws, generation fails with
 * `RandomSourceFailure`.
 */
export type RandomSource = (bytes: Uint8Array) => void;

/** The sources a generator reads. Each defaults to the system's. */
export interface GeneratorOptions {
  /** Defaults to `Date.now`. */
  now?: Clock;
  /** Defaults to `crypto.getRandomValues`. */
  random?: RandomSource;
}

/** A function that generates an ID with the given prefix. */
export type Generator = <const P extends string>(prefix: P) => SouvenirId<P>;

// The Web Crypto global, declared here so the package needs no DOM or Node
// type definitions.
declare const crypto: { getRandomValues(bytes: Uint8Array): unknown } | undefined;

const systemRandom: RandomSource = (bytes) => {
  if (typeof crypto === "undefined") fail("RandomSourceFailure", "no crypto.getRandomValues");
  crypto.getRandomValues(bytes);
};

// Read `Date.now` at each call, so a clock replaced later is the one used.
const systemClock: Clock = () => Date.now();

function readClock(now: Clock): number {
  const ms = now();
  if (typeof ms !== "number" || !(ms >= 0) || ms > TIMESTAMP_MAX) fail("ClockFailure");
  return Math.floor(ms);
}

/** `count` bytes from `random`, as a big-endian integer. */
function draw(random: RandomSource, count: number): bigint {
  const bytes = new Uint8Array(count);
  try {
    random(bytes);
  } catch (cause) {
    if (cause instanceof SouvenirError) throw cause;
    throw new SouvenirError("RandomSourceFailure", undefined, { cause });
  }
  return fromBigEndian(bytes);
}

const random74 = (random: RandomSource) => draw(random, 10) & RANDOM_MAX;
const randomStep = (random: RandomSource) => 1n + draw(random, 4);

/**
 * A generator that makes each ID independently. IDs with the same prefix
 * created in the same millisecond are in random order.
 */
export function randomGenerator(options: GeneratorOptions = {}): Generator {
  const { now = systemClock, random = systemRandom } = options;
  return <const P extends string>(prefix: P) => {
    checkPrefix(prefix);
    return build(prefix, readClock(now), random74(random)) as SouvenirId<P>;
  };
}

/**
 * A generator whose IDs are strictly increasing within each prefix, even
 * when the clock stalls or goes backwards. Its timestamps can run slightly
 * ahead of the clock to keep them so.
 *
 * One generator serves every prefix. IDs with different prefixes still sort
 * by prefix first.
 */
export function monotonicGenerator(options: GeneratorOptions = {}): Generator {
  const { now = systemClock, random = systemRandom } = options;
  let last: { timestampMs: number; random: bigint } | undefined;
  return <const P extends string>(prefix: P) => {
    checkPrefix(prefix);
    let timestampMs = readClock(now);
    let value: bigint;
    if (last === undefined || timestampMs > last.timestampMs) {
      value = random74(random);
    } else {
      // The same millisecond, or the clock went back: step forward.
      timestampMs = last.timestampMs;
      value = last.random + randomStep(random);
      if (value > RANDOM_MAX) {
        // Overflow: borrow the next millisecond.
        if (timestampMs === TIMESTAMP_MAX) fail("TimestampOutOfRange");
        timestampMs += 1;
        value = random74(random);
      }
    }
    // Only now, so that a failure above leaves the state as it was.
    last = { timestampMs, random: value };
    return build(prefix, timestampMs, value) as SouvenirId<P>;
  };
}

/** Generate a new ID with the given prefix. */
export const newId: Generator = randomGenerator();

/**
 * Generate a new ID from the module-wide monotonic generator. Each result is
 * greater than every earlier one with the same prefix, even within the same
 * millisecond.
 */
export const newMonotonicId: Generator = monotonicGenerator();

// Typed

/** Functions for IDs with one fixed prefix. Create with {@link typed}. */
export interface TypedSouvenir<P extends string> {
  /** The prefix. */
  readonly prefix: P;
  /** Generate a new ID. */
  "new"(): SouvenirId<P>;
  /** Generate a new ID from the module-wide monotonic generator. */
  newMonotonic(): SouvenirId<P>;
  /** Validate an ID string with this prefix. */
  parse(text: string): SouvenirId<P>;
  /** Whether `value` is a valid ID with this prefix. Never throws. */
  is(value: unknown): value is SouvenirId<P>;
  /** Build an ID from its parts. */
  fromParts(timestampMs: number, random: bigint): SouvenirId<P>;
  /** Decode the 20-byte binary form, requiring this prefix. */
  fromBytes(bytes: Uint8Array): SouvenirId<P>;
  /** Rebuild an ID from a UUIDv7, attaching this prefix. */
  fromUuid(uuid: string | Uint8Array): SouvenirId<P>;
}

/**
 * Functions for IDs with a fixed prefix. Throws `InvalidPrefix` immediately
 * if `prefix` is not 1-8 characters of a-z.
 *
 * ```ts
 * const UserId = typed("user");
 * type UserId = SouvenirId<"user">;
 * const id: UserId = UserId.new();
 * ```
 */
export function typed<const P extends string>(prefix: P): TypedSouvenir<P> {
  checkPrefix(prefix);
  const start = prefix + "_";
  return Object.freeze({
    prefix,
    new: () => newId(prefix),
    newMonotonic: () => newMonotonicId(prefix),
    parse: (text: string) => parseExpecting(prefix, text),
    is: (value: unknown): value is SouvenirId<P> =>
      typeof value === "string" && value.startsWith(start) && isValid(value),
    fromParts: (timestampMs: number, random: bigint) => fromParts(prefix, timestampMs, random),
    fromBytes: (bytes: Uint8Array) => {
      const id = fromBytes(bytes);
      expectPrefix(prefix, prefixOf(id));
      return id as SouvenirId<P>;
    },
    fromUuid: (uuid: string | Uint8Array) => fromUuid(prefix, uuid),
  });
}
