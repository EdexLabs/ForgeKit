/* tslint:disable */
/* eslint-disable */
/**
 * A UTF-16 source span (start/end code-unit offsets).
 *
 * All spans exposed to JS use UTF-16 units so they map directly to
 * JavaScript string indices.
 */
export interface Span {
    start: number;
    end: number;
}

/**
 * A single node in the depth-first-flattened AST.
 *
 * The `type` field is the serde tag that discriminates the variant, so
 * TypeScript gets a proper discriminated union:
 *
 * ```ts
 * type FlatAstNode =
 *   | { type: \"Program\"; span: Span }
 *   | { type: \"Text\"; content: string; span: Span }
 *   | { type: \"FunctionCall\"; name: string; modifiers: FunctionModifiers;
 *       span: Span; name_span: Span }
 *   | { type: \"JavaScript\"; code: string; span: Span }
 *   | { type: \"Escaped\"; content: string; span: Span };
 * ```
 */
export type FlatAstNode = { type: "Program"; span: Span } | { type: "Text"; content: string; span: Span } | { type: "FunctionCall"; name: string; modifiers: FunctionModifiers; span: Span; name_span: Span } | { type: "JavaScript"; code: string; span: Span } | { type: "Escaped"; content: string; span: Span };

/**
 * A single parse or validation error.
 */
export interface ParseError {
    message: string;
    span: Span;
    /**
     * Debug name of the error kind enum variant (e.g. `\"UnknownFunction\"`).
     */
    kind: string;
}

/**
 * A text leaf node together with its UTF-16 span.
 */
export interface TextNode {
    text: string;
    span: Span;
}

/**
 * An enum registry entry: the enum\'s name and its list of allowed values.
 */
export interface EnumEntry {
    name: string;
    values: string[];
}

/**
 * Detailed validation report returned by `validateCode`.
 */
export interface ValidationResult {
    valid: boolean;
    errorCount: number;
    /**
     * Errors grouped by their kind string.
     * TypeScript type: `{ [kind: string]: ParseError[] }`
     */
    errorsByKind: Map<string, ParseError[]>;
    allErrors: ParseError[];
}

/**
 * Modifier flags attached to a function-call node.
 */
export interface FunctionModifiers {
    silent: boolean;
    negated: boolean;
    count: string | undefined;
}

/**
 * Node-type counts produced by the visitor-based `countNodeTypes`.
 */
export interface NodeTypeCounts {
    textNodes: number;
    functionNodes: number;
    javascriptNodes: number;
    escapedNodes: number;
}

/**
 * Package metadata returned by `version()`.
 */
export interface VersionInfo {
    version: string;
    name: string;
    authors: string;
}

/**
 * Per-source result inside a `validateBatch` response.
 */
export interface BatchValidateResult {
    valid: boolean;
    errorCount: number;
    errors: ParseError[];
}

/**
 * Result of `parseOrError`: either a successful AST or a list of
 * fatal errors — never both.
 */
export interface ParseOrErrorResult {
    ok: boolean;
    ast?: string;
    errors?: ParseError[];
}

/**
 * Result of a `parse*` call: the pretty-printed AST string plus any
 * non-fatal errors collected during parsing.
 */
export interface ParseResult {
    /**
     * Human-readable AST representation (same as `formatAst`).
     */
    ast: string;
    errors: ParseError[];
}

/**
 * Statistics returned by a successful `fetchAll` call.
 */
export interface FetchStats {
    functions: number;
    enums: number;
    events: number;
    /**
     * Number of sources that failed to fetch.
     */
    errors: number;
}

/**
 * Summary statistics about a parsed AST.
 */
export interface AstStats {
    totalNodes: number;
    textNodes: number;
    functionCalls: number;
    javascriptNodes: number;
    escapedNodes: number;
    maxDepth: number;
    uniqueFunctions: number;
}

/**
 * The result of `getFunctionPrefix` / `getFunctionWithMatch` — the
 * resolved function together with the key that was actually matched
 * (which may be an alias rather than the canonical name).
 */
export interface FunctionMatch {
    /**
     * The registry key that was matched (canonical name or alias).
     */
    key: string;
    function: WasmFunction;
}

/**
 * Validation configuration passed to `parseWithConfig`.
 *
 * Mirrors `parser::ValidationConfig` but lives on the WASM boundary so
 * `tsify` can generate a proper TypeScript interface for it.
 */
export interface ValidationConfig {
    validateArguments: boolean;
    validateEnums: boolean;
    validateFunctions: boolean;
    validateBrackets: boolean;
}

/**
 * WASM-boundary event definition (mirrors `types::Event`).
 */
export interface WasmEvent {
    name: string;
    description: string;
    fields: WasmEventField[] | undefined;
}

/**
 * WASM-boundary event field (mirrors `types::EventField`).
 */
export interface WasmEventField {
    name: string;
    description: string;
}

/**
 * WASM-boundary representation of a ForgeScript function definition.
 *
 * The internal `Function` type uses `serde_json::Value` for `version`,
 * `output`, and `arg_type` (and a catch-all `extra` map) — all of which
 * would generate `any` at the boundary.  Here every field is a concrete
 * Rust type.  Dynamic JSON blobs become `Option<String>` (compact JSON),
 * which TypeScript sees as `string | undefined` — strongly typed in the
 * sense that consumers know it\'s a serialised JSON payload, not an opaque
 * `any`.
 *
 * `local_path` (an OS path, irrelevant in a browser context) is omitted.
 * `extra` (forward-compat catch-all) is also omitted; callers should not
 * rely on undocumented fields.
 */
export interface WasmFunction {
    name: string;
    /**
     * Semver string or other version token, JSON-encoded if complex.
     */
    version: string | undefined;
    description: string;
    brackets: boolean | undefined;
    unwrap: boolean;
    args: WasmArg[] | undefined;
    /**
     * JSON-encoded output-type descriptor.
     */
    output: string | undefined;
    category: string | undefined;
    aliases: string[] | undefined;
    experimental: boolean | undefined;
    examples: string[] | undefined;
    deprecated: boolean | undefined;
    extension: string | undefined;
    source_url: string | undefined;
    line: number | undefined;
}

/**
 * WASM-boundary representation of a function argument.
 *
 * Replaces the internal `Arg` whose `arg_type` field is `serde_json::Value`.
 * Here we serialise dynamic fields to their JSON string representations so
 * the boundary type stays fully typed (`string` is far better than `any`).
 */
export interface WasmArg {
    name: string;
    description: string;
    rest: boolean;
    required: boolean | undefined;
    /**
     * JSON-encoded type descriptor.  May be a bare string like `\"string\"`,
     * an array like `[\"string\",\"number\"]`, or a richer object — serialised
     * here so the boundary stays `string` rather than `any`.
     */
    type: string;
    condition: boolean | undefined;
    enum: string[] | undefined;
    enum_name: string | undefined;
    pointer: number | undefined;
    pointer_property: string | undefined;
}

/**
 * `(WasmFunction | undefined)[]` — used by `getFunctionMany` where a
 * name may not resolve.
 */
export type OptionalFunctionList = (WasmFunction | undefined)[];

/**
 * `BatchValidateResult[]`
 */
export type BatchValidateResultList = BatchValidateResult[];

/**
 * `EnumEntry[]`
 */
export type EnumList = EnumEntry[];

/**
 * `FlatAstNode[]`
 */
export type FlatAstNodeList = FlatAstNode[];

/**
 * `ParseResult[]`
 */
export type ParseResultList = ParseResult[];

/**
 * `TextNode[]`
 */
export type TextNodeList = TextNode[];

/**
 * `WasmEvent[]`
 */
export type EventList = WasmEvent[];

/**
 * `WasmFunction[]`
 */
export type FunctionList = WasmFunction[];

/**
 * `string[]` — used wherever a plain list of strings crosses the boundary.
 */
export type StringList = string[];


export class MetadataManagerWrapper {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Add custom functions from a JSON string.  Returns the count added.
     */
    addCustomFunctionsFromJson(json: string): number;
    /**
     * Add a custom source.
     */
    addCustomSource(extension: string, functions_url?: string | null, enums_url?: string | null, events_url?: string | null): void;
    /**
     * Add a GitHub source.
     */
    addGithubSource(extension: string, repo: string, branch: string): void;
    clear(): void;
    enumCount(): number;
    eventCount(): number;
    exportCache(): string;
    /**
     * Fetch all metadata (async).
     *
     * **Before:** `fetchAll(): Promise<any>`
     * **After:**  `fetchAll(): Promise<FetchStats>`
     */
    fetchAll(): Promise<any>;
    functionCount(): number;
    /**
     * Get all registered enums.
     *
     * **Before:** `getAllEnums(): any`
     * **After:**  `getAllEnums(): EnumList`
     *
     * Returns an array of `{ name, values }` objects rather than a raw
     * `Record<string, string[]>`, which is easier to iterate in TypeScript.
     */
    getAllEnums(): EnumList;
    /**
     * Get all registered events.
     *
     * **Before:** `getAllEvents(): any`
     * **After:**  `getAllEvents(): EventList`
     */
    getAllEvents(): EventList;
    /**
     * Get all registered functions.
     *
     * **Before:** `getAllFunctions(): any`
     * **After:**  `getAllFunctions(): FunctionList`
     */
    getAllFunctions(): FunctionList;
    /**
     * Get completions for a prefix string.
     *
     * **Before:** `getCompletions(prefix: string): any`
     * **After:**  `getCompletions(prefix: string): FunctionList`
     */
    getCompletions(prefix: string): FunctionList;
    /**
     * Get all values for a named enum.
     *
     * **Before:** `getEnum(name: string): any | undefined`
     * **After:**  `getEnum(name: string): StringList | undefined`
     */
    getEnum(name: string): StringList | undefined;
    /**
     * Get a single event by name (JSON-serialised).
     */
    getEvent(name: string): string | undefined;
    /**
     * Get a function by name (fuzzy / alias-aware).
     */
    getFunction(name: string): string | undefined;
    /**
     * Get a function by exact name.
     */
    getFunctionExact(name: string): string | undefined;
    /**
     * Look up multiple functions by name in one call.
     *
     * **Before:** `getFunctionMany(names: any): any`
     * **After:**  `getFunctionMany(names: StringList): OptionalFunctionList`
     *
     * TypeScript sees `(WasmFunction | undefined)[]` — each position is either
     * the resolved function or `undefined` if the name was not found.
     */
    getFunctionMany(names: StringList): OptionalFunctionList;
    /**
     * Longest-prefix match: returns the matched key + function, or `undefined`.
     *
     * **Before:** `getFunctionPrefix(text: string): any`
     * **After:**  `getFunctionPrefix(text: string): FunctionMatch | undefined`
     */
    getFunctionPrefix(text: string): FunctionMatch | undefined;
    /**
     * Alias-aware lookup that also returns the matched key.
     *
     * **Before:** `getFunctionWithMatch(name: string): any`
     * **After:**  `getFunctionWithMatch(name: string): FunctionMatch | undefined`
     */
    getFunctionWithMatch(name: string): FunctionMatch | undefined;
    importCache(json: string): void;
    loadFromLocalStorage(key: string): void;
    /**
     * Create a new metadata manager.
     */
    constructor();
    /**
     * Remove all custom functions added via `addCustomFunctionsFromJson`.
     */
    removeCustomFunctions(): void;
    saveToLocalStorage(key: string): void;
}

/**
 * Calculate AST statistics.
 *
 * **Before:** `calculateStats(source: string): any`
 * **After:**  `calculateStats(source: string): AstStats`
 */
export function calculateStats(source: string): AstStats;

/**
 * Collect all function names (visitor-based).
 *
 * **Before:** `collectFunctions(source: string): any`
 * **After:**  `collectFunctions(source: string): StringList`
 */
export function collectFunctions(source: string): StringList;

/**
 * Check whether source contains JavaScript expressions (unchanged — `bool`).
 */
export function containsJavaScript(source: string): boolean;

/**
 * Count node types (visitor-based).
 *
 * **Before:** `countNodeTypes(source: string): any`
 * **After:**  `countNodeTypes(source: string): NodeTypeCounts`
 */
export function countNodeTypes(source: string): NodeTypeCounts;

/**
 * Count the total number of AST nodes (unchanged — already `usize` / `number`).
 */
export function countNodes(source: string): number;

/**
 * Extract all function names from source code.
 *
 * **Before:** `extractFunctionNames(source: string): any`
 * **After:**  `extractFunctionNames(source: string): StringList`
 */
export function extractFunctionNames(source: string): StringList;

/**
 * Extract all text nodes with their UTF-16 spans.
 *
 * **Before:** `extractTextNodes(source: string): any`
 * **After:**  `extractTextNodes(source: string): TextNodeList`
 */
export function extractTextNodes(source: string): TextNodeList;

/**
 * Flatten the AST into a depth-first list of typed node descriptors.
 *
 * **Before:** `flattenAst(source: string): any`
 * **After:**  `flattenAst(source: string): FlatAstNodeList`
 *
 * TypeScript sees a discriminated union on the `type` field, so
 * consumers can narrow with a `switch (node.type)` statement.
 */
export function flattenAst(source: string): FlatAstNodeList;

/**
 * Format the AST as a human-readable string (unchanged — already `String`).
 */
export function formatAst(source: string): string;

/**
 * Return the source-code slice for a UTF-16 span (unchanged — `String`).
 */
export function getSourceSlice(source: string, start_utf16: number, end_utf16: number): string;

export function init(): void;

/**
 * Check whether the character at a UTF-16 index is escaped (unchanged — `bool`).
 */
export function isEscaped(source: string, utf16_idx: number): boolean;

/**
 * Maximum function-nesting depth (unchanged — `usize` / `number`).
 */
export function maxNestingDepth(source: string): number;

/**
 * Parse ForgeScript source code (no validation).
 *
 * **Before:** `parse(source: string): any`
 * **After:**  `parse(source: string): ParseResult`
 */
export function parse(source: string): ParseResult;

/**
 * Parse multiple sources in one call.
 *
 * **Before:** `parseBatch(sources: any): any`
 * **After:**  `parseBatch(sources: StringList): ParseResultList`
 */
export function parseBatch(sources: StringList): ParseResultList;

/**
 * Parse and return either the AST or a list of fatal errors.
 *
 * **Before:** `parseOrError(source: string): any`
 * **After:**  `parseOrError(source: string): ParseOrErrorResult`
 */
export function parseOrError(source: string): ParseOrErrorResult;

/**
 * Parse with all validations enabled.
 *
 * **Before:** `parseStrict(...): any`
 * **After:**  `parseStrict(...): ParseResult`
 */
export function parseStrict(source: string, metadata_wrapper: MetadataManagerWrapper): ParseResult;

/**
 * Parse with a typed validation configuration.
 *
 * **Before:** `parseWithConfig(source: string, config: any): any`
 * **After:**  `parseWithConfig(source: string, config: ValidationConfig): ParseResult`
 *
 * The `config` argument is now a proper TypeScript interface instead of an
 * untyped `any` — consumers get autocomplete and type-checking on all four
 * boolean flags.
 */
export function parseWithConfig(source: string, config: ValidationConfig): ParseResult;

/**
 * Parse with validation (requires a metadata manager).
 *
 * **Before:** `parseWithValidation(...): any`
 * **After:**  `parseWithValidation(...): ParseResult`
 */
export function parseWithValidation(source: string, metadata_wrapper: MetadataManagerWrapper, validate_arguments: boolean, validate_enums: boolean, validate_functions: boolean, validate_brackets: boolean): ParseResult;

/**
 * Validate multiple sources in one call.
 *
 * **Before:** `validateBatch(sources: any, ...): any`
 * **After:**  `validateBatch(sources: StringList, ...): BatchValidateResultList`
 */
export function validateBatch(sources: StringList, metadata_wrapper: MetadataManagerWrapper): BatchValidateResultList;

/**
 * Validate code and return a detailed typed report.
 *
 * **Before:** `validateCode(source: string, metadata_wrapper: MetadataManagerWrapper): any`
 * **After:**  `validateCode(source: string, metadata_wrapper: MetadataManagerWrapper): ValidationResult`
 */
export function validateCode(source: string, metadata_wrapper: MetadataManagerWrapper): ValidationResult;

/**
 * Return a strict `ValidationConfig` (all flags `true`).
 *
 * **Before:** `validationConfigStrict(): any`
 * **After:**  `validationConfigStrict(): ValidationConfig`
 */
export function validationConfigStrict(): ValidationConfig;

/**
 * Return a syntax-only `ValidationConfig` (all flags `false`).
 *
 * **Before:** `validationConfigSyntaxOnly(): any`
 * **After:**  `validationConfigSyntaxOnly(): ValidationConfig`
 */
export function validationConfigSyntaxOnly(): ValidationConfig;

/**
 * Get package version information.
 *
 * **Before:** `version(): any`
 * **After:**  `version(): VersionInfo`
 */
export function version(): VersionInfo;

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
    readonly __wasm_bindgen_func_elem_1190: (a: number, b: number) => void;
    readonly __wasm_bindgen_func_elem_549: (a: number, b: number) => void;
    readonly __wasm_bindgen_func_elem_1232: (a: number, b: number, c: number, d: number) => void;
    readonly __wasm_bindgen_func_elem_1191: (a: number, b: number, c: number) => void;
    readonly __wasm_bindgen_func_elem_550: (a: number, b: number) => void;
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
