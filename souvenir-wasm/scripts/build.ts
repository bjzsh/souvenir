import { $ } from 'bun';
import { mkdtemp, rename, rm } from 'node:fs/promises';
import { delimiter, join, resolve } from 'node:path';

interface CargoPackage {
  name: string;
  version: string;
  description: string | null;
  license: string | null;
  repository: string | null;
  dependencies: { name: string; req: string }[];
  metadata: { 'build-tools'?: { 'wasm-bodge'?: string } };
}

interface CargoMetadata {
  workspace_root: string;
  packages: CargoPackage[];
}

const root = resolve(import.meta.dir, '..');
const toolsDir = join(root, '.tools');
const toolsBin = join(toolsDir, 'bin');
const outputDir = join(root, 'dist');
const toolPath = [toolsBin, join(root, 'node_modules/.bin'), process.env.PATH ?? ''].join(delimiter);
const shell = $.cwd(root).env({ ...process.env, PATH: toolPath });

function executable(name: string, path = toolPath): string {
  const location = Bun.which(name, { PATH: path });
  if (!location) throw new Error(`Required build tool not found: ${name}`);
  return location;
}

async function readMetadata(cargo: string): Promise<{ crate: CargoPackage; lockPath: string }> {
  const metadata: CargoMetadata = await shell`${cargo} metadata --locked --no-deps --format-version 1`.json();
  const crate = metadata.packages.find(({ name }) => name === 'souvenir-wasm');
  if (!crate) throw new Error('souvenir-wasm is missing from Cargo metadata');
  return { crate, lockPath: join(metadata.workspace_root, 'Cargo.lock') };
}

async function installTools(cargo: string, crate: CargoPackage): Promise<string> {
  const bodgeVersion = crate.metadata['build-tools']?.['wasm-bodge'];
  const bindgenVersion = crate.dependencies.find(({ name }) => name === 'wasm-bindgen')?.req;
  if (!bodgeVersion || !bindgenVersion) {
    throw new Error('Cargo.toml must specify wasm-bodge build metadata and a wasm-bindgen dependency');
  }

  // Cargo skips matching installed versions; Cargo.toml remains the source of pins.
  await shell`${cargo} install wasm-bodge --version ${bodgeVersion} --locked --root ${toolsDir}`;
  await shell`${cargo} install wasm-bindgen-cli --version ${bindgenVersion} --locked --root ${toolsDir}`;
  return executable('wasm-bodge', toolsBin);
}

async function writeManifest(stage: string, crate: CargoPackage): Promise<void> {
  // wasm-bodge generates the exports, loaders, and declarations.
  const manifest = {
    name: 'souvenir.js',
    version: crate.version,
    description: crate.description,
    license: crate.license,
    repository: crate.repository,
    engines: { node: '>=22.12.0' },
    files: ['README.md', 'LICENSE'],
  };
  await Bun.write(join(stage, 'package.json'), JSON.stringify(manifest, null, 2) + '\n');
}

async function buildPackage(bodge: string, crate: CargoPackage, lockPath: string): Promise<void> {
  const stage = await mkdtemp(join(root, '.build.'));
  try {
    const lockFile = Bun.file(lockPath);
    const originalLock = await lockFile.text();
    await writeManifest(stage, crate);
    await shell`${bodge} build --crate-path ${root} --package-json ${join(stage, 'package.json')} --out-dir ${join(stage, 'lib')} --panic abort`;

    // Upstream has no --locked flag: reject output if dependency resolution changed.
    if (await lockFile.text() !== originalLock) {
      throw new Error('wasm-bodge changed Cargo.lock; review dependency changes before rebuilding.');
    }
    for (const file of ['README.md', 'LICENSE']) {
      await Bun.write(join(stage, file), Bun.file(join(root, file)));
    }
    await rm(outputDir, { recursive: true, force: true });
    await rename(stage, outputDir);
    console.log(`Built souvenir.js@${crate.version} in ${outputDir}`);
  } finally {
    await rm(stage, { recursive: true, force: true });
  }
}

async function main(): Promise<void> {
  const cargo = executable('cargo');
  executable('wasm-opt');
  executable('esbuild');

  const { crate, lockPath } = await readMetadata(cargo);
  const bodge = await installTools(cargo, crate);
  await buildPackage(bodge, crate, lockPath);
}

await main();
