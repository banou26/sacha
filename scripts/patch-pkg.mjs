import { readFileSync, writeFileSync, copyFileSync } from 'node:fs'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const root = resolve(__dirname, '..')
const pkgDir = resolve(root, 'pkg')

// --- Patch pkg/package.json ---
const pkgJsonPath = resolve(pkgDir, 'package.json')
const pkg = JSON.parse(readFileSync(pkgJsonPath, 'utf-8'))

Object.assign(pkg, {
  name: 'sacha',
  version: pkg.version,
  description: 'A media string parser for filenames, torrent names, and any string representing media metadata',
  type: 'module',
  main: 'sacha.js',
  module: 'sacha.js',
  types: 'sacha.d.ts',
  exports: {
    '.': {
      types: './sacha.d.ts',
      import: './sacha.js'
    },
    './sacha_bg.wasm': './sacha_bg.wasm'
  },
  files: [
    'sacha.js',
    'sacha.d.ts',
    'sacha_bg.wasm',
    'sacha_bg.wasm.d.ts'
  ],
  sideEffects: false,
  repository: {
    type: 'git',
    url: 'git+https://github.com/Banou26/sacha.git'
  },
  keywords: [
    'parser',
    'media',
    'anime',
    'torrent',
    'filename',
    'wasm'
  ],
  author: 'Banou26',
  license: 'MIT'
})

writeFileSync(pkgJsonPath, JSON.stringify(pkg, null, 2) + '\n')
console.log('Patched pkg/package.json')

// --- Copy hand-written types into pkg/ ---
const srcTypes = resolve(root, 'types', 'sacha.d.ts')
const destTypes = resolve(pkgDir, 'sacha.d.ts')
copyFileSync(srcTypes, destTypes)
console.log('Copied types/sacha.d.ts -> pkg/sacha.d.ts')

// --- Patch pkg/sacha.js to add named `init` export ---
const sachaJsPath = resolve(pkgDir, 'sacha.js')
let sachaJs = readFileSync(sachaJsPath, 'utf-8')
sachaJs = sachaJs.replace(
  'export { initSync, __wbg_init as default };',
  'export { __wbg_init as init, initSync, __wbg_init as default };'
)
writeFileSync(sachaJsPath, sachaJs)
console.log('Patched pkg/sacha.js (added named init export)')
