/* tslint:disable */
/* eslint-disable */
/**
 * Extracts average color from blurhash
 *
 * # Arguments
 * * `blurhash` - Blurhash string
 *
 * # Returns
 * `[r, g, b]`
 */
export function get_blur_hash_average_color(blurhash: string): Int32Array;
/**
 * Decode blurhash to WebP image
 *
 * # Arguments
 * * `blurhash` - Blurhash string
 * * `width` - Width of the image
 * * `height` - Height of the image
 * * `punch` - Punch value
 * # Returns
 * WebP image bytes
 */
export function decode(blurhash: string, width: number, height: number, punch?: number | null): Uint8Array;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly get_blur_hash_average_color: (a: number, b: number) => [number, number];
  readonly decode: (a: number, b: number, c: number, d: number, e: number) => [number, number];
  readonly __wbindgen_export_0: WebAssembly.Table;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
