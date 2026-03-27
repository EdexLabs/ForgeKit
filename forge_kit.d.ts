/* tslint:disable */
/* eslint-disable */

export class MetadataManagerWrapper {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Add custom functions from a JSON string
     *
     * The JSON should be an array of Function objects.
     * Returns the number of functions added.
     */
    addCustomFunctionsFromJson(json: string): number;
    /**
     * Add a custom source
     */
    addCustomSource(extension: string, functions_url?: string | null, enums_url?: string | null, events_url?: string | null): void;
    /**
     * Add a GitHub source
     */
    addGithubSource(extension: string, repo: string, branch: string): void;
    /**
     * Clear all metadata
     */
    clear(): void;
    /**
     * Get enum count
     */
    enumCount(): number;
    /**
     * Get event count
     */
    eventCount(): number;
    /**
     * Export cache to JSON
     */
    exportCache(): string;
    /**
     * Fetch all metadata (async)
     */
    fetchAll(): Promise<any>;
    /**
     * Get function count
     */
    functionCount(): number;
    /**
     * Get all enums
     */
    getAllEnums(): any;
    /**
     * Get all events
     */
    getAllEvents(): any;
    /**
     * Get all functions
     */
    getAllFunctions(): any;
    /**
     * Get completions for prefix
     */
    getCompletions(prefix: string): any;
    /**
     * Get enum values
     */
    getEnum(name: string): any | undefined;
    /**
     * Get event by name
     */
    getEvent(name: string): string | undefined;
    /**
     * Get function by name (fuzzy / alias-aware)
     */
    getFunction(name: string): string | undefined;
    /**
     * Get function by exact name
     */
    getFunctionExact(name: string): string | undefined;
    /**
     * Get multiple functions by name in one call.
     *
     * Accepts a JS array of strings; returns a JS array where each element is
     * either a Function object or `null` if the name was not found.
     */
    getFunctionMany(names: any): any;
    /**
     * Get the longest registered function name that is a prefix of `text`,
     * together with the matched key.
     *
     * Returns `{ key: string, function: Function } | null`
     */
    getFunctionPrefix(text: string): any;
    /**
     * Get a function together with the key that matched (handles aliases).
     *
     * Returns `{ key: string, function: Function } | null`
     */
    getFunctionWithMatch(name: string): any;
    /**
     * Import cache from JSON
     */
    importCache(json: string): void;
    /**
     * Load from localStorage
     */
    loadFromLocalStorage(key: string): void;
    /**
     * Create a new metadata manager
     */
    constructor();
    /**
     * Remove all custom functions previously added via `addCustomFunctionsFromJson`
     */
    removeCustomFunctions(): void;
    /**
     * Save to localStorage
     */
    saveToLocalStorage(key: string): void;
}

/**
 * Calculate AST statistics
 */
export function calculateStats(source: string): any;

/**
 * Collect all function names using the visitor pattern
 */
export function collectFunctions(source: string): any;

/**
 * Check if source contains JavaScript expressions
 */
export function containsJavaScript(source: string): boolean;

/**
 * Count node types using visitor
 */
export function countNodeTypes(source: string): any;

/**
 * Count total nodes in source
 */
export function countNodes(source: string): number;

/**
 * Extract function names from source code
 */
export function extractFunctionNames(source: string): any;

/**
 * Extract all text nodes from source.
 *
 * Returns an array of `{ text: string, span: { start: number, end: number } }`.
 */
export function extractTextNodes(source: string): any;

/**
 * Flatten the AST into a depth-first linear list of node descriptors.
 *
 * Returns an array of objects, each with a `type` field and relevant fields
 * for that node type.
 */
export function flattenAst(source: string): any;

/**
 * Format AST as human-readable string
 */
export function formatAst(source: string): string;

/**
 * Return the source-code slice for a given byte span.
 */
export function getSourceSlice(source: string, start: number, end: number): string;

export function init(): void;

/**
 * Check whether the character at `byte_idx` in `source` is escaped
 * (i.e. preceded by an odd number of backslashes).
 */
export function isEscaped(source: string, byte_idx: number): boolean;

/**
 * Get the maximum function-nesting depth in source
 */
export function maxNestingDepth(source: string): number;

/**
 * Parse ForgeScript source code (no validation)
 */
export function parse(source: string): any;

/**
 * Parse multiple sources at once
 */
export function parseBatch(sources: any): any;

/**
 * Parse and return an error if there are any parse errors, otherwise return the AST string
 */
export function parseOrError(source: string): any;

/**
 * Parse with strict validation (all validations enabled)
 */
export function parseStrict(source: string, metadata_wrapper: MetadataManagerWrapper): any;

/**
 * Parse with a specific validation config object
 *
 * `config` should be a JS object with boolean fields:
 * `validateArguments`, `validateEnums`, `validateFunctions`, `validateBrackets`
 */
export function parseWithConfig(source: string, config: any): any;

/**
 * Parse with validation (requires metadata)
 */
export function parseWithValidation(source: string, metadata_wrapper: MetadataManagerWrapper, validate_arguments: boolean, validate_enums: boolean, validate_functions: boolean, validate_brackets: boolean): any;

/**
 * Validate multiple sources at once
 */
export function validateBatch(sources: any, metadata_wrapper: MetadataManagerWrapper): any;

/**
 * Validate code and return detailed results
 */
export function validateCode(source: string, metadata_wrapper: MetadataManagerWrapper): any;

/**
 * Return a strict ValidationConfig as a JS object
 *
 * Returns `{ validateArguments: true, validateEnums: true, validateFunctions: true, validateBrackets: true }`
 */
export function validationConfigStrict(): any;

/**
 * Return a syntax-only ValidationConfig as a JS object
 *
 * Returns `{ validateArguments: false, validateEnums: false, validateFunctions: false, validateBrackets: false }`
 */
export function validationConfigSyntaxOnly(): any;

/**
 * Get version information
 */
export function version(): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_metadatamanagerwrapper_free: (a: number, b: number) => void;
    readonly calculateStats: (a: number, b: number) => number;
    readonly collectFunctions: (a: number, b: number) => number;
    readonly containsJavaScript: (a: number, b: number) => number;
    readonly countNodeTypes: (a: number, b: number) => number;
    readonly countNodes: (a: number, b: number) => number;
    readonly extractFunctionNames: (a: number, b: number) => number;
    readonly extractTextNodes: (a: number, b: number) => number;
    readonly flattenAst: (a: number, b: number) => number;
    readonly formatAst: (a: number, b: number, c: number) => void;
    readonly getSourceSlice: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly isEscaped: (a: number, b: number, c: number) => number;
    readonly maxNestingDepth: (a: number, b: number) => number;
    readonly metadatamanagerwrapper_addCustomFunctionsFromJson: (a: number, b: number, c: number, d: number) => void;
    readonly metadatamanagerwrapper_addCustomSource: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly metadatamanagerwrapper_addGithubSource: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly metadatamanagerwrapper_clear: (a: number) => void;
    readonly metadatamanagerwrapper_enumCount: (a: number) => number;
    readonly metadatamanagerwrapper_eventCount: (a: number) => number;
    readonly metadatamanagerwrapper_exportCache: (a: number, b: number) => void;
    readonly metadatamanagerwrapper_fetchAll: (a: number) => number;
    readonly metadatamanagerwrapper_functionCount: (a: number) => number;
    readonly metadatamanagerwrapper_getAllEnums: (a: number) => number;
    readonly metadatamanagerwrapper_getAllEvents: (a: number) => number;
    readonly metadatamanagerwrapper_getAllFunctions: (a: number) => number;
    readonly metadatamanagerwrapper_getCompletions: (a: number, b: number, c: number) => number;
    readonly metadatamanagerwrapper_getEnum: (a: number, b: number, c: number) => number;
    readonly metadatamanagerwrapper_getEvent: (a: number, b: number, c: number, d: number) => void;
    readonly metadatamanagerwrapper_getFunction: (a: number, b: number, c: number, d: number) => void;
    readonly metadatamanagerwrapper_getFunctionExact: (a: number, b: number, c: number, d: number) => void;
    readonly metadatamanagerwrapper_getFunctionMany: (a: number, b: number) => number;
    readonly metadatamanagerwrapper_getFunctionPrefix: (a: number, b: number, c: number) => number;
    readonly metadatamanagerwrapper_getFunctionWithMatch: (a: number, b: number, c: number) => number;
    readonly metadatamanagerwrapper_importCache: (a: number, b: number, c: number, d: number) => void;
    readonly metadatamanagerwrapper_loadFromLocalStorage: (a: number, b: number, c: number, d: number) => void;
    readonly metadatamanagerwrapper_new: () => number;
    readonly metadatamanagerwrapper_removeCustomFunctions: (a: number) => void;
    readonly metadatamanagerwrapper_saveToLocalStorage: (a: number, b: number, c: number, d: number) => void;
    readonly parse: (a: number, b: number) => number;
    readonly parseBatch: (a: number) => number;
    readonly parseOrError: (a: number, b: number) => number;
    readonly parseStrict: (a: number, b: number, c: number) => number;
    readonly parseWithConfig: (a: number, b: number, c: number) => number;
    readonly parseWithValidation: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => number;
    readonly validateBatch: (a: number, b: number) => number;
    readonly validateCode: (a: number, b: number, c: number) => number;
    readonly validationConfigStrict: () => number;
    readonly validationConfigSyntaxOnly: () => number;
    readonly version: () => number;
    readonly init: () => void;
    readonly __wasm_bindgen_func_elem_1042: (a: number, b: number) => void;
    readonly __wasm_bindgen_func_elem_541: (a: number, b: number) => void;
    readonly __wasm_bindgen_func_elem_1092: (a: number, b: number, c: number, d: number) => void;
    readonly __wasm_bindgen_func_elem_1043: (a: number, b: number, c: number) => void;
    readonly __wasm_bindgen_func_elem_542: (a: number, b: number) => void;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export3: (a: number) => void;
    readonly __wbindgen_export4: (a: number, b: number, c: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
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
