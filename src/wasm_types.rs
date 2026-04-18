//! Strongly-typed WASM-boundary structs for ForgeKit.
//!
//! Every type here is annotated with `#[derive(Tsify)]` so that
//! `wasm-pack` emits accurate TypeScript interfaces/types instead of
//! `any`.  The rule is simple: **nothing that crosses the WASM boundary
//! may be `JsValue` or `serde_json::Value`**.
//!
//! Internal Rust types (`Function`, `Arg`, `Event`, …) stay untouched;
//! we convert them into these boundary types at the call-site in
//! `wasm.rs`.

use crate::types::{Arg, Event, EventField, Function};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

// ============================================================================
// Shared primitives
// ============================================================================

/// A UTF-16 source span (start/end code-unit offsets).
///
/// All spans exposed to JS use UTF-16 units so they map directly to
/// JavaScript string indices.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

// ============================================================================
// Parser / validation types
// ============================================================================

/// A single parse or validation error.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
    /// Debug name of the error kind enum variant (e.g. `"UnknownFunction"`).
    pub kind: String,
}

/// Result of a `parse*` call: the pretty-printed AST string plus any
/// non-fatal errors collected during parsing.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct ParseResult {
    /// Human-readable AST representation (same as `formatAst`).
    pub ast: String,
    pub errors: Vec<ParseError>,
}

/// Result of `parseOrError`: either a successful AST or a list of
/// fatal errors — never both.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct ParseOrErrorResult {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ast: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<ParseError>>,
}

/// Validation configuration passed to `parseWithConfig`.
///
/// Mirrors `parser::ValidationConfig` but lives on the WASM boundary so
/// `tsify` can generate a proper TypeScript interface for it.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct ValidationConfig {
    #[serde(rename = "validateArguments")]
    pub validate_arguments: bool,
    #[serde(rename = "validateEnums")]
    pub validate_enums: bool,
    #[serde(rename = "validateFunctions")]
    pub validate_functions: bool,
    #[serde(rename = "validateBrackets")]
    pub validate_brackets: bool,
}

/// Detailed validation report returned by `validateCode`.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct ValidationResult {
    pub valid: bool,
    #[serde(rename = "errorCount")]
    pub error_count: usize,
    /// Errors grouped by their kind string.
    /// TypeScript type: `{ [kind: string]: ParseError[] }`
    #[serde(rename = "errorsByKind")]
    pub errors_by_kind: HashMap<String, Vec<ParseError>>,
    #[serde(rename = "allErrors")]
    pub all_errors: Vec<ParseError>,
}

/// Per-source result inside a `validateBatch` response.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct BatchValidateResult {
    pub valid: bool,
    #[serde(rename = "errorCount")]
    pub error_count: usize,
    pub errors: Vec<ParseError>,
}

// ============================================================================
// AST / utility types
// ============================================================================

/// Summary statistics about a parsed AST.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct AstStats {
    #[serde(rename = "totalNodes")]
    pub total_nodes: usize,
    #[serde(rename = "textNodes")]
    pub text_nodes: usize,
    #[serde(rename = "functionCalls")]
    pub function_calls: usize,
    #[serde(rename = "javascriptNodes")]
    pub javascript_nodes: usize,
    #[serde(rename = "escapedNodes")]
    pub escaped_nodes: usize,
    #[serde(rename = "maxDepth")]
    pub max_depth: usize,
    #[serde(rename = "uniqueFunctions")]
    pub unique_functions: usize,
}

/// A text leaf node together with its UTF-16 span.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct TextNode {
    pub text: String,
    pub span: Span,
}

/// Modifier flags attached to a function-call node.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct FunctionModifiers {
    pub silent: bool,
    pub negated: bool,
    pub count: Option<String>,
}

/// A single node in the depth-first-flattened AST.
///
/// The `type` field is the serde tag that discriminates the variant, so
/// TypeScript gets a proper discriminated union:
///
/// ```ts
/// type FlatAstNode =
///   | { type: "Program"; span: Span }
///   | { type: "Text"; content: string; span: Span }
///   | { type: "FunctionCall"; name: string; modifiers: FunctionModifiers;
///       span: Span; name_span: Span }
///   | { type: "JavaScript"; code: string; span: Span }
///   | { type: "Escaped"; content: string; span: Span };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "type")]
pub enum FlatAstNode {
    Program {
        span: Span,
    },
    Text {
        content: String,
        span: Span,
    },
    FunctionCall {
        name: String,
        modifiers: FunctionModifiers,
        span: Span,
        name_span: Span,
    },
    JavaScript {
        code: String,
        span: Span,
    },
    Escaped {
        content: String,
        span: Span,
    },
}

/// Node-type counts produced by the visitor-based `countNodeTypes`.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct NodeTypeCounts {
    #[serde(rename = "textNodes")]
    pub text_nodes: usize,
    #[serde(rename = "functionNodes")]
    pub function_nodes: usize,
    #[serde(rename = "javascriptNodes")]
    pub javascript_nodes: usize,
    #[serde(rename = "escapedNodes")]
    pub escaped_nodes: usize,
}

// ============================================================================
// Newtype wrappers for Vec returns
//
// wasm-bindgen cannot return `Vec<T>` for arbitrary `T` directly.
// Wrapping in a newtype struct that derives Tsify generates a TypeScript
// type alias (e.g. `type FunctionList = WasmFunction[]`) and provides
// `IntoWasmAbi` so the function signature compiles cleanly.
// ============================================================================

/// `WasmFunction[]`
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct FunctionList(pub Vec<WasmFunction>);

/// `(WasmFunction | undefined)[]` — used by `getFunctionMany` where a
/// name may not resolve.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct OptionalFunctionList(pub Vec<Option<WasmFunction>>);

/// `string[]` — used wherever a plain list of strings crosses the boundary.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct StringList(pub Vec<String>);

/// `WasmEvent[]`
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct EventList(pub Vec<WasmEvent>);

/// `EnumEntry[]`
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct EnumList(pub Vec<EnumEntry>);

/// `ParseResult[]`
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct ParseResultList(pub Vec<ParseResult>);

/// `BatchValidateResult[]`
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct BatchValidateResultList(pub Vec<BatchValidateResult>);

/// `FlatAstNode[]`
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct FlatAstNodeList(pub Vec<FlatAstNode>);

/// `TextNode[]`
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct TextNodeList(pub Vec<TextNode>);

// ============================================================================
// Metadata types
// ============================================================================

/// WASM-boundary representation of a function argument.
///
/// Replaces the internal `Arg` whose `arg_type` field is `serde_json::Value`.
/// Here we serialise dynamic fields to their JSON string representations so
/// the boundary type stays fully typed (`string` is far better than `any`).
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmArg {
    pub name: String,
    pub description: String,
    pub rest: bool,
    pub required: Option<bool>,
    /// JSON-encoded type descriptor.  May be a bare string like `"string"`,
    /// an array like `["string","number"]`, or a richer object — serialised
    /// here so the boundary stays `string` rather than `any`.
    #[serde(rename = "type")]
    pub arg_type: String,
    pub condition: Option<bool>,
    #[serde(rename = "enum")]
    pub arg_enum: Option<Vec<String>>,
    pub enum_name: Option<String>,
    pub pointer: Option<i64>,
    pub pointer_property: Option<String>,
}

impl From<&Arg> for WasmArg {
    fn from(a: &Arg) -> Self {
        WasmArg {
            name: a.name.clone(),
            description: a.description.clone(),
            rest: a.rest,
            required: a.required,
            // Serialise JsonValue → compact JSON string; falls back to "null"
            arg_type: serde_json::to_string(&a.arg_type).unwrap_or_else(|_| "null".into()),
            condition: a.condition,
            arg_enum: a.arg_enum.clone(),
            enum_name: a.enum_name.clone(),
            pointer: a.pointer,
            pointer_property: a.pointer_property.clone(),
        }
    }
}

/// WASM-boundary representation of a ForgeScript function definition.
///
/// The internal `Function` type uses `serde_json::Value` for `version`,
/// `output`, and `arg_type` (and a catch-all `extra` map) — all of which
/// would generate `any` at the boundary.  Here every field is a concrete
/// Rust type.  Dynamic JSON blobs become `Option<String>` (compact JSON),
/// which TypeScript sees as `string | undefined` — strongly typed in the
/// sense that consumers know it's a serialised JSON payload, not an opaque
/// `any`.
///
/// `local_path` (an OS path, irrelevant in a browser context) is omitted.
/// `extra` (forward-compat catch-all) is also omitted; callers should not
/// rely on undocumented fields.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmFunction {
    pub name: String,
    /// Semver string or other version token, JSON-encoded if complex.
    pub version: Option<String>,
    pub description: String,
    pub brackets: Option<bool>,
    pub unwrap: bool,
    pub args: Option<Vec<WasmArg>>,
    /// JSON-encoded output-type descriptor.
    pub output: Option<String>,
    pub category: Option<String>,
    pub aliases: Option<Vec<String>>,
    pub experimental: Option<bool>,
    pub examples: Option<Vec<String>>,
    pub deprecated: Option<bool>,
    pub extension: Option<String>,
    pub source_url: Option<String>,
    pub line: Option<u32>,
}

impl From<&Function> for WasmFunction {
    fn from(f: &Function) -> Self {
        WasmFunction {
            name: f.name.clone(),
            version: f
                .version
                .as_ref()
                .map(|v| serde_json::to_string(v).unwrap_or_else(|_| "null".into())),
            description: f.description.clone(),
            brackets: f.brackets,
            unwrap: f.unwrap,
            args: f
                .args
                .as_ref()
                .map(|args| args.iter().map(WasmArg::from).collect()),
            output: f
                .output
                .as_ref()
                .map(|v| serde_json::to_string(v).unwrap_or_else(|_| "null".into())),
            category: f.category.clone(),
            aliases: f.aliases.clone(),
            experimental: f.experimental,
            examples: f.examples.clone(),
            deprecated: f.deprecated,
            extension: f.extension.clone(),
            source_url: f.source_url.clone(),
            line: f.line,
        }
    }
}

impl From<Function> for WasmFunction {
    fn from(f: Function) -> Self {
        WasmFunction::from(&f)
    }
}

/// The result of `getFunctionPrefix` / `getFunctionWithMatch` — the
/// resolved function together with the key that was actually matched
/// (which may be an alias rather than the canonical name).
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct FunctionMatch {
    /// The registry key that was matched (canonical name or alias).
    pub key: String,
    pub function: WasmFunction,
}

/// An enum registry entry: the enum's name and its list of allowed values.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct EnumEntry {
    pub name: String,
    pub values: Vec<String>,
}

/// WASM-boundary event field (mirrors `types::EventField`).
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmEventField {
    pub name: String,
    pub description: String,
}

impl From<&EventField> for WasmEventField {
    fn from(f: &EventField) -> Self {
        WasmEventField {
            name: f.name.clone(),
            description: f.description.clone(),
        }
    }
}

/// WASM-boundary event definition (mirrors `types::Event`).
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmEvent {
    pub name: String,
    pub description: String,
    pub fields: Option<Vec<WasmEventField>>,
}

impl From<&Event> for WasmEvent {
    fn from(e: &Event) -> Self {
        WasmEvent {
            name: e.name.clone(),
            description: e.description.clone(),
            fields: e
                .fields
                .as_ref()
                .map(|fs| fs.iter().map(WasmEventField::from).collect()),
        }
    }
}

/// Statistics returned by a successful `fetchAll` call.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct FetchStats {
    pub functions: usize,
    pub enums: usize,
    pub events: usize,
    /// Number of sources that failed to fetch.
    pub errors: usize,
}

/// Package metadata returned by `version()`.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct VersionInfo {
    pub version: String,
    pub name: String,
    pub authors: String,
}
