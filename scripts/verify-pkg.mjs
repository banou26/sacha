// Smoke-tests the BUILT artifact, not the source.
//
// `cargo test` proves the Rust. It cannot prove that the wasm in `pkg/` was rebuilt from it, and a
// stale `pkg/` is exactly what gets published: the directory is gitignored, so nothing else notices.
// This loads the real `pkg/sacha.js` against the real `pkg/sacha_bg.wasm` and asserts behaviour that
// only the current grammar produces.

import { readFile } from 'node:fs/promises'

import init, { parse } from '../pkg/sacha.js'

await init({ module_or_path: await readFile(new URL('../pkg/sacha_bg.wasm', import.meta.url)) })

const season = (name) => (parse(name).seasonTerms ?? [])[0]

// One case per season shape the parser knows, so a half-applied build cannot pass.
const EXPECTED = [
  ['That Time I Got Reincarnated as a Slime Season 4', 'Season 4'],
  ['Sousou no Frieren 2nd Season', '2nd Season'],
  ['86: Eighty Six Part 2', 'Part 2'],
  ['Dr. STONE SCIENCE FUTURE Cour 3', 'Cour 3'],
  ['幼女战记 第二季', '第二季'],
  ['転生したら剣でした 第2期', '第2期'],
  ['おでかけ子ザメ シーズン2', 'シーズン2'],
  ['전생했더니 슬라임이었던 건에 대하여 4기', '4기'],
]

// A season number is an identity claim, so inventing one is worse than missing one.
const NEGATIVE = ['Death Note', 'Cowboy Bebop', '進撃の巨人 第1話', '鬼滅の刃 第25集', 'Yami Shibai 17']

const failures = []

for (const [name, want] of EXPECTED) {
  const got = season(name)
  if (got !== want) failures.push(`  ${JSON.stringify(name)} -> ${JSON.stringify(got)}, wanted ${JSON.stringify(want)}`)
}

for (const name of NEGATIVE) {
  const got = season(name)
  if (got !== undefined) failures.push(`  ${JSON.stringify(name)} invented a season: ${JSON.stringify(got)}`)
}

// A release name has to survive whole, or the parser is passing the season cases by eating everything else.
const release = parse('[Erai-raws] Mushoku Tensei - Isekai Ittara Honki Dasu Part 2 - 01 ~ 12 [1080p]')
const shape = {
  titles: release.titles?.[0],
  seasonTerms: release.seasonTerms?.[0],
  groups: release.groups?.[0],
  resolutionTerms: release.resolutionTerms?.[0],
}
const wanted = {
  titles: 'Mushoku Tensei - Isekai Ittara Honki Dasu',
  seasonTerms: 'Part 2',
  groups: 'Erai-raws',
  resolutionTerms: '1080p',
}
for (const [field, want] of Object.entries(wanted)) {
  if (shape[field] !== want) failures.push(`  release name ${field}: ${JSON.stringify(shape[field])}, wanted ${JSON.stringify(want)}`)
}

if (failures.length) {
  console.error(`pkg/ failed ${failures.length} check(s):\n${failures.join('\n')}\n\nRebuild with: npm run build`)
  process.exit(1)
}

console.log(`pkg/ verified: ${EXPECTED.length} season shapes, ${NEGATIVE.length} negatives, and one release name intact`)
