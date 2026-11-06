/**
 * Souvenir IDs for JavaScript: type-safe, sortable, prefixed identifiers
 * like `user_00t35pg1sg9ech32tymrkdq6v`.
 *
 * IDs are plain strings at run time. In TypeScript they are branded with
 * their prefix, so a `SouvenirId<"user">` can't be passed where a
 * `SouvenirId<"org">` is expected.
 */
import * as raw from "#wasm-bodge/bindings";

export type { SouvenirError, SouvenirErrorCode } from "#wasm-bodge/bindings";

declare const brand: unique symbol;

/** A validated Souvenir ID string with prefix `P`. */
export type SouvenirId<P extends string = string> = `${P}_${string}` & {
  readonly [brand]: P;
};

/** Generate a new ID with the given prefix. */
export function newId<const P extends string>(prefix: P): SouvenirId<P> {
  return raw.newId(prefix) as SouvenirId<P>;
}

/**
 * Generate a new ID from the module-wide monotonic generator. Each result is
 * greater than the previous one, even within the same millisecond.
 */
export function newMonotonicId<const P extends string>(prefix: P): SouvenirId<P> {
  return raw.newMonotonicId(prefix) as SouvenirId<P>;
}

/** Validate an ID string, throwing a `SouvenirError` if it is invalid. */
export function parse(text: string): SouvenirId {
  return raw.parse(text) as SouvenirId;
}

/**
 * Validate an ID string with the given prefix. An invalid `prefix` throws
 * `InvalidPrefix`; a valid ID with another prefix throws `PrefixMismatch`.
 */
export function parseExpecting<const P extends string>(prefix: P, text: string): SouvenirId<P> {
  return raw.parseExpecting(prefix, text) as SouvenirId<P>;
}

/** Whether `value` is a valid ID string. Never throws. */
export function isValid(value: unknown): value is SouvenirId {
  return raw.isValid(value);
}

/** Build an ID from its parts: Unix milliseconds and a 74-bit random value. */
export function fromParts<const P extends string>(
  prefix: P,
  timestampMs: number,
  random: bigint,
): SouvenirId<P> {
  return raw.fromParts(prefix, timestampMs, random) as SouvenirId<P>;
}

/** The prefix of an ID. */
export function prefixOf<P extends string>(id: SouvenirId<P> | string): P {
  return raw.prefixOf(id) as P;
}

/** Creation time in Unix milliseconds, like `Date.now()`. */
export const timestampOf: (id: string) => number = raw.timestampMs;

/** Creation time as a `Date`. */
export function dateOf(id: string): Date {
  return new Date(raw.timestampMs(id));
}

/** The 74-bit random field. */
export const randomOf: (id: string) => bigint = raw.randomOf;

/** The 20-byte binary form. */
export const toBytes: (id: string) => Uint8Array = raw.toBytes;

/** Decode the 20-byte binary form. All-zero bytes throw `ZeroPrefix`. */
export function fromBytes(bytes: Uint8Array): SouvenirId {
  return raw.fromBytes(bytes) as SouvenirId;
}

/** The UUIDv7 form (lowercase, hyphenated). The prefix is dropped. */
export const toUuid: (id: string) => string = raw.toUuid;

/** Rebuild an ID from a prefix and a UUIDv7 (text or 16 bytes). */
export function fromUuid<const P extends string>(prefix: P, uuid: string | Uint8Array): SouvenirId<P> {
  return raw.fromUuid(prefix, uuid) as SouvenirId<P>;
}

/**
 * Compare two IDs. Text order is the same as binary order, so this is plain
 * string comparison: IDs sort by prefix, then by creation time.
 */
export function compare(a: string, b: string): -1 | 0 | 1 {
  return a < b ? -1 : a > b ? 1 : 0;
}

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
  raw.validatePrefix(prefix);
  return Object.freeze({
    prefix,
    new: () => raw.newId(prefix) as SouvenirId<P>,
    newMonotonic: () => raw.newMonotonicId(prefix) as SouvenirId<P>,
    parse: (text: string) => raw.parseExpecting(prefix, text) as SouvenirId<P>,
    is: (value: unknown): value is SouvenirId<P> => raw.isValid(value, prefix),
    fromParts: (timestampMs: number, random: bigint) =>
      raw.fromParts(prefix, timestampMs, random) as SouvenirId<P>,
    fromBytes: (bytes: Uint8Array) =>
      raw.parseExpecting(prefix, raw.fromBytes(bytes)) as SouvenirId<P>,
    fromUuid: (uuid: string | Uint8Array) => raw.fromUuid(prefix, uuid) as SouvenirId<P>,
  });
}
