//! WASM bindings for ForgeScript parser, metadata, and utilities
//!
//! This module exports all functionality to JavaScript/TypeScript via wasm-bindgen.

#![cfg(feature = "wasm")]

use crate::metadata::{MetadataManager, MetadataSource, github_source};
use crate::parser::{ValidationConfig, parse as rust_parse};
use crate::types::Function;
use crate::utils::{calculate_stats, extract_function_names, format_ast};
use crate::visitor::{AstVisitor, FunctionCollector, NodeCounter};
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

// ============================================================================
// Setup and Initialization
// ============================================================================

#[wasm_bindgen(start)]
pub fn init() {
    // Set panic hook for better error messages
    #[cfg(feature = "panic_hook")]
    console_error_panic_hook::set_once();
}

// ============================================================================
// Parser Bindings
// ============================================================================

/// Parse ForgeScript source code (no validation)
#[wasm_bindgen(js_name = "parse")]
pub fn parse_wasm(source: &str) -> JsValue {
    let (ast, errors) = rust_parse(source);

    let errors_json: Vec<serde_json::Value> = errors
        .into_iter()
        .map(|e| {
            serde_json::json!({
                "message": e.message,
                "span": { "start": e.span.start, "end": e.span.end },
                "kind": format!("{:?}", e.kind),
            })
        })
        .collect();

    let result = serde_json::json!({
        "ast": format_ast(&ast),
        "errors": errors_json,
    });

    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

/// Parse and return an error if there are any parse errors, otherwise return the AST string
#[wasm_bindgen(js_name = "parseOrError")]
pub fn parse_or_error_wasm(source: &str) -> JsValue {
    match crate::parser::parse_with_errors(source) {
        Ok(ast) => {
            let result = serde_json::json!({
                "ok": true,
                "ast": format_ast(&ast),
            });
            serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
        }
        Err(errors) => {
            let errors_json: Vec<serde_json::Value> = errors
                .into_iter()
                .map(|e| {
                    serde_json::json!({
                        "message": e.message,
                        "span": { "start": e.span.start, "end": e.span.end },
                        "kind": format!("{:?}", e.kind),
                    })
                })
                .collect();
            let result = serde_json::json!({
                "ok": false,
                "errors": errors_json,
            });
            serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
        }
    }
}

/// Parse with a specific validation config object
///
/// `config` should be a JS object with boolean fields:
/// `validateArguments`, `validateEnums`, `validateFunctions`, `validateBrackets`
#[wasm_bindgen(js_name = "parseWithConfig")]
pub fn parse_with_config_wasm(source: &str, config: JsValue) -> JsValue {
    // Parse config from JS object
    let validate_arguments = js_sys::Reflect::get(&config, &JsValue::from_str("validateArguments"))
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let validate_enums = js_sys::Reflect::get(&config, &JsValue::from_str("validateEnums"))
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let validate_functions = js_sys::Reflect::get(&config, &JsValue::from_str("validateFunctions"))
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let validate_brackets = js_sys::Reflect::get(&config, &JsValue::from_str("validateBrackets"))
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let cfg = ValidationConfig {
        validate_arguments,
        validate_enums,
        validate_functions,
        validate_brackets,
    };

    let (ast, errors) = crate::parser::parse_with_config(source, cfg);

    let errors_json: Vec<serde_json::Value> = errors
        .into_iter()
        .map(|e| {
            serde_json::json!({
                "message": e.message,
                "span": { "start": e.span.start, "end": e.span.end },
                "kind": format!("{:?}", e.kind),
            })
        })
        .collect();

    let result = serde_json::json!({
        "ast": format_ast(&ast),
        "errors": errors_json,
    });

    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

/// Parse with validation (requires metadata)
#[wasm_bindgen(js_name = "parseWithValidation")]
pub fn parse_with_validation_wasm(
    source: &str,
    metadata_wrapper: &MetadataManagerWrapper,
    validate_arguments: bool,
    validate_enums: bool,
    validate_functions: bool,
    validate_brackets: bool,
) -> JsValue {
    let config = ValidationConfig {
        validate_arguments,
        validate_enums,
        validate_functions,
        validate_brackets,
    };

    let (ast, errors) =
        crate::parser::parse_with_validation(source, config, metadata_wrapper.manager.clone());

    let errors_json: Vec<serde_json::Value> = errors
        .into_iter()
        .map(|e| {
            serde_json::json!({
                "message": e.message,
                "span": { "start": e.span.start, "end": e.span.end },
                "kind": format!("{:?}", e.kind),
            })
        })
        .collect();

    let result = serde_json::json!({
        "ast": format_ast(&ast),
        "errors": errors_json,
    });

    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

/// Parse with strict validation (all validations enabled)
#[wasm_bindgen(js_name = "parseStrict")]
pub fn parse_strict_wasm(source: &str, metadata_wrapper: &MetadataManagerWrapper) -> JsValue {
    let (ast, errors) = crate::parser::parse_strict(source, metadata_wrapper.manager.clone());

    let errors_json: Vec<serde_json::Value> = errors
        .into_iter()
        .map(|e| {
            serde_json::json!({
                "message": e.message,
                "span": { "start": e.span.start, "end": e.span.end },
                "kind": format!("{:?}", e.kind),
            })
        })
        .collect();

    let result = serde_json::json!({
        "ast": format_ast(&ast),
        "errors": errors_json,
    });

    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

/// Return a strict ValidationConfig as a JS object
///
/// Returns `{ validateArguments: true, validateEnums: true, validateFunctions: true, validateBrackets: true }`
#[wasm_bindgen(js_name = "validationConfigStrict")]
pub fn validation_config_strict() -> JsValue {
    let cfg = ValidationConfig::strict();
    serde_json::json!({
        "validateArguments": cfg.validate_arguments,
        "validateEnums": cfg.validate_enums,
        "validateFunctions": cfg.validate_functions,
        "validateBrackets": cfg.validate_brackets,
    })
    .pipe(|v| serde_wasm_bindgen::to_value(&v).unwrap_or(JsValue::NULL))
}

/// Return a syntax-only ValidationConfig as a JS object
///
/// Returns `{ validateArguments: false, validateEnums: false, validateFunctions: false, validateBrackets: false }`
#[wasm_bindgen(js_name = "validationConfigSyntaxOnly")]
pub fn validation_config_syntax_only() -> JsValue {
    let cfg = ValidationConfig::syntax_only();
    serde_json::json!({
        "validateArguments": cfg.validate_arguments,
        "validateEnums": cfg.validate_enums,
        "validateFunctions": cfg.validate_functions,
        "validateBrackets": cfg.validate_brackets,
    })
    .pipe(|v| serde_wasm_bindgen::to_value(&v).unwrap_or(JsValue::NULL))
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
    /// Create a new metadata manager
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            manager: Arc::new(MetadataManager::new()),
        }
    }

    /// Add a GitHub source
    #[wasm_bindgen(js_name = "addGithubSource")]
    pub fn add_github_source(&self, extension: &str, repo: &str, branch: &str) {
        self.manager
            .add_source(github_source(extension, repo, branch));
    }

    /// Add a custom source
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

    /// Fetch all metadata (async)
    #[wasm_bindgen(js_name = "fetchAll")]
    pub fn fetch_all(&self) -> js_sys::Promise {
        let manager = self.manager.clone();

        future_to_promise(async move {
            match manager.fetch_all().await {
                Ok(stats) => {
                    let result = serde_json::json!({
                        "functions": stats.functions,
                        "enums": stats.enums,
                        "events": stats.events,
                        "errors": stats.errors,
                    });

                    Ok(serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL))
                }
                Err(e) => Err(JsValue::from_str(&e.to_string())),
            }
        })
    }

    /// Add custom functions from a JSON string
    ///
    /// The JSON should be an array of Function objects.
    /// Returns the number of functions added.
    #[wasm_bindgen(js_name = "addCustomFunctionsFromJson")]
    pub fn add_custom_functions_from_json(&self, json: &str) -> Result<usize, JsValue> {
        self.manager
            .add_custom_functions_from_json(json)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Remove all custom functions previously added via `addCustomFunctionsFromJson`
    #[wasm_bindgen(js_name = "removeCustomFunctions")]
    pub fn remove_custom_functions(&self) {
        self.manager.remove_custom_functions();
    }

    /// Get function by name (fuzzy / alias-aware)
    #[wasm_bindgen(js_name = "getFunction")]
    pub fn get_function(&self, name: &str) -> Option<String> {
        self.manager
            .get(name)
            .map(|f| serde_json::to_string(&*f).unwrap_or_else(|_| "{}".to_string()))
    }

    /// Get function by exact name
    #[wasm_bindgen(js_name = "getFunctionExact")]
    pub fn get_function_exact(&self, name: &str) -> Option<String> {
        self.manager
            .get_exact(name)
            .map(|f| serde_json::to_string(&*f).unwrap_or_else(|_| "{}".to_string()))
    }

    /// Get the longest registered function name that is a prefix of `text`,
    /// together with the matched key.
    ///
    /// Returns `{ key: string, function: Function } | null`
    #[wasm_bindgen(js_name = "getFunctionPrefix")]
    pub fn get_function_prefix(&self, text: &str) -> JsValue {
        match self.manager.get_prefix(text) {
            Some((key, func)) => {
                let result = serde_json::json!({
                    "key": key,
                    "function": *func,
                });
                serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
            }
            None => JsValue::NULL,
        }
    }

    /// Get a function together with the key that matched (handles aliases).
    ///
    /// Returns `{ key: string, function: Function } | null`
    #[wasm_bindgen(js_name = "getFunctionWithMatch")]
    pub fn get_function_with_match(&self, name: &str) -> JsValue {
        match self.manager.get_with_match(name) {
            Some((key, func)) => {
                let result = serde_json::json!({
                    "key": key,
                    "function": *func,
                });
                serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
            }
            None => JsValue::NULL,
        }
    }

    /// Get multiple functions by name in one call.
    ///
    /// Accepts a JS array of strings; returns a JS array where each element is
    /// either a Function object or `null` if the name was not found.
    #[wasm_bindgen(js_name = "getFunctionMany")]
    pub fn get_function_many(&self, names: JsValue) -> JsValue {
        let names: Vec<String> = match serde_wasm_bindgen::from_value(names) {
            Ok(v) => v,
            Err(_) => return JsValue::NULL,
        };
        let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        let results: Vec<Option<Function>> = self
            .manager
            .get_many(&name_refs)
            .into_iter()
            .map(|opt| opt.map(|f| (*f).clone()))
            .collect();
        serde_wasm_bindgen::to_value(&results).unwrap_or(JsValue::NULL)
    }

    /// Get completions for prefix
    #[wasm_bindgen(js_name = "getCompletions")]
    pub fn get_completions(&self, prefix: &str) -> JsValue {
        let completions: Vec<Function> = self
            .manager
            .get_completions(prefix)
            .into_iter()
            .map(|f| (*f).clone())
            .collect();

        serde_wasm_bindgen::to_value(&completions).unwrap_or(JsValue::NULL)
    }

    /// Get all functions
    #[wasm_bindgen(js_name = "getAllFunctions")]
    pub fn get_all_functions(&self) -> JsValue {
        let functions: Vec<Function> = self
            .manager
            .all_functions()
            .into_iter()
            .map(|f| (*f).clone())
            .collect();

        serde_wasm_bindgen::to_value(&functions).unwrap_or(JsValue::NULL)
    }

    /// Get enum values
    #[wasm_bindgen(js_name = "getEnum")]
    pub fn get_enum(&self, name: &str) -> Option<JsValue> {
        self.manager
            .get_enum(name)
            .map(|values| serde_wasm_bindgen::to_value(&values).unwrap_or(JsValue::NULL))
    }

    /// Get all enums
    #[wasm_bindgen(js_name = "getAllEnums")]
    pub fn get_all_enums(&self) -> JsValue {
        let enums = self.manager.all_enums();
        serde_wasm_bindgen::to_value(&enums).unwrap_or(JsValue::NULL)
    }

    /// Get event by name
    #[wasm_bindgen(js_name = "getEvent")]
    pub fn get_event(&self, name: &str) -> Option<String> {
        self.manager
            .get_event(name)
            .map(|e| serde_json::to_string(&e).unwrap_or_else(|_| "{}".to_string()))
    }

    /// Get all events
    #[wasm_bindgen(js_name = "getAllEvents")]
    pub fn get_all_events(&self) -> JsValue {
        let events = self.manager.all_events();
        serde_wasm_bindgen::to_value(&events).unwrap_or(JsValue::NULL)
    }

    /// Get function count
    #[wasm_bindgen(js_name = "functionCount")]
    pub fn function_count(&self) -> usize {
        self.manager.function_count()
    }

    /// Get enum count
    #[wasm_bindgen(js_name = "enumCount")]
    pub fn enum_count(&self) -> usize {
        self.manager.enum_count()
    }

    /// Get event count
    #[wasm_bindgen(js_name = "eventCount")]
    pub fn event_count(&self) -> usize {
        self.manager.event_count()
    }

    /// Clear all metadata
    #[wasm_bindgen(js_name = "clear")]
    pub fn clear(&self) {
        self.manager.clear();
    }

    /// Export cache to JSON
    #[wasm_bindgen(js_name = "exportCache")]
    pub fn export_cache(&self) -> Result<String, JsValue> {
        self.manager
            .cache_to_json()
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Import cache from JSON
    #[wasm_bindgen(js_name = "importCache")]
    pub fn import_cache(&self, json: &str) -> Result<(), JsValue> {
        self.manager
            .cache_from_json(json)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Save to localStorage
    #[wasm_bindgen(js_name = "saveToLocalStorage")]
    pub fn save_to_local_storage(&self, key: &str) -> Result<(), JsValue> {
        let json = self.export_cache()?;

        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;

        let storage = window
            .local_storage()
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?
            .ok_or_else(|| JsValue::from_str("No localStorage"))?;

        storage
            .set_item(key, &json)
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))
    }

    /// Load from localStorage
    #[wasm_bindgen(js_name = "loadFromLocalStorage")]
    pub fn load_from_local_storage(&self, key: &str) -> Result<(), JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;

        let storage = window
            .local_storage()
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?
            .ok_or_else(|| JsValue::from_str("No localStorage"))?;

        let json = storage
            .get_item(key)
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?
            .ok_or_else(|| JsValue::from_str("No cached data"))?;

        self.import_cache(&json)
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Extract function names from source code
#[wasm_bindgen(js_name = "extractFunctionNames")]
pub fn extract_function_names_wasm(source: &str) -> JsValue {
    let (ast, _) = rust_parse(source);
    let names = extract_function_names(&ast);
    serde_wasm_bindgen::to_value(&names).unwrap_or(JsValue::NULL)
}

/// Calculate AST statistics
#[wasm_bindgen(js_name = "calculateStats")]
pub fn calculate_stats_wasm(source: &str) -> JsValue {
    let (ast, _) = rust_parse(source);
    let stats = calculate_stats(&ast);

    let result = serde_json::json!({
        "totalNodes": stats.total_nodes,
        "textNodes": stats.text_nodes,
        "functionCalls": stats.function_calls,
        "javascriptNodes": stats.javascript_nodes,
        "escapedNodes": stats.escaped_nodes,
        "maxDepth": stats.max_depth,
        "uniqueFunctions": stats.unique_functions,
    });

    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

/// Format AST as human-readable string
#[wasm_bindgen(js_name = "formatAst")]
pub fn format_ast_wasm(source: &str) -> String {
    let (ast, _) = rust_parse(source);
    format_ast(&ast)
}

/// Count total nodes in source
#[wasm_bindgen(js_name = "countNodes")]
pub fn count_nodes_wasm(source: &str) -> usize {
    let (ast, _) = rust_parse(source);
    crate::utils::count_nodes(&ast)
}

/// Check if source contains JavaScript expressions
#[wasm_bindgen(js_name = "containsJavaScript")]
pub fn contains_javascript_wasm(source: &str) -> bool {
    let (ast, _) = rust_parse(source);
    crate::utils::contains_javascript(&ast)
}

/// Get the maximum function-nesting depth in source
#[wasm_bindgen(js_name = "maxNestingDepth")]
pub fn max_nesting_depth_wasm(source: &str) -> usize {
    let (ast, _) = rust_parse(source);
    crate::utils::max_nesting_depth(&ast)
}

/// Extract all text nodes from source.
///
/// Returns an array of `{ text: string, span: { start: number, end: number } }`.
#[wasm_bindgen(js_name = "extractTextNodes")]
pub fn extract_text_nodes_wasm(source: &str) -> JsValue {
    let (ast, _) = rust_parse(source);
    let nodes = crate::utils::extract_text_nodes(&ast);
    let result: Vec<serde_json::Value> = nodes
        .into_iter()
        .map(|(text, span)| {
            serde_json::json!({
                "text": text,
                "span": { "start": span.start, "end": span.end },
            })
        })
        .collect();
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

/// Flatten the AST into a depth-first linear list of node descriptors.
///
/// Returns an array of objects, each with a `type` field and relevant fields
/// for that node type.
#[wasm_bindgen(js_name = "flattenAst")]
pub fn flatten_ast_wasm(source: &str) -> JsValue {
    use crate::parser::AstNode;

    let (ast, _) = rust_parse(source);
    let flat = crate::utils::flatten_ast(&ast);

    let result: Vec<serde_json::Value> = flat
        .iter()
        .map(|node| match node {
            AstNode::Program { span, .. } => serde_json::json!({
                "type": "Program",
                "span": { "start": span.start, "end": span.end },
            }),
            AstNode::Text { content, span } => serde_json::json!({
                "type": "Text",
                "content": content,
                "span": { "start": span.start, "end": span.end },
            }),
            AstNode::FunctionCall {
                name,
                modifiers,
                span,
                ..
            } => serde_json::json!({
                "type": "FunctionCall",
                "name": name,
                "modifiers": {
                    "silent": modifiers.silent,
                    "negated": modifiers.negated,
                    "count": modifiers.count,
                },
                "span": { "start": span.start, "end": span.end },
            }),
            AstNode::JavaScript { code, span } => serde_json::json!({
                "type": "JavaScript",
                "code": code,
                "span": { "start": span.start, "end": span.end },
            }),
            AstNode::Escaped { content, span } => serde_json::json!({
                "type": "Escaped",
                "content": content,
                "span": { "start": span.start, "end": span.end },
            }),
        })
        .collect();

    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

/// Return the source-code slice for a given byte span.
#[wasm_bindgen(js_name = "getSourceSlice")]
pub fn get_source_slice_wasm(source: &str, start: usize, end: usize) -> String {
    let span = crate::parser::Span { start, end };
    crate::utils::get_source_slice(source, span).to_string()
}

/// Check whether the character at `byte_idx` in `source` is escaped
/// (i.e. preceded by an odd number of backslashes).
#[wasm_bindgen(js_name = "isEscaped")]
pub fn is_escaped_wasm(source: &str, byte_idx: usize) -> bool {
    crate::parser::is_escaped(source, byte_idx)
}

// ============================================================================
// Visitor Pattern Helpers
// ============================================================================

/// Collect all function names using the visitor pattern
#[wasm_bindgen(js_name = "collectFunctions")]
pub fn collect_functions_wasm(source: &str) -> JsValue {
    let (ast, _) = rust_parse(source);
    let mut collector = FunctionCollector::new();
    collector.visit(&ast);
    serde_wasm_bindgen::to_value(&collector.functions).unwrap_or(JsValue::NULL)
}

/// Count node types using visitor
#[wasm_bindgen(js_name = "countNodeTypes")]
pub fn count_node_types_wasm(source: &str) -> JsValue {
    let (ast, _) = rust_parse(source);
    let mut counter = NodeCounter::default();
    counter.visit(&ast);

    let result = serde_json::json!({
        "textNodes": counter.text_nodes,
        "functionNodes": counter.function_nodes,
        "javascriptNodes": counter.javascript_nodes,
        "escapedNodes": counter.escaped_nodes,
    });

    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

// ============================================================================
// Validation Helpers
// ============================================================================

/// Validate code and return detailed results
#[wasm_bindgen(js_name = "validateCode")]
pub fn validate_code_wasm(source: &str, metadata_wrapper: &MetadataManagerWrapper) -> JsValue {
    let (_, errors) = crate::parser::parse_strict(source, metadata_wrapper.manager.clone());

    // Group errors by kind
    let mut by_kind = std::collections::HashMap::new();
    for error in &errors {
        let kind = format!("{:?}", error.kind);
        by_kind
            .entry(kind)
            .or_insert_with(Vec::new)
            .push(serde_json::json!({
                "message": error.message,
                "span": { "start": error.span.start, "end": error.span.end },
            }));
    }

    let result = serde_json::json!({
        "valid": errors.is_empty(),
        "errorCount": errors.len(),
        "errorsByKind": by_kind,
        "allErrors": errors.iter().map(|e| {
            serde_json::json!({
                "message": e.message,
                "span": { "start": e.span.start, "end": e.span.end },
                "kind": format!("{:?}", e.kind),
            })
        }).collect::<Vec<_>>(),
    });

    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

// ============================================================================
// Batch Operations
// ============================================================================

/// Parse multiple sources at once
#[wasm_bindgen(js_name = "parseBatch")]
pub fn parse_batch_wasm(sources: JsValue) -> JsValue {
    let sources: Vec<String> = match serde_wasm_bindgen::from_value(sources) {
        Ok(s) => s,
        Err(_) => return JsValue::NULL,
    };

    let results: Vec<_> = sources
        .iter()
        .map(|source| {
            let (ast, errors) = rust_parse(source);
            serde_json::json!({
                "ast": format_ast(&ast),
                "errors": errors.iter().map(|e| {
                    serde_json::json!({
                        "message": e.message,
                        "span": { "start": e.span.start, "end": e.span.end },
                        "kind": format!("{:?}", e.kind),
                    })
                }).collect::<Vec<_>>(),
            })
        })
        .collect();

    serde_wasm_bindgen::to_value(&results).unwrap_or(JsValue::NULL)
}

/// Validate multiple sources at once
#[wasm_bindgen(js_name = "validateBatch")]
pub fn validate_batch_wasm(sources: JsValue, metadata_wrapper: &MetadataManagerWrapper) -> JsValue {
    let sources: Vec<String> = match serde_wasm_bindgen::from_value(sources) {
        Ok(s) => s,
        Err(_) => return JsValue::NULL,
    };

    let results: Vec<_> = sources
        .iter()
        .map(|source| {
            let (_, errors) = crate::parser::parse_strict(source, metadata_wrapper.manager.clone());

            serde_json::json!({
                "valid": errors.is_empty(),
                "errorCount": errors.len(),
                "errors": errors.iter().map(|e| {
                    serde_json::json!({
                        "message": e.message,
                        "span": { "start": e.span.start, "end": e.span.end },
                        "kind": format!("{:?}", e.kind),
                    })
                }).collect::<Vec<_>>(),
            })
        })
        .collect();

    serde_wasm_bindgen::to_value(&results).unwrap_or(JsValue::NULL)
}

// ============================================================================
// Version Info
// ============================================================================

/// Get version information
#[wasm_bindgen(js_name = "version")]
pub fn version() -> JsValue {
    let info = serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "name": env!("CARGO_PKG_NAME"),
        "authors": env!("CARGO_PKG_AUTHORS"),
    });

    serde_wasm_bindgen::to_value(&info).unwrap_or(JsValue::NULL)
}

// ============================================================================
// Internal helpers
// ============================================================================

trait Pipe: Sized {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(Self) -> R,
    {
        f(self)
    }
}

impl<T> Pipe for T {}
