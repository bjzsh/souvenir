import { runInNewContext } from 'node:vm';
import { afterEach, describe, expect, test } from 'bun:test';
import { Id, type SouvenirError } from '../dist';

type ErrorDetails<T = SouvenirError> = T extends Error ? Omit<T, keyof Error> : never;

// Shared with the Rust crate's encoding example; assertions do not rely solely
// on one conversion undoing another conversion with the same bug.
const text = 'user_02v58c5a3fy30k560qrtg4';
const suffix = '02v58c5a3fy30k560qrtg4';
const bytes = new Uint8Array([
  0xcb, 0xce, 0xe7, 0xb6, 0x95, 0xfe, 0x09, 0x7e,
  0x9d, 0x18, 0xec, 0x46, 0xbc, 0x97, 0xfe, 0x46,
]);
const bigint = 0xcbcee7b695fe097e9d18ec46bc97fe46n;
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
    expect(id.equals(keep(id.withPrefix('team')))).toBe(false);
    expect(id.equals(keep(Id.parse('user_' + '0'.repeat(22))))).toBe(false);
  });

  test('withPrefix returns a new value and preserves the original', () => {
    const id = keep(Id.parse(text));
    const updated = keep(id.withPrefix('team'));
    expect(updated.toString()).toBe(`team_${suffix}`);
    expect(id.toString()).toBe(text);
    expectError(() => id.withPrefix('invalid'), { code: 'InvalidPrefix' });
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

  test('accepts the zero and maximum binary values', () => {
    for (const value of [0n, 1n, (1n << 128n) - 1n]) {
      const id = keep(Id.fromBigInt(value));
      expect(id.toBigInt()).toBe(value);
      expect(keep(Id.parse(id.toString())).equals(id)).toBe(true);
    }
    expect(keep(Id.fromBytes(new Uint8Array(16))).toString()).toBe('a_' + '0'.repeat(22));
    expect(keep(Id.fromBigInt((1n << 128n) - 1n)).toString()).toBe('zzzz_hmtd1hxh33gedanpsmt9qk');
  });

  test('checks the prefix-dependent suffix boundary', () => {
    const id = keep(Id.parse('zgsv_hmtd1hxh33gedanpsmt9qm'));
    expect(id.suffix).toBe('hmtd1hxh33gedanpsmt9qm');
    expectError(() => id.withPrefix('zgsw'), { code: 'InvalidData' });
    expectError(() => Id.parse('zgsw_hmtd1hxh33gedanpsmt9qm'), { code: 'InvalidData' });
    expectError(() => Id.parse('a_' + 'z'.repeat(22)), { code: 'InvalidData' });
    expect(keep(keep(Id.parse('zgsv_hmtd1hxh33gedanpsmt9qk')).withPrefix('zgsw')).toString())
      .toBe('zgsw_hmtd1hxh33gedanpsmt9qk');
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
        () => id.withPrefix(value),
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
