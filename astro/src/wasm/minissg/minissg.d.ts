/* tslint:disable */
/* eslint-disable */
export function save_data(db_name: string, store_name: string, key: string, value: Uint8Array): Promise<void>;
export function load_data(db_name: string, store_name: string, key: string): Promise<string>;
export function main(): void;
export function render(current_file_id: number, context: any): string;
export function __wbgtest_cov_dump(): Uint8Array | undefined;
/**
 * Handler for `console.log` invocations.
 *
 * If a test is currently running it takes the `args` array and stringifies
 * it and appends it to the current output of the test. Otherwise it passes
 * the arguments to the original `console.log` function, psased as
 * `original`.
 */
export function __wbgtest_console_log(args: Array<any>): void;
/**
 * Handler for `console.debug` invocations. See above.
 */
export function __wbgtest_console_debug(args: Array<any>): void;
/**
 * Handler for `console.info` invocations. See above.
 */
export function __wbgtest_console_info(args: Array<any>): void;
/**
 * Handler for `console.warn` invocations. See above.
 */
export function __wbgtest_console_warn(args: Array<any>): void;
/**
 * Handler for `console.error` invocations. See above.
 */
export function __wbgtest_console_error(args: Array<any>): void;
/**
 * The Actor struct manages the application state and handles messages
 */
export class Actor {
  free(): void;
  constructor();
  /**
   * Process a message and return a response
   */
  process_message(message_json: string): string;
}
export class EventEmitter {
  free(): void;
  constructor();
  on(event_name: string, callback: Function): Function;
  off(event_name: string, callback: Function): void;
  emit(event_name: string, args: any): void;
}
/**
 * Runtime test harness support instantiated in JS.
 *
 * The node.js entry script instantiates a `Context` here which is used to
 * drive test execution.
 */
export class WasmBindgenTestContext {
  free(): void;
  /**
   * Creates a new context ready to run tests.
   *
   * A `Context` is the main structure through which test execution is
   * coordinated, and this will collect output and results for all executed
   * tests.
   */
  constructor();
  /**
   * Inform this context about runtime arguments passed to the test
   * harness.
   */
  args(args: any[]): void;
  /**
   * Executes a list of tests, returning a promise representing their
   * eventual completion.
   *
   * This is the main entry point for executing tests. All the tests passed
   * in are the JS `Function` object that was plucked off the
   * `WebAssembly.Instance` exports list.
   *
   * The promise returned resolves to either `true` if all tests passed or
   * `false` if at least one test failed.
   */
  run(tests: any[]): Promise<any>;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_actor_free: (a: number, b: number) => void;
  readonly actor_new: () => number;
  readonly actor_process_message: (a: number, b: number, c: number) => [number, number, number, number];
  readonly save_data: (a: number, b: number, c: number, d: number, e: number, f: number, g: any) => any;
  readonly load_data: (a: number, b: number, c: number, d: number, e: number, f: number) => any;
  readonly main: () => void;
  readonly render: (a: number, b: any) => [number, number, number, number];
  readonly __wbg_eventemitter_free: (a: number, b: number) => void;
  readonly eventemitter_new: () => number;
  readonly eventemitter_on: (a: number, b: number, c: number, d: any) => any;
  readonly eventemitter_off: (a: number, b: number, c: number, d: any) => void;
  readonly eventemitter_emit: (a: number, b: number, c: number, d: any) => void;
  readonly __wbg_wasmbindgentestcontext_free: (a: number, b: number) => void;
  readonly wasmbindgentestcontext_new: () => number;
  readonly wasmbindgentestcontext_args: (a: number, b: number, c: number) => void;
  readonly wasmbindgentestcontext_run: (a: number, b: number, c: number) => any;
  readonly __wbgtest_console_log: (a: any) => void;
  readonly __wbgtest_console_debug: (a: any) => void;
  readonly __wbgtest_console_info: (a: any) => void;
  readonly __wbgtest_console_warn: (a: any) => void;
  readonly __wbgtest_console_error: (a: any) => void;
  readonly __wbgtest_cov_dump: () => [number, number];
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_export_1: WebAssembly.Table;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_export_6: WebAssembly.Table;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly closure1028_externref_shim: (a: number, b: number, c: any) => void;
  readonly closure1048_externref_shim: (a: number, b: number, c: any, d: number, e: any) => void;
  readonly closure1052_externref_shim: (a: number, b: number, c: any, d: any) => void;
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
