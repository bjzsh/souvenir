import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { Id as EsmId } from '../dist/lib/esm/node.js';

const { Id: CommonJsId } = createRequire(import.meta.url)('../dist');
for (const Id of [EsmId, CommonJsId]) {
  const text = 'user_02v58c5a3fy30k560qrtg4';
  const id = Id.parse(text);
  try {
    assert.equal(id.toString(), text);
    assert.equal(id.toBigInt(), 0xcbcee7b695fe097e9d18ec46bc97fe46n);
  } finally {
    id.free();
  }
}
console.log('Node ESM and CommonJS imports passed');
