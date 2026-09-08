export interface ParseResult {
  titles?: string[]
  episodeTerms?: string[]
  containerTerms?: string[]
  videoCodecTerms?: string[]
  audioCodecTerms?: string[]
  audioLanguageTerms?: string[]
  groups?: string[]
  versionTerms?: string[]
  batchTerms?: string[]
  subtitleTerms?: string[]
  typeTerms?: string[]
  subtitleLanguageTerms?: string[]
  audioTerms?: string[]
  videoTerms?: string[]
  seasonTerms?: string[]
  sourceTerms?: string[]
  dates?: string[]
  resolutionTerms?: string[]
}

export function parse(input: string): ParseResult
export function format(input: string): ParseResult

export interface SearchOptions {
  /**
   * Query characters allowed to go unmatched. `null` scores every entry
   * instead of dropping those that do not contain the query in order, which is
   * what you want when ranking rather than filtering. Defaults to 0.
   */
  maxTypos?: number | null
  /** Smallest score worth returning. Defaults to 1. */
  minScore?: number
  /** Cap on returned rows. 0 or omitted means no cap. */
  limit?: number
}

export interface SearchHit {
  /** Position in the list the index was built from. */
  index: number
  /** Best score across every (query, title candidate) pair. */
  score: number
  /** The title candidate that scored best. */
  title: string
  /** Whether that candidate equals the query exactly. */
  exact: boolean
}

/**
 * A parsed list of release names that can be searched repeatedly.
 *
 * Names are parsed on construction and stay on the wasm side, so a scraped
 * list crosses the boundary once however many queries run against it.
 */
export class ReleaseIndex {
  constructor(names: string[])
  /** Number of releases in the index. */
  readonly len: number
  /** The full parse of one release, or `undefined` when out of range. */
  parsed(index: number): ParseResult | undefined
  /**
   * Ranks the index against one or more aliases of the same show, best first.
   * Pass every alias you have: a release named only `進撃の巨人` and one named
   * only `Attack on Titan` are the same show, and no single query finds both.
   */
  search(queries: string[], options?: SearchOptions | null): SearchHit[]
  /**
   * How well each release NAMES the show the aliases describe, one score per release, in the order
   * the index was built from. Near 1 when it does and well under 0.7 when it names another show.
   *
   * Prefer this to `search` for deciding whether a release belongs to a show. A raw alignment score
   * cannot be thresholded: it scales with length, and normalising it from outside is not possible,
   * because the only self score a caller can reach goes through an API that parses one side and not
   * the other. That asymmetry let an unrelated release score above 1. Here both sides are normalised
   * the same way, aligned in both directions, and divided by the shorter side's own self score.
   *
   * Measured over 42,000 labelled decisions from real nyaa rows: no false positive and no false
   * negative at any threshold from 0.72 to 0.90, and 0.78 is the middle of that window.
   *
   * A SIBLING SEASON scores near 1 and is meant to, since it genuinely names the same show.
   */
  titleScores(aliases: string[]): number[]
  /** The same scores with the pair behind each one, keeping only releases at or above `minScore`. */
  titleMatches(aliases: string[], minScore?: number | null): TitleMatch[]
  free(): void
}

/** One release that named the show, with the pair that scored best behind it. */
export interface TitleMatch {
  /** Position in the list the index was built from. */
  index: number
  /** 0 to 1, comparable across releases and across shows. */
  score: number
  /** The alias that scored best. */
  query: string
  /** The form of the release's title that it scored against. */
  title: string
}

/**
 * Parses `names` and ranks them against `queries` in one call.
 *
 * Equivalent to building a `ReleaseIndex` and searching it once, without a
 * handle to keep or free. Prefer it for a list you query once and discard; keep
 * the index when several queries run against one list, since the parse is the
 * whole cost and the index pays it only once.
 */
export function searchNames(
  names: string[],
  queries: string[],
  options?: SearchOptions | null
): SearchHit[]

/** Longest query the given options accept, in characters. */
export function maxQueryLen(options?: SearchOptions | null): number

/**
 * Builds an index over `names` and scores it against `aliases` in one call.
 *
 * Equivalent to `new ReleaseIndex(names).titleScores(aliases)` without a handle to keep or free.
 * Keep the index when several alias sets run against one list, since the parse is the whole cost.
 */
export function titleScores(names: string[], aliases: string[]): number[]

export type InitInput =
  | RequestInfo
  | URL
  | Response
  | BufferSource
  | WebAssembly.Module

export interface InitOutput {
  readonly memory: WebAssembly.Memory
}

export type SyncInitInput = BufferSource | WebAssembly.Module

export function initSync(
  module: { module: SyncInitInput } | SyncInitInput
): InitOutput

export function init(
  module_or_path?: InitInput | Promise<InitInput>
): Promise<InitOutput>

export default init
