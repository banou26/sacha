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
