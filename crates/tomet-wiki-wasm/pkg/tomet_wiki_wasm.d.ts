/* tslint:disable */
/* eslint-disable */

/**
 * High-level content management client for GitHub repositories.
 */
export class GitHubCms {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Creates a fresh document at `path`.
     *
     * If the file already exists at HEAD, rejects with `err.code === "AlreadyExists"`.
     */
    createFile(req: any): Promise<any>;
    /**
     * Edits an existing document at `path` based on `baseRev`.
     *
     * Performs 3-way AST merge with remote changes. On concurrent branch movement,
     * retries up to `maxRetries` (default 3).
     */
    editFile(req: any): Promise<any>;
    /**
     * Fetches the target branch's current head commit revision OID.
     */
    getHead(): Promise<string>;
    /**
     * Creates a new `GitHubCms` client.
     *
     * `options` is an object: `{ owner, repo, branch?, token, baseUrl?, fetch? }`.
     * `branch` defaults to `"main"`.
     * `fetch` defaults to `globalThis.fetch`.
     */
    constructor(options: any);
    /**
     * Reads a file at `path` from revision `rev`.
     *
     * Returns `{ content: string, revision: string }` if present, or `null` if absent.
     */
    readFile(path: string, rev: string): Promise<any>;
}

/**
 * The colored ranges of a Tomet text, for `tometHighlighting` of `@telaion/tomet`:
 * an array of `{ from, to, tag }` with UTF-16 offsets.
 */
export function highlightSpans(src: string): any;

/**
 * Finds and lists all `@conflict` markers in `.tmt` document source.
 *
 * Returns an array of [`JsConflictInfo`] objects in top-down document order. `span` is a
 * range of BYTE offsets into the UTF-8 source, and `a`/`b` are the sides as written in it.
 * Throws an `Error` with `.code = "ConflictParseError"` if a marker cannot be read: an
 * unreadable marker is never skipped.
 */
export function listConflicts(source: string): any;

/**
 * Resolves a single `@conflict` marker at `index` to the chosen `side` (`"a"` or `"b"`).
 *
 * Returns the updated document source text with the marker replaced by the chosen
 * side, written as it stood in the marker. Every byte outside the marker is kept; a block
 * marker also takes the line break after it with it.
 *
 * Throws an `Error` with:
 * - `.code = "IndexOutOfBounds"` if `index` does not exist.
 * - `.code = "InvalidSide"` if `side` is not `"a"` or `"b"`.
 * - `.code = "ConflictParseError"` if a marker in the document cannot be read.
 */
export function resolveConflict(source: string, index: number, side: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_githubcms_free: (a: number, b: number) => void;
    readonly githubcms_createFile: (a: number, b: number) => number;
    readonly githubcms_editFile: (a: number, b: number) => number;
    readonly githubcms_getHead: (a: number) => number;
    readonly githubcms_new: (a: number, b: number) => void;
    readonly githubcms_readFile: (a: number, b: number, c: number, d: number, e: number) => number;
    readonly highlightSpans: (a: number, b: number, c: number) => void;
    readonly listConflicts: (a: number, b: number, c: number) => void;
    readonly resolveConflict: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly __wasm_bindgen_func_elem_849: (a: number, b: number, c: number, d: number) => void;
    readonly __wasm_bindgen_func_elem_851: (a: number, b: number, c: number, d: number) => void;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export3: (a: number) => void;
    readonly __wbindgen_export4: (a: number, b: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_export5: (a: number, b: number, c: number) => void;
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
