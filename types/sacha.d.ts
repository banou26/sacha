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
  /** Query characters allowed to go unmatched. `null` scores every entry instead of dropping those that do not contain the query in order. Defaults to 0. */
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

export class ReleaseIndex {
  constructor(names: string[])
  readonly len: number
  parsed(index: number): ParseResult | undefined
  search(queries: string[], options?: SearchOptions | null): SearchHit[]
  free(): void
}

export function searchNames(
  names: string[],
  queries: string[],
  options?: SearchOptions | null
): SearchHit[]

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
