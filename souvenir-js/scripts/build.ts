// Compile `src/` twice: ES modules in `dist/esm`, CommonJS in `dist/cjs`.
import { $ } from 'bun';
import { rm } from 'node:fs/promises';
import { join, resolve } from 'node:path';

const root = resolve(import.meta.dir, '..');
const dist = join(root, 'dist');
const shell = $.cwd(root);

await rm(dist, { recursive: true, force: true });
await shell`bun x tsc -p tsconfig.json`;
await shell`bun x tsc -p tsconfig.cjs.json`;
// The package is `"type": "module"`; this marks the CommonJS build as such.
await Bun.write(join(dist, 'cjs/package.json'), JSON.stringify({ type: 'commonjs' }) + '\n');
console.log(`Built ${dist}`);
