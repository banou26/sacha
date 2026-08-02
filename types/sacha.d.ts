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
  free(): void
}

/** Longest query the given options accept, in characters. */
export function maxQueryLen(options?: SearchOptions | null): number

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
