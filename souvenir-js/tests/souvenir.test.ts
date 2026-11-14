import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { createRequire } from 'node:module';
import {
  compare, dateOf, fromBytes, fromParts, fromUuid, isValid, monotonicGenerator, newId,
  newMonotonicId, parse, parseExpecting, prefixOf, randomGenerator, randomOf, SouvenirError,
  timestampOf, toBytes, toUuid, typed,
  type SouvenirId,
} from '../src/index';

interface Vectors {
  valid: { prefix: string; prefix_rank: number; timestamp_ms: number; random_hex: string;
    bytes_hex: string; text: string; uuid: string }[];
  sorted_text: string[];
  invalid: { kind: 'text' | 'bytes'; input: string; error: string; why: string }[];
  parse_expecting: { expected: string; input: string; error?: string; why: string }[];
  from_parts: { prefix: string; timestamp_ms: number; random_hex: string; text?: string;
    error?: string; why: string }[];
  from_uuid: { prefix: string; uuid: string; text?: string; error?: string; why: string }[];
  monotonic: { name: string; steps: { prefix: string; now_ms: number; draws_hex: string[];
    timestamp_ms?: number; random_hex?: string; text?: string; error?: string; why: string }[] }[];
}

const vectors: Vectors = JSON.parse(
  readFileSync(join(import.meta.dir, '../../tests/fixtures/souvenir_v2_vectors.json'), 'utf8'),
);

const hex = (s: string) => Uint8Array.from(s.match(/../g) ?? [], b => parseInt(b, 16));
const toHex = (b: Uint8Array) => Array.from(b, x => x.toString(16).padStart(2, '0')).join('');
const TEXT = 'user_00t35pg1sg9ech32tymrkdq6v';

function expectCode(action: () => unknown, code: string): void {
  let caught: unknown;
  try {
    action();
  } catch (error) {
    caught = error;
  }
  expect(caught).toBeInstanceOf(SouvenirError);
  expect(caught).toBeInstanceOf(Error);
  const error = caught as SouvenirError;
  expect(error.name).toBe('SouvenirError');
  expect(error.code).toBe(code as SouvenirError['code']);
  expect(error.message.length).toBeGreaterThan(0);
}

describe('conformance vectors', () => {
  test.each(vectors.valid.map(v => [v.text, v] as const))('valid %s', (_, v) => {
    const id = fromParts(v.prefix, v.timestamp_ms, BigInt('0x' + v.random_hex));
    expect(id).toBe(v.text as SouvenirId);
    expect(toHex(toBytes(id))).toBe(v.bytes_hex);
    expect(toUuid(id)).toBe(v.uuid);
    expect(parse(v.text)).toBe(id);
    expect(fromBytes(hex(v.bytes_hex))).toBe(id);
    expect(fromUuid(v.prefix, v.uuid)).toBe(id);
    expect(fromUuid(v.prefix, hex(v.uuid.replaceAll('-', '')))).toBe(id);
    expect(prefixOf(id)).toBe(v.prefix);
    expect(timestampOf(id)).toBe(v.timestamp_ms);
    expect(randomOf(id)).toBe(BigInt('0x' + v.random_hex));
    expect(isValid(v.text)).toBe(true);
  });

  test('sorted_text is plain string order', () => {
    const shuffled = [...vectors.sorted_text].reverse();
    expect(shuffled.sort(compare)).toEqual(vectors.sorted_text);
  });

  test.each(vectors.invalid.map(v => [v.why, v] as const))('invalid: %s', (_, v) => {
    if (v.kind === 'text') {
      expectCode(() => parse(v.input), v.error);
      expect(isValid(v.input)).toBe(false);
    } else {
      expectCode(() => fromBytes(hex(v.input)), v.error);
    }
  });

  test.each(vectors.parse_expecting.map(v => [v.why, v] as const))('parseExpecting: %s', (_, v) => {
    if (v.error) {
      expectCode(() => parseExpecting(v.expected, v.input), v.error);
    } else {
      expect(parseExpecting(v.expected, v.input)).toBe(v.input as SouvenirId<string>);
    }
  });

  test.each(vectors.from_parts.map(v => [v.why, v] as const))('fromParts: %s', (_, v) => {
    const build = () => fromParts(v.prefix, v.timestamp_ms, BigInt('0x' + v.random_hex));
    if (v.error) expectCode(build, v.error);
    else expect(build()).toBe(v.text as SouvenirId);
  });

  test.each(vectors.from_uuid.map(v => [v.why, v] as const))('fromUuid: %s', (_, v) => {
    for (const uuid of [v.uuid, v.uuid.toUpperCase(), hex(v.uuid.replaceAll('-', ''))]) {
      if (v.error) {
        expectCode(() => fromUuid(v.prefix, uuid), v.error);
      } else {
        expect(fromUuid(v.prefix, uuid)).toBe(v.text as SouvenirId);
        expect(toUuid(v.text!)).toBe(v.uuid);
      }
    }
  });

  test.each(vectors.monotonic.map(v => [v.name, v] as const))('monotonic: %s', (_, sequence) => {
    let now = 0;
    const draws: bigint[] = [];
    // `csprng_bits(n)` returns the scripted value: the generator's
    // big-endian read of the bytes it asks for gives it back.
    const random = (bytes: Uint8Array) => {
      let value = draws.shift();
      if (value === undefined) throw new Error('an unscripted draw');
      for (let i = bytes.length - 1; i >= 0; i--) {
        bytes[i] = Number(value & 0xffn);
        value >>= 8n;
      }
    };
    const generate = monotonicGenerator({ now: () => now, random });
    for (const step of sequence.steps) {
      now = step.now_ms;
      draws.push(...step.draws_hex.map(d => BigInt('0x' + d)));
      if (step.error) {
        expectCode(() => generate(step.prefix), step.error);
      } else {
        const id = generate(step.prefix);
        expect(id).toBe(step.text as SouvenirId);
        expect(timestampOf(id)).toBe(step.timestamp_ms!);
        expect(randomOf(id)).toBe(BigInt('0x' + step.random_hex));
      }
      expect(draws).toEqual([]);
    }
  });
});

describe('generation', () => {
  test('newId uses the prefix and the current time', () => {
    const before = Date.now();
    const id = newId('user');
    const after = Date.now();
    expect(prefixOf(id)).toBe('user');
    expect(timestampOf(id)).toBeGreaterThanOrEqual(before);
    expect(timestampOf(id)).toBeLessThanOrEqual(after);
    expect(dateOf(id).getTime()).toBe(timestampOf(id));
    expect(newId('user')).not.toBe(id);
  });

  test('newMonotonicId is strictly increasing', () => {
    let last = newMonotonicId('user');
    for (let i = 0; i < 100_000; i++) {
      const next = newMonotonicId('user');
      if (!(next > last)) throw new Error(`${next} is not greater than ${last}`);
      last = next;
    }
  });

  test('invalid prefixes are rejected', () => {
    for (const prefix of ['', 'User', 'us3r', 'toolongpx', 'a_b']) {
      expectCode(() => newId(prefix), 'InvalidPrefix');
    }
  });
});

describe('generators with their own sources', () => {
  const zeros = (bytes: Uint8Array) => { bytes.fill(0); };

  test('randomGenerator reads the clock and random source it is given', () => {
    const generate = randomGenerator({ now: () => 1_793_531_580_000, random: zeros });
    expect(generate('user')).toBe('user_00t35pg1sg000000000000000' as SouvenirId<'user'>);
    const ones = randomGenerator({ now: () => 0, random: bytes => { bytes.fill(0xff); } });
    expect(randomOf(ones('user'))).toBe((1n << 74n) - 1n);
  });

  test('a fractional clock reading is rounded down', () => {
    const generate = randomGenerator({ now: () => 1234.9, random: zeros });
    expect(timestampOf(generate('user'))).toBe(1234);
  });

  test('a failing random source is RandomSourceFailure, with its cause', () => {
    const cause = new Error('no entropy');
    for (const make of [randomGenerator, monotonicGenerator]) {
      const generate = make({ random: () => { throw cause; } });
      expectCode(() => generate('user'), 'RandomSourceFailure');
      try {
        generate('user');
      } catch (error) {
        expect((error as Error).cause).toBe(cause);
      }
    }
  });

  test('a failing clock is ClockFailure', () => {
    for (const reading of [NaN, -1, Infinity, 2 ** 48, '0' as unknown as number]) {
      expectCode(() => randomGenerator({ now: () => reading })('user'), 'ClockFailure');
      expectCode(() => monotonicGenerator({ now: () => reading })('user'), 'ClockFailure');
    }
  });

  test('monotonic generators are independent of each other', () => {
    const a = monotonicGenerator({ now: () => 5, random: zeros });
    const b = monotonicGenerator({ now: () => 5, random: zeros });
    expect(a('user')).toBe(b('user'));
    expect(compare(a('user'), b('org'))).toBe(1);
  });

  test('IDs of different prefixes sort by prefix, whatever their order of creation', () => {
    const generate = monotonicGenerator();
    const user = generate('user');
    const org = generate('org');
    // IDs of different prefixes have different types, so `<` needs `compare`.
    expect(compare(org, user)).toBe(-1);
    // The later ID's timestamp and random bits are still the greater ones.
    const body = (id: string) => (BigInt(timestampOf(id)) << 74n) | randomOf(id);
    expect(body(org) > body(user)).toBe(true);
  });
});

describe('clock', () => {
  test('a broken Date.now() fails with ClockFailure', () => {
    const realNow = Date.now;
    try {
      for (const broken of [NaN, -1, Infinity]) {
        Date.now = () => broken;
        expectCode(() => newId('user'), 'ClockFailure');
        expectCode(() => newMonotonicId('user'), 'ClockFailure');
      }
    } finally {
      Date.now = realNow;
    }
  });
});

describe('typed', () => {
  const UserId = typed('user');
  const OrgId = typed('org');

  test('IDs carry their prefix in their type', () => {
    const user: SouvenirId<'user'> = UserId.new();
    const takesUser = (id: SouvenirId<'user'>) => id;
    takesUser(user);
    // @ts-expect-error An org ID is not a user ID.
    takesUser(OrgId.new());
    // @ts-expect-error Nor is an unvalidated string.
    takesUser('user_00t35pg1sg9ech32tymrkdq6v');
    // Any ID is still a string.
    const text: string = user;
    expect(text).toBe(user);
    const value: unknown = user;
    if (UserId.is(value)) takesUser(value);
  });

  test('generates and parses IDs with its prefix', () => {
    const id: SouvenirId<'user'> = UserId.new();
    expect(id.startsWith('user_')).toBe(true);
    expect(UserId.parse(TEXT)).toBe(TEXT as SouvenirId<'user'>);
    expect(UserId.newMonotonic() < UserId.newMonotonic()).toBe(true);
    expect(UserId.prefix).toBe('user');
  });

  test('rejects other prefixes', () => {
    expectCode(() => UserId.parse('users_00t35pg1sg9ech32tymrkdq6v'), 'PrefixMismatch');
    expect(() => UserId.parse('org_00t35pg1sg9ech32tymrkdq6v')).toThrow(
      'ID has a different prefix than expected (expected `user`, found `org`)',
    );
    expect(UserId.is('users_00t35pg1sg9ech32tymrkdq6v')).toBe(false);
    expect(UserId.is('org_00t35pg1sg9ech32tymrkdq6v')).toBe(false);
    expect(UserId.is(TEXT)).toBe(true);
    expect(UserId.is(42)).toBe(false);
    expectCode(
      () => UserId.fromBytes(toBytes(parse('org_00t35pg1sg9ech32tymrkdq6v'))),
      'PrefixMismatch',
    );
  });

  test('round-trips bytes, UUIDs and parts', () => {
    const id = UserId.parse(TEXT);
    expect(UserId.fromBytes(toBytes(id))).toBe(id);
    expect(UserId.fromUuid(toUuid(id))).toBe(id);
    expect(UserId.fromParts(timestampOf(id), randomOf(id))).toBe(id);
  });

  test('validates the prefix immediately', () => {
    expectCode(() => typed('User'), 'InvalidPrefix');
    expectCode(() => typed(''), 'InvalidPrefix');
  });

  test('is frozen', () => {
    expect(Object.isFrozen(UserId)).toBe(true);
  });
});

describe('JavaScript input boundaries', () => {
  const nonStrings = [null, undefined, 123, true, {}, [], new String('x'), Symbol('id')];

  test.each(nonStrings.map(value => ({ value })))('non-string %# is InvalidType', ({ value }) => {
    expect(isValid(value)).toBe(false);
    // @ts-expect-error Deliberately exercise invalid JavaScript inputs.
    expectCode(() => parse(value), 'InvalidType');
    // @ts-expect-error Deliberately exercise invalid JavaScript inputs.
    expectCode(() => newId(value), 'InvalidType');
    // @ts-expect-error Deliberately exercise invalid JavaScript inputs.
    expectCode(() => typed(value), 'InvalidType');
    // @ts-expect-error Deliberately exercise invalid JavaScript inputs.
    expectCode(() => parseExpecting('user', value), 'InvalidType');
    // @ts-expect-error Deliberately exercise invalid JavaScript inputs.
    expectCode(() => prefixOf(value), 'InvalidType');
    // @ts-expect-error Deliberately exercise invalid JavaScript inputs.
    expectCode(() => toBytes(value), 'InvalidType');
  });

  test('byte inputs must be Uint8Array', () => {
    const bytes = toBytes(parse(TEXT));
    for (const value of [Array.from(bytes), bytes.buffer, new Uint16Array(20), new DataView(bytes.buffer)]) {
      // @ts-expect-error Deliberately exercise invalid JavaScript inputs.
      expectCode(() => fromBytes(value), 'InvalidType');
    }
    expect(fromBytes(Buffer.from(bytes))).toBe(parse(TEXT));
    expectCode(() => fromBytes(new Uint8Array(19)), 'InvalidLength');
    expectCode(() => fromBytes(new Uint8Array(20)), 'ZeroPrefix');
  });

  test('toBytes returns a fresh copy', () => {
    const id = parse(TEXT);
    toBytes(id).fill(0);
    expect(toHex(toBytes(id))).not.toBe('00'.repeat(20));
  });

  test('fromParts checks its numeric inputs', () => {
    expectCode(() => fromParts('user', -1, 0n), 'TimestampOutOfRange');
    expectCode(() => fromParts('user', 1.5, 0n), 'TimestampOutOfRange');
    expectCode(() => fromParts('user', 2 ** 48, 0n), 'TimestampOutOfRange');
    expectCode(() => fromParts('user', 0, 1n << 74n), 'RandomOutOfRange');
    expectCode(() => fromParts('user', 0, -1n), 'RandomOutOfRange');
    // @ts-expect-error Deliberately exercise invalid JavaScript inputs.
    expectCode(() => fromParts('user', '0', 0n), 'InvalidType');
    // @ts-expect-error Deliberately exercise invalid JavaScript inputs.
    expectCode(() => fromParts('user', 0, 0), 'InvalidType');
    expect(fromParts('user', 2 ** 48 - 1, (1n << 74n) - 1n)).toBe(
      'user_3zzzzzzzzzzzzzzzzzzzzzzzz' as SouvenirId<'user'>,
    );
  });

  test('fromUuid checks its input', () => {
    expectCode(() => fromUuid('user', '01a196d0-0e60-4a97-b3c9-e1b7d04a6c8e'), 'NotUuidV7');
    expectCode(() => fromUuid('user', 'not-a-uuid'), 'InvalidLength');
    expectCode(() => fromUuid('user', '01a196d0x0e60-7a97-b3c9-e1b7d04a6c8e'), 'InvalidCharacter');
    expectCode(() => fromUuid('user', '01a196d00e607a97b3c9e1b7d04a6c8e0000'), 'InvalidCharacter');
    expectCode(() => fromUuid('user', new Uint8Array(15)), 'InvalidLength');
    // @ts-expect-error Deliberately exercise invalid JavaScript inputs.
    expectCode(() => fromUuid('user', 42), 'InvalidType');
  });

  test('IDs are plain strings and JSON-serialize as themselves', () => {
    const id = parse(TEXT);
    expect(typeof id).toBe('string');
    expect(JSON.stringify({ id })).toBe(`{"id":"${TEXT}"}`);
  });
});

describe('package', () => {
  const root = join(import.meta.dir, '..');
  const manifest = JSON.parse(readFileSync(join(root, 'package.json'), 'utf8'));

  test('has the same version as the Rust crates', () => {
    const cargo = readFileSync(join(root, '../Cargo.toml'), 'utf8');
    expect(cargo).toContain(`\nversion = "${manifest.version}"\n`);
  });

  test('has no runtime dependencies', () => {
    expect(manifest.dependencies).toBeUndefined();
  });

  // Run `bun run build` first: these load the built package.
  test('the built ES module and CommonJS module agree with the source', async () => {
    const esm = await import(join(root, manifest.exports['.'].import.default));
    const cjs = createRequire(import.meta.url)(join(root, manifest.exports['.'].require.default));
    for (const built of [esm, cjs]) {
      expect(built.parse(TEXT)).toBe(TEXT);
      expect(built.fromBytes(toBytes(TEXT))).toBe(TEXT);
      expect(built.toUuid(TEXT)).toBe(toUuid(TEXT));
      expect(built.typed('user').is(built.newId('user'))).toBe(true);
      expect(Object.keys(built).sort()).toEqual(Object.keys(await import('../src/index')).sort());
    }
  });
});
