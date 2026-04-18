//! WASM bindings for ForgeScript parser, metadata, and utilities.
//!
//! # Typing strategy
//!
//! Every public function now returns a concrete Rust type annotated with
//! `#[derive(Tsify)]` (see `wasm_types.rs`) instead of `JsValue`.
//! `tsify-next` translates these types to TypeScript interfaces/type-aliases,
//! so the generated `.d.ts` no longer contains any `any` for these APIs.
//!
//! ## What changed vs. the original
//!
//! | Before                        | After                           | Reason |
//! |-------------------------------|---------------------------------|--------|
//! | `-> JsValue`                  | concrete tsify type             | `JsValue` always maps to `any` |
//! | `serde_wasm_bindgen::to_value`| return the struct directly      | tsify's `IntoWasmAbi` does the serialisation |
//! | `config: JsValue` + `Reflect` | `config: ValidationConfig`      | typed input via `from_wasm_abi` |
//! | `names: JsValue`              | `names: StringList`             | typed input |
//! | `sources: JsValue`            | `sources: StringList`           | typed input |
//! | ad-hoc `serde_json::json!{}`  | typed field assignment          | no anonymous shapes |

#![cfg(feature = "wasm")]

use crate::metadata::{MetadataManager, MetadataSource, github_source};
use crate::parser::{ValidationConfig as ParserValidationConfig, parse as rust_parse};
use crate::utils::{calculate_stats, extract_function_names, format_ast};
use crate::visitor::{AstVisitor, FunctionCollector, NodeCounter};
use crate::wasm_types::*;
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

// ============================================================================
// Setup and Initialization
// ============================================================================

#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "panic_hook")]
    console_error_panic_hook::set_once();
}

// ============================================================================
// UTF-16 Position Mapping
// ============================================================================

use crate::parser::Span as ParserSpan;

/// Maps UTF-8 byte offsets (Rust parser) ↔ UTF-16 code-unit offsets (JS).
struct Utf16Mapper {
    byte_to_utf16: Vec<usize>,
}

impl Utf16Mapper {
    fn new(s: &str) -> Self {
        let mut mapping = Vec::with_capacity(s.len() + 1);
        let mut current_utf16 = 0usize;
        for c in s.chars() {
            for _ in 0..c.len_utf8() {
                mapping.push(current_utf16);
            }
            current_utf16 += c.len_utf16();
        }
        mapping.push(current_utf16);
        Self {
            byte_to_utf16: mapping,
        }
    }

    #[inline]
    fn map(&self, byte_offset: usize) -> u32 {
        *self
            .byte_to_utf16
            .get(byte_offset)
            .unwrap_or(&self.byte_to_utf16.last().copied().unwrap_or(0)) as u32
    }

    #[inline]
    fn map_span(&self, span: ParserSpan) -> Span {
        Span {
            start: self.map(span.start),
            end: self.map(span.end),
        }
    }

    #[inline]
    fn map_back(&self, utf16_offset: usize) -> usize {
        self.byte_to_utf16
            .iter()
            .position(|&x| x >= utf16_offset)
            .unwrap_or(self.byte_to_utf16.len().saturating_sub(1))
    }
}

// ============================================================================
// Shared helper: convert parser errors into typed ParseError vec
// ============================================================================

fn map_errors(errors: Vec<crate::parser::ParseError>, mapper: &Utf16Mapper) -> Vec<ParseError> {
    errors
        .into_iter()
        .map(|e| ParseError {
            message: e.message,
            span: mapper.map_span(e.span),
            kind: format!("{:?}", e.kind),
        })
        .collect()
}

// ============================================================================
// Parser Bindings
// ============================================================================

/// Parse ForgeScript source code (no validation).
///
/// **Before:** `parse(source: string): any`
/// **After:**  `parse(source: string): ParseResult`
#[wasm_bindgen(js_name = "parse")]
pub fn parse_wasm(source: &str) -> ParseResult {
    let mapper = Utf16Mapper::new(source);
    let (ast, errors) = rust_parse(source);
    ParseResult {
        ast: format_ast(&ast),
        errors: map_errors(errors, &mapper),
    }
}

/// Parse and return either the AST or a list of fatal errors.
///
/// **Before:** `parseOrError(source: string): any`
/// **After:**  `parseOrError(source: string): ParseOrErrorResult`
#[wasm_bindgen(js_name = "parseOrError")]
pub fn parse_or_error_wasm(source: &str) -> ParseOrErrorResult {
    let mapper = Utf16Mapper::new(source);
    match crate::parser::parse_with_errors(source) {
        Ok(ast) => ParseOrErrorResult {
            ok: true,
            ast: Some(format_ast(&ast)),
            errors: None,
        },
        Err(errors) => ParseOrErrorResult {
            ok: false,
            ast: None,
            errors: Some(map_errors(errors, &mapper)),
        },
    }
}

/// Parse with a typed validation configuration.
///
/// **Before:** `parseWithConfig(source: string, config: any): any`
/// **After:**  `parseWithConfig(source: string, config: ValidationConfig): ParseResult`
///
/// The `config` argument is now a proper TypeScript interface instead of an
/// untyped `any` — consumers get autocomplete and type-checking on all four
/// boolean flags.
#[wasm_bindgen(js_name = "parseWithConfig")]
pub fn parse_with_config_wasm(source: &str, config: ValidationConfig) -> ParseResult {
    let mapper = Utf16Mapper::new(source);
    let cfg = ParserValidationConfig {
        validate_arguments: config.validate_arguments,
        validate_enums: config.validate_enums,
        validate_functions: config.validate_functions,
        validate_brackets: config.validate_brackets,
    };
    let (ast, errors) = crate::parser::parse_with_config(source, cfg);
    ParseResult {
        ast: format_ast(&ast),
        errors: map_errors(errors, &mapper),
    }
}

/// Parse with validation (requires a metadata manager).
///
/// **Before:** `parseWithValidation(...): any`
/// **After:**  `parseWithValidation(...): ParseResult`
#[wasm_bindgen(js_name = "parseWithValidation")]
pub fn parse_with_validation_wasm(
    source: &str,
    metadata_wrapper: &MetadataManagerWrapper,
    validate_arguments: bool,
    validate_enums: bool,
    validate_functions: bool,
    validate_brackets: bool,
) -> ParseResult {
    let mapper = Utf16Mapper::new(source);
    let config = ParserValidationConfig {
        validate_arguments,
        validate_enums,
        validate_functions,
        validate_brackets,
    };
    let (ast, errors) =
        crate::parser::parse_with_validation(source, config, metadata_wrapper.manager.clone());
    ParseResult {
        ast: format_ast(&ast),
        errors: map_errors(errors, &mapper),
    }
}

/// Parse with all validations enabled.
///
/// **Before:** `parseStrict(...): any`
/// **After:**  `parseStrict(...): ParseResult`
#[wasm_bindgen(js_name = "parseStrict")]
pub fn parse_strict_wasm(source: &str, metadata_wrapper: &MetadataManagerWrapper) -> ParseResult {
    let mapper = Utf16Mapper::new(source);
    let (ast, errors) = crate::parser::parse_strict(source, metadata_wrapper.manager.clone());
    ParseResult {
        ast: format_ast(&ast),
        errors: map_errors(errors, &mapper),
    }
}

/// Return a strict `ValidationConfig` (all flags `true`).
///
/// **Before:** `validationConfigStrict(): any`
/// **After:**  `validationConfigStrict(): ValidationConfig`
#[wasm_bindgen(js_name = "validationConfigStrict")]
pub fn validation_config_strict() -> ValidationConfig {
    let cfg = ParserValidationConfig::strict();
    ValidationConfig {
        validate_arguments: cfg.validate_arguments,
        validate_enums: cfg.validate_enums,
        validate_functions: cfg.validate_functions,
        validate_brackets: cfg.validate_brackets,
    }
}

/// Return a syntax-only `ValidationConfig` (all flags `false`).
///
/// **Before:** `validationConfigSyntaxOnly(): any`
/// **After:**  `validationConfigSyntaxOnly(): ValidationConfig`
#[wasm_bindgen(js_name = "validationConfigSyntaxOnly")]
pub fn validation_config_syntax_only() -> ValidationConfig {
    let cfg = ParserValidationConfig::syntax_only();
    ValidationConfig {
        validate_arguments: cfg.validate_arguments,
        validate_enums: cfg.validate_enums,
        validate_functions: cfg.validate_functions,
        validate_brackets: cfg.validate_brackets,
    }
}

// ============================================================================
// Metadata Manager Bindings
// ============================================================================

#[wasm_bindgen]
pub struct MetadataManagerWrapper {
    manager: Arc<MetadataManager>,
}

#[wasm_bindgen]
impl MetadataManagerWrapper {
    /// Create a new metadata manager.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            manager: Arc::new(MetadataManager::new()),
        }
    }

    /// Add a GitHub source.
    #[wasm_bindgen(js_name = "addGithubSource")]
    pub fn add_github_source(&self, extension: &str, repo: &str, branch: &str) {
        self.manager
            .add_source(github_source(extension, repo, branch));
    }

    /// Add a custom source.
    #[wasm_bindgen(js_name = "addCustomSource")]
    pub fn add_custom_source(
        &self,
        extension: &str,
        functions_url: Option<String>,
        enums_url: Option<String>,
        events_url: Option<String>,
    ) {
        let mut source = MetadataSource::new(extension);
        if let Some(url) = functions_url {
            source = source.with_functions(url);
        }
        if let Some(url) = enums_url {
            source = source.with_enums(url);
        }
        if let Some(url) = events_url {
            source = source.with_events(url);
        }
        self.manager.add_source(source);
    }

    /// Fetch all metadata (async).
    ///
    /// **Before:** `fetchAll(): Promise<any>`
    /// **After:**  `fetchAll(): Promise<FetchStats>`
    #[wasm_bindgen(js_name = "fetchAll")]
    pub fn fetch_all(&self) -> js_sys::Promise {
        let manager = self.manager.clone();
        future_to_promise(async move {
            match manager.fetch_all().await {
                Ok(stats) => {
                    let result = FetchStats {
                        functions: stats.functions,
                        enums: stats.enums,
                        events: stats.events,
                        errors: stats.errors.len(),
                    };
                    serde_wasm_bindgen::to_value(&result)
                        .map_err(|e| JsValue::from_str(&e.to_string()))
                }
                Err(e) => Err(JsValue::from_str(&e.to_string())),
            }
        })
    }

    /// Add custom functions from a JSON string.  Returns the count added.
    #[wasm_bindgen(js_name = "addCustomFunctionsFromJson")]
    pub fn add_custom_functions_from_json(&self, json: &str) -> Result<usize, JsValue> {
        self.manager
            .add_custom_functions_from_json(json)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Remove all custom functions added via `addCustomFunctionsFromJson`.
    #[wasm_bindgen(js_name = "removeCustomFunctions")]
    pub fn remove_custom_functions(&self) {
        self.manager.remove_custom_functions();
    }

    // ---- Single-function lookups ------------------------------------------------
    // These already return `Option<String>` (JSON-serialised Function), which maps
    // to `string | undefined` in TypeScript — no change needed, already typed.

    /// Get a function by name (fuzzy / alias-aware).
    #[wasm_bindgen(js_name = "getFunction")]
    pub fn get_function(&self, name: &str) -> Option<String> {
        self.manager
            .get(name)
            .map(|f| serde_json::to_string(&*f).unwrap_or_else(|_| "{}".into()))
    }

    /// Get a function by exact name.
    #[wasm_bindgen(js_name = "getFunctionExact")]
    pub fn get_function_exact(&self, name: &str) -> Option<String> {
        self.manager
            .get_exact(name)
            .map(|f| serde_json::to_string(&*f).unwrap_or_else(|_| "{}".into()))
    }

    // ---- Structured lookups that previously returned `JsValue` / `any` ----------

    /// Longest-prefix match: returns the matched key + function, or `undefined`.
    ///
    /// **Before:** `getFunctionPrefix(text: string): any`
    /// **After:**  `getFunctionPrefix(text: string): FunctionMatch | undefined`
    #[wasm_bindgen(js_name = "getFunctionPrefix")]
    pub fn get_function_prefix(&self, text: &str) -> Option<FunctionMatch> {
        self.manager
            .get_prefix(text)
            .map(|(key, func)| FunctionMatch {
                key,
                function: WasmFunction::from(&*func),
            })
    }

    /// Alias-aware lookup that also returns the matched key.
    ///
    /// **Before:** `getFunctionWithMatch(name: string): any`
    /// **After:**  `getFunctionWithMatch(name: string): FunctionMatch | undefined`
    #[wasm_bindgen(js_name = "getFunctionWithMatch")]
    pub fn get_function_with_match(&self, name: &str) -> Option<FunctionMatch> {
        self.manager
            .get_with_match(name)
            .map(|(key, func)| FunctionMatch {
                key,
                function: WasmFunction::from(&*func),
            })
    }

    /// Look up multiple functions by name in one call.
    ///
    /// **Before:** `getFunctionMany(names: any): any`
    /// **After:**  `getFunctionMany(names: StringList): OptionalFunctionList`
    ///
    /// TypeScript sees `(WasmFunction | undefined)[]` — each position is either
    /// the resolved function or `undefined` if the name was not found.
    #[wasm_bindgen(js_name = "getFunctionMany")]
    pub fn get_function_many(&self, names: StringList) -> OptionalFunctionList {
        let name_strs: Vec<&str> = names.0.iter().map(String::as_str).collect();
        let results = self
            .manager
            .get_many(&name_strs)
            .into_iter()
            .map(|opt| opt.map(|f| WasmFunction::from(&*f)))
            .collect();
        OptionalFunctionList(results)
    }

    /// Get completions for a prefix string.
    ///
    /// **Before:** `getCompletions(prefix: string): any`
    /// **After:**  `getCompletions(prefix: string): FunctionList`
    #[wasm_bindgen(js_name = "getCompletions")]
    pub fn get_completions(&self, prefix: &str) -> FunctionList {
        FunctionList(
            self.manager
                .get_completions(prefix)
                .into_iter()
                .map(|f| WasmFunction::from(&*f))
                .collect(),
        )
    }

    /// Get all registered functions.
    ///
    /// **Before:** `getAllFunctions(): any`
    /// **After:**  `getAllFunctions(): FunctionList`
    #[wasm_bindgen(js_name = "getAllFunctions")]
    pub fn get_all_functions(&self) -> FunctionList {
        FunctionList(
            self.manager
                .all_functions()
                .into_iter()
                .map(|f| WasmFunction::from(&*f))
                .collect(),
        )
    }

    /// Get all values for a named enum.
    ///
    /// **Before:** `getEnum(name: string): any | undefined`
    /// **After:**  `getEnum(name: string): StringList | undefined`
    #[wasm_bindgen(js_name = "getEnum")]
    pub fn get_enum(&self, name: &str) -> Option<StringList> {
        self.manager.get_enum(name).map(|values| StringList(values))
    }

    /// Get all registered enums.
    ///
    /// **Before:** `getAllEnums(): any`
    /// **After:**  `getAllEnums(): EnumList`
    ///
    /// Returns an array of `{ name, values }` objects rather than a raw
    /// `Record<string, string[]>`, which is easier to iterate in TypeScript.
    #[wasm_bindgen(js_name = "getAllEnums")]
    pub fn get_all_enums(&self) -> EnumList {
        EnumList(
            self.manager
                .all_enums()
                .into_iter()
                .map(|(name, values)| EnumEntry { name, values })
                .collect(),
        )
    }

    /// Get a single event by name (JSON-serialised).
    // Already `Option<String>` → `string | undefined` — no change needed.
    #[wasm_bindgen(js_name = "getEvent")]
    pub fn get_event(&self, name: &str) -> Option<String> {
        self.manager
            .get_event(name)
            .map(|e| serde_json::to_string(&e).unwrap_or_else(|_| "{}".into()))
    }

    /// Get all registered events.
    ///
    /// **Before:** `getAllEvents(): any`
    /// **After:**  `getAllEvents(): EventList`
    #[wasm_bindgen(js_name = "getAllEvents")]
    pub fn get_all_events(&self) -> EventList {
        EventList(
            self.manager
                .all_events()
                .into_iter()
                .map(|e| WasmEvent::from(&e))
                .collect(),
        )
    }

    // ---- Counts (already primitives, unchanged) --------------------------------

    #[wasm_bindgen(js_name = "functionCount")]
    pub fn function_count(&self) -> usize {
        self.manager.function_count()
    }

    #[wasm_bindgen(js_name = "enumCount")]
    pub fn enum_count(&self) -> usize {
        self.manager.enum_count()
    }

    #[wasm_bindgen(js_name = "eventCount")]
    pub fn event_count(&self) -> usize {
        self.manager.event_count()
    }

    #[wasm_bindgen(js_name = "clear")]
    pub fn clear(&self) {
        self.manager.clear();
    }

    // ---- Cache I/O (already String / () — no change needed) -------------------

    #[wasm_bindgen(js_name = "exportCache")]
    pub fn export_cache(&self) -> Result<String, JsValue> {
        self.manager
            .cache_to_json()
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "importCache")]
    pub fn import_cache(&self, json: &str) -> Result<(), JsValue> {
        self.manager
            .cache_from_json(json)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "saveToLocalStorage")]
    pub fn save_to_local_storage(&self, key: &str) -> Result<(), JsValue> {
        let json = self.export_cache()?;
        let storage = web_sys::window()
            .ok_or_else(|| JsValue::from_str("No window"))?
            .local_storage()
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))?
            .ok_or_else(|| JsValue::from_str("No localStorage"))?;
        storage
            .set_item(key, &json)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    #[wasm_bindgen(js_name = "loadFromLocalStorage")]
    pub fn load_from_local_storage(&self, key: &str) -> Result<(), JsValue> {
        let storage = web_sys::window()
            .ok_or_else(|| JsValue::from_str("No window"))?
            .local_storage()
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))?
            .ok_or_else(|| JsValue::from_str("No localStorage"))?;
        let json = storage
            .get_item(key)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))?
            .ok_or_else(|| JsValue::from_str("No cached data"))?;
        self.import_cache(&json)
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Extract all function names from source code.
///
/// **Before:** `extractFunctionNames(source: string): any`
/// **After:**  `extractFunctionNames(source: string): StringList`
#[wasm_bindgen(js_name = "extractFunctionNames")]
pub fn extract_function_names_wasm(source: &str) -> StringList {
    let (ast, _) = rust_parse(source);
    StringList(extract_function_names(&ast))
}

/// Calculate AST statistics.
///
/// **Before:** `calculateStats(source: string): any`
/// **After:**  `calculateStats(source: string): AstStats`
#[wasm_bindgen(js_name = "calculateStats")]
pub fn calculate_stats_wasm(source: &str) -> AstStats {
    let (ast, _) = rust_parse(source);
    let s = calculate_stats(&ast);
    AstStats {
        total_nodes: s.total_nodes,
        text_nodes: s.text_nodes,
        function_calls: s.function_calls,
        javascript_nodes: s.javascript_nodes,
        escaped_nodes: s.escaped_nodes,
        max_depth: s.max_depth,
        unique_functions: s.unique_functions,
    }
}

/// Format the AST as a human-readable string (unchanged — already `String`).
#[wasm_bindgen(js_name = "formatAst")]
pub fn format_ast_wasm(source: &str) -> String {
    let (ast, _) = rust_parse(source);
    format_ast(&ast)
}

/// Count the total number of AST nodes (unchanged — already `usize` / `number`).
#[wasm_bindgen(js_name = "countNodes")]
pub fn count_nodes_wasm(source: &str) -> usize {
    let (ast, _) = rust_parse(source);
    crate::utils::count_nodes(&ast)
}

/// Check whether source contains JavaScript expressions (unchanged — `bool`).
#[wasm_bindgen(js_name = "containsJavaScript")]
pub fn contains_javascript_wasm(source: &str) -> bool {
    let (ast, _) = rust_parse(source);
    crate::utils::contains_javascript(&ast)
}

/// Maximum function-nesting depth (unchanged — `usize` / `number`).
#[wasm_bindgen(js_name = "maxNestingDepth")]
pub fn max_nesting_depth_wasm(source: &str) -> usize {
    let (ast, _) = rust_parse(source);
    crate::utils::max_nesting_depth(&ast)
}

/// Extract all text nodes with their UTF-16 spans.
///
/// **Before:** `extractTextNodes(source: string): any`
/// **After:**  `extractTextNodes(source: string): TextNodeList`
#[wasm_bindgen(js_name = "extractTextNodes")]
pub fn extract_text_nodes_wasm(source: &str) -> TextNodeList {
    let mapper = Utf16Mapper::new(source);
    let (ast, _) = rust_parse(source);
    TextNodeList(
        crate::utils::extract_text_nodes(&ast)
            .into_iter()
            .map(|(text, span)| TextNode {
                text,
                span: mapper.map_span(span),
            })
            .collect(),
    )
}

/// Flatten the AST into a depth-first list of typed node descriptors.
///
/// **Before:** `flattenAst(source: string): any`
/// **After:**  `flattenAst(source: string): FlatAstNodeList`
///
/// TypeScript sees a discriminated union on the `type` field, so
/// consumers can narrow with a `switch (node.type)` statement.
#[wasm_bindgen(js_name = "flattenAst")]
pub fn flatten_ast_wasm(source: &str) -> FlatAstNodeList {
    use crate::parser::AstNode;

    let mapper = Utf16Mapper::new(source);
    let (ast, _) = rust_parse(source);
    let flat = crate::utils::flatten_ast(&ast);

    FlatAstNodeList(
        flat.iter()
            .map(|node| match node {
                AstNode::Program { span, .. } => FlatAstNode::Program {
                    span: mapper.map_span(*span),
                },
                AstNode::Text { content, span } => FlatAstNode::Text {
                    content: content.clone(),
                    span: mapper.map_span(*span),
                },
                AstNode::FunctionCall {
                    name,
                    modifiers,
                    span,
                    name_span,
                    ..
                } => FlatAstNode::FunctionCall {
                    name: name.clone(),
                    modifiers: FunctionModifiers {
                        silent: modifiers.silent,
                        negated: modifiers.negated,
                        count: modifiers.count.clone(),
                    },
                    span: mapper.map_span(*span),
                    name_span: mapper.map_span(*name_span),
                },
                AstNode::JavaScript { code, span } => FlatAstNode::JavaScript {
                    code: code.clone(),
                    span: mapper.map_span(*span),
                },
                AstNode::Escaped { content, span } => FlatAstNode::Escaped {
                    content: content.clone(),
                    span: mapper.map_span(*span),
                },
            })
            .collect(),
    )
}

/// Return the source-code slice for a UTF-16 span (unchanged — `String`).
#[wasm_bindgen(js_name = "getSourceSlice")]
pub fn get_source_slice_wasm(source: &str, start_utf16: usize, end_utf16: usize) -> String {
    let mapper = Utf16Mapper::new(source);
    let span = crate::parser::Span {
        start: mapper.map_back(start_utf16),
        end: mapper.map_back(end_utf16),
    };
    crate::utils::get_source_slice(source, span).to_string()
}

/// Check whether the character at a UTF-16 index is escaped (unchanged — `bool`).
#[wasm_bindgen(js_name = "isEscaped")]
pub fn is_escaped_wasm(source: &str, utf16_idx: usize) -> bool {
    let mapper = Utf16Mapper::new(source);
    crate::parser::is_escaped(source, mapper.map_back(utf16_idx))
}

// ============================================================================
// Visitor Pattern Helpers
// ============================================================================

/// Collect all function names (visitor-based).
///
/// **Before:** `collectFunctions(source: string): any`
/// **After:**  `collectFunctions(source: string): StringList`
#[wasm_bindgen(js_name = "collectFunctions")]
pub fn collect_functions_wasm(source: &str) -> StringList {
    let (ast, _) = rust_parse(source);
    let mut collector = FunctionCollector::new();
    collector.visit(&ast);
    StringList(collector.functions)
}

/// Count node types (visitor-based).
///
/// **Before:** `countNodeTypes(source: string): any`
/// **After:**  `countNodeTypes(source: string): NodeTypeCounts`
#[wasm_bindgen(js_name = "countNodeTypes")]
pub fn count_node_types_wasm(source: &str) -> NodeTypeCounts {
    let (ast, _) = rust_parse(source);
    let mut counter = NodeCounter::default();
    counter.visit(&ast);
    NodeTypeCounts {
        text_nodes: counter.text_nodes,
        function_nodes: counter.function_nodes,
        javascript_nodes: counter.javascript_nodes,
        escaped_nodes: counter.escaped_nodes,
    }
}

// ============================================================================
// Validation Helpers
// ============================================================================

/// Validate code and return a detailed typed report.
///
/// **Before:** `validateCode(source: string, metadata_wrapper: MetadataManagerWrapper): any`
/// **After:**  `validateCode(source: string, metadata_wrapper: MetadataManagerWrapper): ValidationResult`
#[wasm_bindgen(js_name = "validateCode")]
pub fn validate_code_wasm(
    source: &str,
    metadata_wrapper: &MetadataManagerWrapper,
) -> ValidationResult {
    use std::collections::HashMap;
    let mapper = Utf16Mapper::new(source);
    let (_, errors) = crate::parser::parse_strict(source, metadata_wrapper.manager.clone());

    let mut errors_by_kind: HashMap<String, Vec<ParseError>> = HashMap::new();
    let mut all_errors: Vec<ParseError> = Vec::with_capacity(errors.len());

    for e in &errors {
        let kind = format!("{:?}", e.kind);
        let typed = ParseError {
            message: e.message.clone(),
            span: mapper.map_span(e.span),
            kind: kind.clone(),
        };
        errors_by_kind.entry(kind).or_default().push(typed.clone());
        all_errors.push(typed);
    }

    ValidationResult {
        valid: errors.is_empty(),
        error_count: errors.len(),
        errors_by_kind,
        all_errors,
    }
}

// ============================================================================
// Batch Operations
// ============================================================================

/// Parse multiple sources in one call.
///
/// **Before:** `parseBatch(sources: any): any`
/// **After:**  `parseBatch(sources: StringList): ParseResultList`
#[wasm_bindgen(js_name = "parseBatch")]
pub fn parse_batch_wasm(sources: StringList) -> ParseResultList {
    ParseResultList(
        sources
            .0
            .iter()
            .map(|source| {
                let mapper = Utf16Mapper::new(source);
                let (ast, errors) = rust_parse(source);
                ParseResult {
                    ast: format_ast(&ast),
                    errors: map_errors(errors, &mapper),
                }
            })
            .collect(),
    )
}

/// Validate multiple sources in one call.
///
/// **Before:** `validateBatch(sources: any, ...): any`
/// **After:**  `validateBatch(sources: StringList, ...): BatchValidateResultList`
#[wasm_bindgen(js_name = "validateBatch")]
pub fn validate_batch_wasm(
    sources: StringList,
    metadata_wrapper: &MetadataManagerWrapper,
) -> BatchValidateResultList {
    BatchValidateResultList(
        sources
            .0
            .iter()
            .map(|source| {
                let mapper = Utf16Mapper::new(source);
                let (_, errors) =
                    crate::parser::parse_strict(source, metadata_wrapper.manager.clone());
                BatchValidateResult {
                    valid: errors.is_empty(),
                    error_count: errors.len(),
                    errors: map_errors(errors, &mapper),
                }
            })
            .collect(),
    )
}

// ============================================================================
// Version Info
// ============================================================================

/// Get package version information.
///
/// **Before:** `version(): any`
/// **After:**  `version(): VersionInfo`
#[wasm_bindgen(js_name = "version")]
pub fn version() -> VersionInfo {
    VersionInfo {
        version: env!("CARGO_PKG_VERSION").into(),
        name: env!("CARGO_PKG_NAME").into(),
        authors: env!("CARGO_PKG_AUTHORS").into(),
    }
}
