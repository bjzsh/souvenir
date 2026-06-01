import { runInNewContext } from 'node:vm';
import { afterEach, describe, expect, test } from 'bun:test';
import { Id, type SouvenirError } from '../dist';

type ErrorDetails<T = SouvenirError> = T extends Error ? Omit<T, keyof Error> : never;

// Shared with the Rust crate's encoding example; assertions do not rely solely
// on one conversion undoing another conversion with the same bug.
const text = 'user_02v58c5a3fy30k560qrtg4';
const suffix = '02v58c5a3fy30k560qrtg4';
const bytes = new Uint8Array([
  0xac, 0xcb, 0x20, 0x2d, 0x95, 0x0c, 0x2a, 0x86,
  0xff, 0x0c, 0x13, 0x29, 0x81, 0x7c, 0x6a, 0x04,
]);
const bigint = 0xaccb202d950c2a86ff0c1329817c6a04n;
const allocated: Id[] = [];
const keep = (id: Id): Id => (allocated.push(id), id);
afterEach(() => {
  for (const id of allocated.splice(0)) id.free();
});

function expectError(action: () => unknown, details: ErrorDetails): void {
  let caught: unknown;
  try {
    const result = action();
    if (result instanceof Id) keep(result);
  } catch (error) { caught = error; }
  expect(caught).toBeInstanceOf(Error);
  if (!(caught instanceof Error)) throw new Error('Expected an Error to be thrown');
  expect(caught.name).toBe('SouvenirError');
  expect(caught.message.length).toBeGreaterThan(0);
  expect(caught).toMatchObject(details);
  // Runtime fields should match the discriminated union exactly.
  for (const field of ['expected', 'found']) {
    expect(Object.hasOwn(caught, field)).toBe(Object.hasOwn(details, field));
  }
}

describe('Id values', () => {
  test('parses a known identifier and exposes read-only properties', () => {
    const id = keep(Id.parse(text));
    expect(id.toString()).toBe(text);
    expect(String(id)).toBe(text);
    expect(id.prefix).toBe('user');
    expect(id.suffix).toBe(suffix);
    expect(() => {
      // @ts-expect-error Deliberately test assignment to a read-only property.
      id.prefix = 'team';
    }).toThrow(TypeError);
    expect(() => {
      // @ts-expect-error Deliberately test assignment to a read-only property.
      id.suffix = '0'.repeat(22);
    }).toThrow(TypeError);
    expect(id.toString()).toBe(text);
  });

  test('compares values rather than object identities', () => {
    const id = keep(Id.parse(text));
    const copy = keep(Id.parse(text));
    expect(id).not.toBe(copy);
    expect(id.equals(copy)).toBe(true);
    expect(copy.equals(id)).toBe(true);
    expect(id.equals(keep(id.cast('team')))).toBe(false);
    expect(id.equals(keep(Id.parse('user_' + '0'.repeat(22))))).toBe(false);
  });

  test('cast returns a new value and preserves the original', () => {
    const id = keep(Id.parse(text));
    const cast = keep(id.cast('team'));
    expect(cast.toString()).toBe(`team_${suffix}`);
    expect(id.toString()).toBe(text);
    expectError(() => id.cast('invalid'), { code: 'InvalidPrefix' });
    expect(id.toString()).toBe(text);
  });

  test('serializes directly, nested in objects, and in arrays', () => {
    const id = keep(Id.parse(text));
    expect(id.toJSON()).toBe(text);
    expect(JSON.stringify(id)).toBe(JSON.stringify(text));
    expect(JSON.stringify({ id, ids: [id] })).toBe(JSON.stringify({ id: text, ids: [text] }));
  });

  test.each(['a', 'user', 'zzzz'])('generates a valid random ID for %s without initialization', prefix => {
    const id = keep(Id.random(prefix));
    expect(id.prefix).toBe(prefix);
    expect(Id.test(id.toString())).toBe(true);
    expect(keep(Id.parse(id.toString())).equals(id)).toBe(true);
  });
});

describe('conversions', () => {
  test('converts the known big-endian representation in both directions', () => {
    const id = keep(Id.parse(text));
    expect(id.toBytes()).toEqual(bytes);
    expect(id.toBigInt()).toBe(bigint);
    for (const value of [new Id(bytes), Id.fromBytes(bytes), Id.fromBigInt(bigint)]) {
      expect(keep(value).toString()).toBe(text);
    }
  });

  test('input and output byte arrays do not alias the identifier', () => {
    const input = bytes.slice();
    const id = keep(Id.fromBytes(input));
    input.fill(0);
    const output = id.toBytes();
    output.fill(0);
    expect(id.toBytes()).toEqual(bytes);
    expect(id.toString()).toBe(text);
  });

  test.each([0, 15, 17])('rejects %i bytes with precise length details', length => {
    for (const make of [(value: Uint8Array) => new Id(value), Id.fromBytes]) {
      expectError(() => make(new Uint8Array(length)), { code: 'InvalidLength', expected: 16, found: length });
    }
  });

  test.each([-1n, 1n << 128n, (1n << 128n) + bigint])('rejects out-of-range bigint %s without truncation', value => {
    expectError(() => Id.fromBigInt(value), { code: 'InvalidRange' });
  });

  test.each([1, '1', null, undefined, {}])('rejects non-bigint input %j', value => {
    expectError(() => {
      // @ts-expect-error Deliberately exercise invalid JavaScript inputs.
      return Id.fromBigInt(value);
    }, { code: 'InvalidType' });
  });

  test('validates encoded prefixes even when the numeric value is in range', () => {
    expectError(() => Id.fromBigInt(0n), { code: 'InvalidData' });
    expectError(() => Id.fromBigInt((1n << 128n) - 1n), { code: 'InvalidData' });
    expectError(() => Id.fromBytes(new Uint8Array(16)), { code: 'InvalidData' });
  });
});

describe('parsing and errors', () => {
  test('tryParse returns a value for valid input', () => {
    const id = Id.tryParse(text);
    if (id === undefined) throw new Error('Expected a valid identifier');
    expect(keep(id).toString()).toBe(text);
    expect(Id.test(text)).toBe(true);
  });

  test.each([
    ['invalid', { code: 'InvalidFormat' }],
    [`TOOLONG_${suffix}`, { code: 'InvalidPrefix' }],
    ['user_abc', { code: 'InvalidLength', expected: 22, found: 3 }],
    ['user_' + '!'.repeat(22), { code: 'InvalidChar', found: '!' }],
  ] satisfies [string, ErrorDetails][])('reports structured errors for %s', (value, details) => {
    expectError(() => Id.parse(value), details);
    expect(Id.tryParse(value)).toBeUndefined();
    expect(Id.test(value)).toBe(false);
  });

  test('random rejects an invalid prefix with a validation error', () => {
    expectError(() => Id.random('invalid'), { code: 'InvalidPrefix' });
  });
});

describe('JavaScript input boundaries', () => {
  test.each([null, undefined, 123, true, {}, [], new String(text), Symbol('id')].map(value => ({ value })))(
    'handles non-string input %# without entering string conversion', ({ value }) => {
      expect(Id.test(value)).toBe(false);
      expect(Id.tryParse(value)).toBeUndefined();
      const id = keep(Id.parse(text));
      for (const action of [
        // @ts-expect-error Deliberately exercise invalid JavaScript inputs.
        () => Id.parse(value),
        // @ts-expect-error Deliberately exercise invalid JavaScript inputs.
        () => Id.random(value),
        // @ts-expect-error Deliberately exercise invalid JavaScript inputs.
        () => id.cast(value),
      ]) expectError(action, { code: 'InvalidType' });
      expect(id.toString()).toBe(text);
    },
  );

  test.each([
    null, undefined, 123, Array.from(bytes), bytes.buffer,
    new DataView(bytes.buffer), new Uint16Array(bytes), new Int8Array(bytes),
    new Uint8ClampedArray(bytes), { length: 16, [Symbol.toStringTag]: 'Uint8Array' },
  ].map(value => ({ value })))('rejects non-Uint8Array byte input %#', ({ value }) => {
    for (const make of [(input: Uint8Array) => new Id(input), Id.fromBytes]) {
      // @ts-expect-error Deliberately exercise invalid JavaScript inputs.
      expectError(() => make(value), { code: 'InvalidType' });
    }
  });

  test('accepts Buffer, subarrays, and cross-realm Uint8Arrays', () => {
    const padded = new Uint8Array(20);
    padded.set(bytes, 2);
    const foreign = runInNewContext('new Uint8Array(bytes)', { bytes: Array.from(bytes) });
    for (const value of [Buffer.from(bytes), padded.subarray(2, 18), foreign]) {
      expect(keep(Id.fromBytes(value)).toString()).toBe(text);
    }
  });

  test.each(['é', '€', '🦀'])('reports the actual invalid Unicode character %s', ch => {
    const value = 'user_0' + ch + '0'.repeat(21 - new TextEncoder().encode(ch).length);
    expectError(() => Id.parse(value), { code: 'InvalidChar', found: ch });
    expect(Id.tryParse(value)).toBeUndefined();
    expect(Id.test(value)).toBe(false);
  });

  test('reports suffix lengths in UTF-8 bytes', () => {
    expectError(() => Id.parse('user_é'), { code: 'InvalidLength', expected: 22, found: 2 });
  });
});
