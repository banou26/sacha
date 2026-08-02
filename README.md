# Sacha
*TL;DR A web/improved version of [anitomy](https://github.com/erengy/anitomy)*

Sacha is a media string parser and more, for filenames, torrent names, or any sort of strings that represent a media name/metadata through a string and formats everything neatly for you.

## Examples:

Sacha can parse all sorts of formats:
```
[silly] Cyberpunk Edgerunners (WEB-DL 1080p HEVC E-AC-3) [Dual-Audio]
Cyberpunk.Edgerunners.S01.1080p.NF.WEB-DL.DDP5.1.DV.HDR.H.265.HUN.JPN.ENG-VARYG (DUAL, Multi-sub)
Ni Zhenshi Ge Tiancai - 15 - 1080p WEB H.264 -NanDesuKa (B-Global).mkv
Initial D - Third Stage (High Quality) MKV [1080p] Blu-Ray Rip (Stabilized V2)
[Erai-raws] Youkoso Jitsuryoku Shijou Shugi no Kyoushitsu e S2 - 02 [480p][Multiple Subtitle][5F5B5979].mkv
```


```ts
const result = parse('[Erai-raws] Youkoso Jitsuryoku Shijou Shugi no Kyoushitsu e S2 - 12 [1080p][HEVC][Multiple Subtitle] [ENG][POR-BR][SPA-LA][SPA][ARA][FRE][GER][ITA]')
expect(result, {
  group: 'Erai-raws',
  title: 'Youkoso Jitsuryoku Shijou Shugi no Kyoushitsu e',
  season: 2,
  episode: 12,
  resolution: '1080p',
  videoCodec: 'HEVC',
  subtitleTerms: ['Multiple Subtitle'],
  subtitleLanguages: ['ENG', 'POR-BR', 'SPA-LA', 'SPA', 'ARA', 'FRE', 'GER', 'ITA']
})

const formattedResults = format(result)
expect(formattedResults, {
  /** Release group name */
  group: 'Erai-raws',
  /** Title of the media */
  title: 'Youkoso Jitsuryoku Shijou Shugi no Kyoushitsu e',
  /** Season number */
  season: 2,
  /** Episode number */
  episode: 12,
  /** Common resolution, can be undefined as inferring doesn't make sense */
  resolution: Resolution.FHD, // 1080
  /** Common video codec, can be undefined */
  videoCodec: VideoCodec.H265, // 'H265'
  subtitleTerms: ['Multiple Subtitle'],
  subtitleLanguages: [
    LanguageTag.EN // 'en' ISO 639-2 Code for English
    LanguageTag.PT // 'pt' Code for Portuguese
    LanguageTag.ES // 'es' Code for Spanish
    LanguageTag.AR // 'ar' Code for Arabic
    LanguageTag.FR // 'fr' Code for French
    LanguageTag.DE // 'de' Code for German
    LanguageTag.IT // 'it' Code for Italian
  ]
})

const inferredResults = infer(result, { type: 'ANIME' })
expect(inferredResults, {
  group: 'Erai-raws',
  title: 'Youkoso Jitsuryoku Shijou Shugi no Kyoushitsu e',
  // ...
  /** Listed languages or inferred languages */
  audioLanguages: [
    LanguageTag.JA // 'ja' ISO 639-2 Code for Japanese
  ]
})
```

```ts
const result = parse('[silly] Cyberpunk Edgerunners (WEB-DL 1080p HEVC E-AC-3) [Dual-Audio]')
expect(result, {
  group: 'silly',
  title: 'Cyberpunk Edgerunners',
  type: 'WEB-DL',
  resolution: '1080p',
  videoCodec: 'HEVC',
  audioCodec: 'E-AC-3',
  audioTerms: ['Dual-Audio']
})

const formattedResults = format(result)
expect(formattedResults, {
  group: 'silly',
  title: 'Cyberpunk Edgerunners',
  /** Type of release, Web, BlueRay, ect... */
  type: ReleaseType.WEB,
  resolution: Resolution.FHD,
  videoCodec: VideoCodec.H265, // 'H265'
  /** Common audio codec, can be undefined */
  audioCodec: AudioCodec.EAC3, // 'EAC3'
  /** Various audio terms that can help inferring languages, quality, ect... */
  audioTerms: ['Dual-Audio'],
})


const inferredResults = infer(result, { type: 'ANIME' })
expect(inferredResults, {
  group: 'silly',
  title: 'Cyberpunk Edgerunners',
  season: 1,
  /** Episode range describing how many episodes there is, if unknown infer Infinity */
  episodes: [1, Infinity],
  /** If a batch release, meaning if there is multiple files included in that (torrent/archive) file */
  batch: true,
  type: ReleaseType.WEB,
  resolution: Resolution.FHD,
  videoCodec: VideoCodec.H265,
  audioCodec: AudioCodec.EAC3,
  audioTerms: ['Dual-Audio'],
  audioLanguages: [
    LanguageTag.JA
    LanguageTag.EN
  ]
})
```

```ts
const result = parse('[MTBB] Made in Abyss S2 - The Golden City of the Scorching Sun - 04')
expect(result, {
  group: 'MTBB',
  title: 'Made in Abyss - The Golden City of the Scorching Sun',
  season: 'S2',
  episode: '04'
})

const formattedResults = format(result)
expect(formattedResults, {
  group: 'MTBB',
  title: 'Made in Abyss - The Golden City of the Scorching Sun',
  season: 2,
  episode: 4
})


const inferredResults = infer(result, { type: 'ANIME' })
expect(inferredResults, {
  group: 'MTBB',
  title: 'Made in Abyss - The Golden City of the Scorching Sun',
  season: 2,
  episode: 4,
  audioLanguages: [LanguageTag.JA]
})
```

## Matching

Sacha bundles [frizbee](https://github.com/saghen/frizbee), a SIMD Smith-Waterman
matcher, so a scraped list of release names can be parsed and ranked against a
title without leaving wasm.

Parsing first is what makes the ranking work. A release name is mostly noise,
and the group tag and quality flags compete with the title for alignment.
Matching the parsed title candidates instead is both faster and more accurate.

```ts
import init, { ReleaseIndex } from 'sacha'

await init()

// parsed once, on construction; the list stays on the wasm side
const index = new ReleaseIndex(scrapedNames)

// pass every alias of the show, not just one
const hits = index.search(
  ['Shingeki no Kyojin', 'Attack on Titan', '進撃の巨人'],
  { limit: 20 }
)
// [{ index: 12, score: 316, title: 'Shingeki no Kyojin', exact: true }, ...]

const release = scrapedNames[hits[0].index]
const metadata = index.parsed(hits[0].index)   // the full ParseResult
```

Passing several aliases is the point rather than a convenience. A release named
only `進撃の巨人` and one named only `Attack on Titan` are the same show, and no
single query reaches both. Each release is scored by its best (alias, title
candidate) pair.

For a list you query once and discard, `searchNames` does the same work without
a handle to keep or free:

```ts
import { searchNames } from 'sacha'

const hits = searchNames(scrapedNames, ['Shingeki no Kyojin', 'Attack on Titan'], { limit: 20 })
```

The two cost the same. Keep a `ReleaseIndex` only when several queries run
against one list: parsing is the whole cost and the index pays it once, so a
repeat search is about 0.03 ms where the first is a few milliseconds.

### Options

| option | default | meaning |
| --- | --- | --- |
| `maxTypos` | `0` | Query characters allowed to go unmatched. `null` scores every entry instead of dropping those that do not contain the query in order. |
| `minScore` | `1` | Smallest score worth returning. |
| `limit` | `0` | Cap on returned rows; `0` means no cap. |

`maxTypos: 0` filters and is roughly six times faster; `maxTypos: null` ranks
the whole list and never silently drops a candidate. Use `null` when you want a
score for everything and `0` when you want the releases that plainly match.

Scores are raw Smith-Waterman totals, not normalized. They rank candidates
against one query well, but are not comparable across queries of different
lengths: a longer title scores higher simply for being longer. To compare across
queries, divide by the query's score against itself.

Queries are capped at `maxQueryLen()` characters (3639 under the default
scoring) because frizbee aborts beyond that, which in wasm takes the whole
module down. Over-long queries throw instead.

### Accuracy

Against 1175 real nyaa.si names, 50 of them Attack on Titan releases and the
rest unrelated, searching the four aliases above returns 50 of 50 with no false
positives.

### Cost

Parsing is the whole cost; building the index adds nothing measurable, and
matching an already-parsed list is a rounding error. Measured in wasm under
node, one query over a freshly parsed list:

| names | parse and search | search again on the same index |
| --- | --- | --- |
| 50 | 1.5 ms | 0.02 ms |
| 100 | 3.0 ms | 0.03 ms |
| 1175 | 33 ms | 2.1 ms |

Cost is linear in the number of names, so a list of a few dozen is a couple of
milliseconds.

The repeat-search column assumes the default `maxTypos: 0`. Setting it to `null`
scores every entry rather than prefiltering, which costs tens of times more per
search: on the 1175 name list, one query goes from about 0.03 ms to about 0.9 ms.
Still small beside the parse, but not free, so prefer the default when a query
plainly appears in the names you are matching.

### Todos:
- make a system that takes all terms, sort them by length, apply them, and re-categorize them back to prevent issues with small terms overriding longer ones
- `format()` currently returns the same shape as `parse()`, and `infer()` does not exist yet; the examples above describe the intended API rather than the current one
