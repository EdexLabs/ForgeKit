/* tslint:disable */
/* eslint-disable */

export class MetadataManagerWrapper {
    free(): void;
    [Symbol.dispose](): void;
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
     * Get function by name
     */
    getFunction(name: string): string | undefined;
    /**
     * Get function by exact name
     */
    getFunctionExact(name: string): string | undefined;
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
     * Save to localStorage
     */
    saveToLocalStorage(key: string): void;
}

/**
 * Calculate AST statistics
 */
export function calculateStats(source: string): any;

/**
 * Collect all function names using visitor
 */
export function collectFunctions(source: string): any;

/**
 * Check if source contains JavaScript
 */
export function containsJavaScript(source: string): boolean;

/**
 * Count node types using visitor
 */
export function countNodeTypes(source: string): any;

/**
 * Count nodes in source
 */
export function countNodes(source: string): number;

/**
 * Extract function names from source code
 */
export function extractFunctionNames(source: string): any;

/**
 * Format AST as string
 */
export function formatAst(source: string): string;

export function init(): void;

/**
 * Get max nesting depth
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
 * Parse with strict validation (all validations enabled)
 */
export function parseStrict(source: string, metadata_wrapper: MetadataManagerWrapper): any;

/**
 * Parse with validation (requires metadata)
 */
export function parseWithValidation(source: string, metadata_wrapper: MetadataManagerWrapper, validate_arguments: boolean, validate_enums: boolean, validate_functions: boolean, validate_brackets: boolean, validate_escapes: boolean): any;

/**
 * Validate multiple sources at once
 */
export function validateBatch(sources: any, metadata_wrapper: MetadataManagerWrapper): any;

/**
 * Validate code and return detailed results
 */
export function validateCode(source: string, metadata_wrapper: MetadataManagerWrapper): any;

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
    readonly formatAst: (a: number, b: number, c: number) => void;
    readonly maxNestingDepth: (a: number, b: number) => number;
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
    readonly metadatamanagerwrapper_importCache: (a: number, b: number, c: number, d: number) => void;
    readonly metadatamanagerwrapper_loadFromLocalStorage: (a: number, b: number, c: number, d: number) => void;
    readonly metadatamanagerwrapper_new: () => number;
    readonly metadatamanagerwrapper_saveToLocalStorage: (a: number, b: number, c: number, d: number) => void;
    readonly parse: (a: number, b: number) => number;
    readonly parseBatch: (a: number) => number;
    readonly parseStrict: (a: number, b: number, c: number) => number;
    readonly parseWithValidation: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => number;
    readonly validateBatch: (a: number, b: number) => number;
    readonly validateCode: (a: number, b: number, c: number) => number;
    readonly version: () => number;
    readonly init: () => void;
    readonly __wasm_bindgen_func_elem_927: (a: number, b: number) => void;
    readonly __wasm_bindgen_func_elem_429: (a: number, b: number) => void;
    readonly __wasm_bindgen_func_elem_970: (a: number, b: number, c: number, d: number) => void;
    readonly __wasm_bindgen_func_elem_928: (a: number, b: number, c: number) => void;
    readonly __wasm_bindgen_func_elem_430: (a: number, b: number) => void;
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
