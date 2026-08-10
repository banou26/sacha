// The publish flow, from the repo root.
//
// `pkg/` is gitignored and generated, so publishing by hand from inside it can ship a build made
// from code that was never committed, or a version that does not match Cargo.toml. Every step below
// exists because skipping it produces a package that looks fine and is not.
//
//   npm run publish                 full run
//   npm run publish -- --otp=123456 pass anything through to npm publish
//   npm run publish -- --dry-run    everything including npm's own dry run

import { execFileSync, spawnSync } from 'node:child_process'
import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const forwarded = process.argv.slice(2)

const die = (message) => {
  console.error(`\npublish aborted: ${message}`)
  process.exit(1)
}

const run = (command, args, options = {}) => {
  const result = spawnSync(command, args, { cwd: root, stdio: 'inherit', ...options })
  if (result.status !== 0) die(`\`${command} ${args.join(' ')}\` failed`)
}

const capture = (command, args) =>
  execFileSync(command, args, { cwd: root, encoding: 'utf8' }).trim()

const step = (name) => console.log(`\n=== ${name} ===`)

// A publish is a claim that this code is what shipped, and only a committed tree can back it.
step('working tree')
if (capture('git', ['status', '--porcelain'])) die('working tree is dirty, commit or stash first')

const branch = capture('git', ['rev-parse', '--abbrev-ref', 'HEAD'])
if (branch !== 'main') die(`on branch ${branch}, expected main`)

// Asks the remote rather than a remote-tracking ref, which a skipped fetch leaves stale.
const remote = capture('git', ['ls-remote', '--heads', 'origin', 'main']).split(/\s+/)[0]
const local = capture('git', ['rev-parse', 'HEAD'])
if (remote !== local) die(`HEAD ${local.slice(0, 7)} is not pushed (origin/main is ${remote.slice(0, 7)})`)
console.log(`clean, on main, pushed at ${local.slice(0, 7)}`)

step('cargo test')
run('cargo', ['test'])

step('build')
run('npm', ['run', 'build'])

// wasm-pack copies the version out of Cargo.toml; a mismatch means pkg/ is from an older build.
step('version')
const cargoVersion = /^version = "(.+)"$/m.exec(readFileSync(resolve(root, 'Cargo.toml'), 'utf8'))?.[1]
const pkgVersion = JSON.parse(readFileSync(resolve(root, 'pkg/package.json'), 'utf8')).version
if (!cargoVersion) die('could not read version from Cargo.toml')
if (cargoVersion !== pkgVersion) die(`Cargo.toml is ${cargoVersion} but pkg/package.json is ${pkgVersion}`)

// npm rejects a duplicate version with a 403 that reads like an auth failure, which sends you
// looking in the wrong place entirely.
const published = spawnSync('npm', ['view', `sacha@${cargoVersion}`, 'version'], { cwd: root, encoding: 'utf8' })
if (published.status === 0 && published.stdout.trim() === cargoVersion) {
  die(`sacha@${cargoVersion} is already on npm, bump Cargo.toml first (its own commit: chore: 🔖release X.Y.Z)`)
}
console.log(`publishing ${cargoVersion}`)

step('verify the built artifact')
run('node', ['scripts/verify-pkg.mjs'])

step('npm publish')
run('npm', ['publish', ...forwarded], { cwd: resolve(root, 'pkg') })

console.log(`\nsacha@${cargoVersion} published`)
