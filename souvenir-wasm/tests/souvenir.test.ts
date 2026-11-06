import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import {
  compare, dateOf, fromBytes, fromParts, fromUuid, isValid, newId, newMonotonicId, parse,
  parseExpecting, prefixOf, randomOf, timestampOf, toBytes, toUuid, typed,
  type SouvenirError, type SouvenirId,
} from '../dist';

interface Vectors {
  valid: { prefix: string; prefix_rank: number; timestamp_ms: number; random_hex: string;
    bytes_hex: string; text: string; uuid: string }[];
  sorted_text: string[];
  invalid: { kind: 'text' | 'bytes'; input: string; error: string; why: string }[];
  parse_expecting: { expected: string; input: string; error?: string; why: string }[];
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

  test('generates and parses IDs with its prefix', () => {
    const id: SouvenirId<'user'> = UserId.new();
    expect(id.startsWith('user_')).toBe(true);
    expect(UserId.parse(TEXT)).toBe(TEXT as SouvenirId<'user'>);
    expect(UserId.newMonotonic() < UserId.newMonotonic()).toBe(true);
    expect(UserId.prefix).toBe('user');
  });

  test('rejects other prefixes', () => {
    expectCode(() => UserId.parse('users_00t35pg1sg9ech32tymrkdq6v'), 'PrefixMismatch');
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
  test('raw bindings are not exported', () => {
    const manifest = JSON.parse(readFileSync(join(import.meta.dir, '../dist/package.json'), 'utf8'));
    expect(Object.keys(manifest.exports).filter((k) => k.startsWith('./bindings'))).toEqual([]);
  });
});
