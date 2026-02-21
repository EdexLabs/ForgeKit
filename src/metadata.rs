//! High-performance metadata manager for ForgeScript functions, enums, and events.
//!
//! This module provides:
//! - Fast function lookup using a prefix trie
//! - WASM-compatible by default (no filesystem dependencies)
//! - Optional caching support for native platforms
//! - Robust error handling with no panics
//! - Concurrent access with DashMap

use crate::types::{Event, Function};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

// ============================================================================
// Core Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventField {
    pub name: String,
    #[serde(default)]
    pub description: String,
}

/// Source configuration for fetching metadata
#[derive(Debug, Clone)]
pub struct MetadataSource {
    pub extension: String,
    pub functions_url: Option<String>,
    pub enums_url: Option<String>,
    pub events_url: Option<String>,
}

impl MetadataSource {
    /// Create a new metadata source
    pub fn new(extension: impl Into<String>) -> Self {
        Self {
            extension: extension.into(),
            functions_url: None,
            enums_url: None,
            events_url: None,
        }
    }

    /// Set functions URL
    pub fn with_functions(mut self, url: impl Into<String>) -> Self {
        self.functions_url = Some(url.into());
        self
    }

    /// Set enums URL
    pub fn with_enums(mut self, url: impl Into<String>) -> Self {
        self.enums_url = Some(url.into());
        self
    }

    /// Set events URL
    pub fn with_events(mut self, url: impl Into<String>) -> Self {
        self.events_url = Some(url.into());
        self
    }
}

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, Clone)]
pub enum MetadataError {
    NetworkError(String),
    ParseError(String),
    NotFound(String),
    InvalidData(String),
    CacheError(String),
}

impl std::fmt::Display for MetadataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NetworkError(e) => write!(f, "Network error: {}", e),
            Self::ParseError(e) => write!(f, "Parse error: {}", e),
            Self::NotFound(e) => write!(f, "Not found: {}", e),
            Self::InvalidData(e) => write!(f, "Invalid data: {}", e),
            Self::CacheError(e) => write!(f, "Cache error: {}", e),
        }
    }
}

impl std::error::Error for MetadataError {}

pub type Result<T> = std::result::Result<T, MetadataError>;

// ============================================================================
// Fast Trie for Function Lookup
// ============================================================================

#[derive(Default)]
struct TrieNode {
    children: HashMap<char, Box<TrieNode>>,
    value: Option<Arc<Function>>,
}

/// High-performance prefix trie for function lookup
#[derive(Default)]
pub struct FunctionTrie {
    root: TrieNode,
    count: usize,
}

impl FunctionTrie {
    /// Create a new empty trie
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a function into the trie
    pub fn insert(&mut self, key: &str, func: Arc<Function>) {
        let mut node = &mut self.root;

        for ch in key.to_lowercase().chars() {
            node = node
                .children
                .entry(ch)
                .or_insert_with(|| Box::new(TrieNode::default()));
        }

        if node.value.is_none() {
            self.count += 1;
        }
        node.value = Some(func);
    }

    /// Get exact match (case-insensitive)
    pub fn get_exact(&self, key: &str) -> Option<Arc<Function>> {
        let mut node = &self.root;

        for ch in key.to_lowercase().chars() {
            match node.children.get(&ch) {
                Some(next) => node = next,
                None => return None,
            }
        }

        node.value.clone()
    }

    /// Get the longest registered function name that is a prefix of `text`,
    /// matching strictly from the start of `text`.
    ///
    /// For example, if `$ping` is registered:
    ///   - `get_prefix("$pingmsoko")`     → Some(("$ping", …))
    ///   - `get_prefix("$pingsmmonwind")` → Some(("$ping", …))
    ///   - `get_prefix("$send")`          → None  (no registered prefix)
    ///
    /// The search always starts at position 0 of `text`; it will never match
    /// a function name found only in the middle of the string.
    pub fn get_prefix(&self, text: &str) -> Option<(String, Arc<Function>)> {
        let mut node = &self.root;
        let mut last_match: Option<(String, Arc<Function>)> = None;
        let mut matched = String::with_capacity(text.len());

        for ch in text.to_lowercase().chars() {
            match node.children.get(&ch) {
                Some(next) => {
                    matched.push(ch);
                    node = next;
                    if let Some(func) = &node.value {
                        last_match = Some((matched.clone(), func.clone()));
                    }
                }
                None => break,
            }
        }

        last_match
    }

    /// Get all functions with a given prefix
    pub fn get_completions(&self, prefix: &str) -> Vec<Arc<Function>> {
        let mut node = &self.root;

        for ch in prefix.to_lowercase().chars() {
            match node.children.get(&ch) {
                Some(next) => node = next,
                None => return Vec::new(),
            }
        }

        let mut results = Vec::new();
        self.collect_all(node, &mut results);
        results
    }

    fn collect_all(&self, node: &TrieNode, results: &mut Vec<Arc<Function>>) {
        if let Some(func) = &node.value {
            results.push(func.clone());
        }

        for child in node.children.values() {
            self.collect_all(child, results);
        }
    }

    /// Get all functions in the trie
    pub fn all_functions(&self) -> Vec<Arc<Function>> {
        let mut results = Vec::with_capacity(self.count);
        self.collect_all(&self.root, &mut results);
        results
    }

    /// Number of functions in trie
    #[inline]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Check if trie is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Clear all functions
    pub fn clear(&mut self) {
        self.root = TrieNode::default();
        self.count = 0;
    }
}

// ============================================================================
// HTTP Fetcher
// ============================================================================

/// HTTP fetcher for metadata
pub struct Fetcher {
    client: reqwest::Client,
}

impl Fetcher {
    /// Create a new fetcher
    pub fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        #[cfg(target_arch = "wasm32")]
        let client = reqwest::Client::builder()
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self { client }
    }

    /// Fetch JSON from a URL with proper error handling
    pub async fn fetch_json<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T> {
        let response =
            self.client.get(url).send().await.map_err(|e| {
                MetadataError::NetworkError(format!("Failed to fetch {}: {}", url, e))
            })?;

        let status = response.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(MetadataError::NotFound(format!("URL not found: {}", url)));
        }
        if !status.is_success() {
            return Err(MetadataError::NetworkError(format!(
                "HTTP {}: {}",
                status, url
            )));
        }

        let text = response.text().await.map_err(|e| {
            MetadataError::NetworkError(format!("Failed to read response from {}: {}", url, e))
        })?;

        serde_json::from_str(&text).map_err(|e| {
            let preview: String = text.chars().take(200).collect();
            MetadataError::ParseError(format!(
                "Failed to parse JSON from {}: {}\nJSON preview: {}…",
                url, e, preview
            ))
        })
    }

    /// Fetch functions from URL, parsing each item individually so one bad entry
    /// doesn't block the rest.
    pub async fn fetch_functions(&self, url: &str, extension: String) -> Result<Vec<Function>> {
        let response =
            self.client.get(url).send().await.map_err(|e| {
                MetadataError::NetworkError(format!("Failed to fetch {}: {}", url, e))
            })?;

        let status = response.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(MetadataError::NotFound(format!("URL not found: {}", url)));
        }
        if !status.is_success() {
            return Err(MetadataError::NetworkError(format!(
                "HTTP {}: {}",
                status, url
            )));
        }

        let text = response.text().await.map_err(|e| {
            MetadataError::NetworkError(format!("Failed to read response from {}: {}", url, e))
        })?;

        let raw_items: Vec<serde_json::Value> = serde_json::from_str(&text).map_err(|e| {
            let preview: String = text.chars().take(200).collect();
            MetadataError::ParseError(format!(
                "Failed to parse JSON array from {}: {}\nJSON preview: {}…",
                url, e, preview
            ))
        })?;

        let mut functions = Vec::with_capacity(raw_items.len());
        for (i, raw) in raw_items.into_iter().enumerate() {
            match serde_json::from_value::<Function>(raw) {
                Ok(mut func) => {
                    func.extension = Some(extension.clone());
                    func.source_url = Some(url.to_string());
                    functions.push(func);
                }
                Err(e) => {
                    eprintln!("[forge-kit] Skipping function #{} from {}: {}", i, url, e);
                }
            }
        }

        Ok(functions)
    }

    /// Fetch enums from URL
    pub async fn fetch_enums(&self, url: &str) -> Result<HashMap<String, Vec<String>>> {
        self.fetch_json(url).await
    }

    /// Fetch events from URL
    pub async fn fetch_events(&self, url: &str) -> Result<Vec<Event>> {
        self.fetch_json(url).await
    }
}

impl Default for Fetcher {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Metadata Manager
// ============================================================================

/// High-performance metadata manager
pub struct MetadataManager {
    trie: std::sync::RwLock<FunctionTrie>,
    enums: DashMap<String, Vec<String>>,
    events: DashMap<String, Event>,
    sources: std::sync::RwLock<Vec<MetadataSource>>,
    fetcher: Fetcher,
}

impl MetadataManager {
    /// Create a new metadata manager
    pub fn new() -> Self {
        Self {
            trie: std::sync::RwLock::new(FunctionTrie::new()),
            enums: DashMap::new(),
            events: DashMap::new(),
            sources: std::sync::RwLock::new(Vec::new()),
            fetcher: Fetcher::new(),
        }
    }

    /// Add a metadata source
    pub fn add_source(&self, source: MetadataSource) {
        self.sources.write().unwrap().push(source);
    }

    /// Fetch all metadata from configured sources
    pub async fn fetch_all(&self) -> Result<FetchStats> {
        let sources = self.sources.read().unwrap().clone();

        let mut total_functions = 0;
        let mut total_enums = 0;
        let mut total_events = 0;
        let mut errors = Vec::new();

        for source in sources {
            if let Some(url) = &source.functions_url {
                match self
                    .fetcher
                    .fetch_functions(url, source.extension.clone())
                    .await
                {
                    Ok(functions) => {
                        total_functions += functions.len();
                        self.add_functions(functions);
                    }
                    Err(MetadataError::NotFound(_)) => {}
                    Err(e) => {
                        errors.push(format!("Functions from {}: {}", source.extension, e));
                    }
                }
            }

            if let Some(url) = &source.enums_url {
                match self.fetcher.fetch_enums(url).await {
                    Ok(enums) => {
                        total_enums += enums.len();
                        for (name, values) in enums {
                            self.enums.insert(name, values);
                        }
                    }
                    Err(e) => {
                        if !matches!(e, MetadataError::NotFound(_)) {
                            errors.push(format!("Enums from {}: {}", source.extension, e));
                        }
                    }
                }
            }

            if let Some(url) = &source.events_url {
                match self.fetcher.fetch_events(url).await {
                    Ok(events) => {
                        total_events += events.len();
                        for event in events {
                            self.events.insert(event.name.clone(), event);
                        }
                    }
                    Err(e) => {
                        if !matches!(e, MetadataError::NotFound(_)) {
                            errors.push(format!("Events from {}: {}", source.extension, e));
                        }
                    }
                }
            }
        }

        Ok(FetchStats {
            functions: total_functions,
            enums: total_enums,
            events: total_events,
            errors,
        })
    }

    fn add_functions(&self, functions: Vec<Function>) {
        let mut trie = self.trie.write().unwrap();

        for func in functions {
            let arc_func = Arc::new(func.clone());
            trie.insert(&func.name, arc_func.clone());

            if let Some(aliases) = &func.aliases {
                for alias in aliases {
                    let alias_name = if alias.starts_with('$') {
                        alias.clone()
                    } else {
                        format!("${}", alias)
                    };
                    let mut alias_func = (*arc_func).clone();
                    alias_func.name = alias_name.clone();
                    trie.insert(&alias_name, Arc::new(alias_func));
                }
            }
        }
    }

    // ========================================================================
    // Custom Functions: ingest from JSON
    // ========================================================================

    /// Register custom functions from a JSON string.
    ///
    /// The JSON must be an array of `Function` objects — exactly the format that
    /// [`generate_custom_functions_json`] produces.  This is the fast startup
    /// path: generate the file once (build step / CLI), commit it, then load it
    /// here at LSP startup with no JS/TS source parsing at runtime.
    ///
    /// ```json
    /// [
    ///   {
    ///     "name": "$myFunc",
    ///     "version": "1.0.0",
    ///     "description": "Does something useful",
    ///     "brackets": true,
    ///     "unwrap": false,
    ///     "args": [
    ///       { "name": "value", "type": "String", "required": true, "rest": false }
    ///     ]
    ///   }
    /// ]
    /// ```
    ///
    /// Returns the number of successfully registered functions (invalid entries
    /// are skipped and logged to stderr).
    pub fn add_custom_functions_from_json(&self, json: &str) -> Result<usize> {
        let raw_items: Vec<serde_json::Value> = serde_json::from_str(json).map_err(|e| {
            MetadataError::ParseError(format!("Invalid custom-functions JSON: {}", e))
        })?;

        let mut count = 0;
        let mut trie = self.trie.write().unwrap();

        for (i, raw) in raw_items.into_iter().enumerate() {
            match serde_json::from_value::<Function>(raw) {
                Ok(mut func) => {
                    // Guarantee $ prefix
                    if !func.name.starts_with('$') {
                        func.name = format!("${}", func.name);
                    }
                    func.category = func.category.or(Some("custom".to_string()));

                    let arc_func = Arc::new(func.clone());
                    trie.insert(&func.name, arc_func.clone());
                    count += 1;

                    // Register aliases
                    if let Some(aliases) = &func.aliases {
                        for alias in aliases {
                            let alias_name = if alias.starts_with('$') {
                                alias.clone()
                            } else {
                                format!("${}", alias)
                            };
                            let mut alias_func = (*arc_func).clone();
                            alias_func.name = alias_name.clone();
                            trie.insert(&alias_name, Arc::new(alias_func));
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[forge-kit] Skipping custom function #{}: {}", i, e);
                }
            }
        }

        Ok(count)
    }

    /// Load custom-functions JSON from a file on disk and register every entry.
    ///
    /// The file must be an array of `Function` objects — the format produced by
    /// [`generate_custom_functions_json_to_file`].
    #[cfg(not(target_arch = "wasm32"))]
    pub fn add_custom_functions_from_json_file(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<usize> {
        let path = path.as_ref();
        let json = std::fs::read_to_string(path).map_err(|e| {
            MetadataError::CacheError(format!(
                "Cannot read custom-functions file {}: {}",
                path.display(),
                e
            ))
        })?;
        self.add_custom_functions_from_json(&json)
    }

    // ========================================================================
    // Custom Functions: generate JSON from JS/TS source files
    // ========================================================================

    /// Scan `folder` (and all sub-folders) for `*.js` / `*.ts` files, extract
    /// every custom function found via regex-based heuristics, and return a
    /// pretty-printed JSON string of `Function` objects.
    ///
    /// **Intended as a one-time build / CLI step.**  Save the result to a file
    /// and load it at LSP startup with [`add_custom_functions_from_json_file`]
    /// — no source parsing is needed at runtime.
    ///
    /// The output is directly consumable by [`add_custom_functions_from_json`].
    #[cfg(not(target_arch = "wasm32"))]
    pub fn generate_custom_functions_json(
        &self,
        folder: impl AsRef<std::path::Path>,
    ) -> Result<String> {
        let folder = folder.as_ref();
        if !folder.exists() || !folder.is_dir() {
            return Err(MetadataError::InvalidData(format!(
                "generate_custom_functions_json: {} is not a directory",
                folder.display()
            )));
        }

        let mut functions: Vec<Function> = Vec::new();
        collect_functions_from_folder(folder, &mut functions)?;

        serde_json::to_string_pretty(&functions).map_err(|e| {
            MetadataError::ParseError(format!("Failed to serialize custom functions: {}", e))
        })
    }

    /// Like [`generate_custom_functions_json`] but writes the output directly to
    /// `output_path`, creating parent directories as needed.
    ///
    /// Returns the number of functions written.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn generate_custom_functions_json_to_file(
        &self,
        folder: impl AsRef<std::path::Path>,
        output_path: impl AsRef<std::path::Path>,
    ) -> Result<usize> {
        let json = self.generate_custom_functions_json(folder)?;

        // Count without a second scan
        let entries: Vec<serde_json::Value> =
            serde_json::from_str(&json).map_err(|e| MetadataError::ParseError(e.to_string()))?;
        let count = entries.len();

        let output_path = output_path.as_ref();
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                MetadataError::CacheError(format!("Cannot create directories: {}", e))
            })?;
        }
        std::fs::write(output_path, json).map_err(|e| {
            MetadataError::CacheError(format!("Cannot write to {}: {}", output_path.display(), e))
        })?;

        Ok(count)
    }

    // ========================================================================
    // Standard lookups
    // ========================================================================

    /// Get function by exact name (case-insensitive)
    #[inline]
    pub fn get_exact(&self, name: &str) -> Option<Arc<Function>> {
        self.trie.read().unwrap().get_exact(name)
    }

    /// Get the longest registered function name that is a prefix of `text`,
    /// matching strictly from the start of `text`.
    #[inline]
    pub fn get_prefix(&self, text: &str) -> Option<(String, Arc<Function>)> {
        self.trie.read().unwrap().get_prefix(text)
    }

    /// Get function: tries exact match first, then prefix match from the start.
    ///
    /// Use `get_exact` when you need strict lookup (e.g. bracketed calls).
    pub fn get(&self, name: &str) -> Option<Arc<Function>> {
        let trie = self.trie.read().unwrap();
        if let Some(func) = trie.get_exact(name) {
            return Some(func);
        }
        trie.get_prefix(name).map(|(_, func)| func)
    }

    /// Get function with match info (matched key + Arc)
    pub fn get_with_match(&self, name: &str) -> Option<(String, Arc<Function>)> {
        let trie = self.trie.read().unwrap();
        if let Some(func) = trie.get_exact(name) {
            return Some((name.to_string(), func));
        }
        trie.get_prefix(name)
    }

    /// Get multiple functions
    pub fn get_many(&self, names: &[&str]) -> Vec<Option<Arc<Function>>> {
        names.iter().map(|name| self.get(name)).collect()
    }

    /// Get completions for a prefix
    #[inline]
    pub fn get_completions(&self, prefix: &str) -> Vec<Arc<Function>> {
        self.trie.read().unwrap().get_completions(prefix)
    }

    /// Get all functions
    #[inline]
    pub fn all_functions(&self) -> Vec<Arc<Function>> {
        self.trie.read().unwrap().all_functions()
    }

    /// Get enum values
    #[inline]
    pub fn get_enum(&self, name: &str) -> Option<Vec<String>> {
        self.enums.get(name).map(|v| v.clone())
    }

    /// Get all enums
    pub fn all_enums(&self) -> HashMap<String, Vec<String>> {
        self.enums
            .iter()
            .map(|e| (e.key().clone(), e.value().clone()))
            .collect()
    }

    /// Get event by name
    #[inline]
    pub fn get_event(&self, name: &str) -> Option<Event> {
        self.events.get(name).map(|v| v.clone())
    }

    /// Get all events
    pub fn all_events(&self) -> Vec<Event> {
        self.events.iter().map(|e| e.value().clone()).collect()
    }

    /// Get function count
    #[inline]
    pub fn function_count(&self) -> usize {
        self.trie.read().unwrap().len()
    }

    /// Get enum count
    #[inline]
    pub fn enum_count(&self) -> usize {
        self.enums.len()
    }

    /// Get event count
    #[inline]
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Clear all metadata
    pub fn clear(&self) {
        self.trie.write().unwrap().clear();
        self.enums.clear();
        self.events.clear();
    }
}

impl Default for MetadataManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// JS/TS source parser  (used only by generate_custom_functions_json)
// ============================================================================

/// Recursively walk `path`, collecting `Function` values from every JS/TS file.
/// No trie registration happens here — output is for serialization only.
#[cfg(not(target_arch = "wasm32"))]
fn collect_functions_from_folder(path: &std::path::Path, out: &mut Vec<Function>) -> Result<()> {
    let entries = std::fs::read_dir(path).map_err(|e| {
        MetadataError::InvalidData(format!("Cannot read dir {}: {}", path.display(), e))
    })?;

    for entry in entries {
        let entry_path = entry
            .map_err(|e| MetadataError::InvalidData(e.to_string()))?
            .path();

        if entry_path.is_dir() {
            collect_functions_from_folder(&entry_path, out)?;
        } else if entry_path.is_file() {
            let is_js_ts = entry_path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e == "js" || e == "ts")
                .unwrap_or(false);

            if is_js_ts {
                let content = std::fs::read_to_string(&entry_path).map_err(|e| {
                    MetadataError::InvalidData(format!(
                        "Cannot read {}: {}",
                        entry_path.display(),
                        e
                    ))
                })?;
                out.extend(parse_functions_from_js_ts(
                    &content,
                    entry_path.to_str().unwrap_or_default(),
                ));
            }
        }
    }

    Ok(())
}

/// Extract `Function` metadata from a single JS/TS source file using regex
/// heuristics.  Mirrors the logic from `parse_custom_functions_from_js` in the
/// older metadata implementation but produces `Function` values directly so they
/// can be round-tripped through JSON without a lossy intermediate type.
#[cfg(not(target_arch = "wasm32"))]
fn parse_functions_from_js_ts(content: &str, file_path: &str) -> Vec<Function> {
    use regex::Regex;
    use serde_json::Value as JsonValue;

    // ── Regexes ──────────────────────────────────────────────────────────────
    let name_re = Regex::new(r#"name:\s*['"]([^'"]+)['"]"#).expect("regex");
    let params_re = Regex::new(r#"(?:params|args):\s*\["#).expect("regex");
    let desc_re = Regex::new(
        r#"(?s)description:\s*(?:'((?:[^'\\]|\\.)*?)'|"((?:[^"\\]|\\.)*?)"|`((?:[^`\\]|\\.)*?)`)"#,
    )
    .expect("regex");
    let brackets_re = Regex::new(r"brackets:\s*(true|false)").expect("regex");
    let p_name_re = Regex::new(r#"name:\s*['"]([^'"]+)['"]"#).expect("regex");
    let required_re = Regex::new(r"(?i)required:\s*(true|false)").expect("regex");
    let rest_re = Regex::new(r"(?i)rest:\s*(true|false)").expect("regex");
    let type_re = Regex::new(r"type:\s*([^,}\n\s]+)").expect("regex");
    let output_re = Regex::new(r"output:\s*([^,}\n\s]+)").expect("regex");

    // ── Collect all name: positions with line numbers ─────────────────────────
    let name_matches: Vec<(usize, usize, String, u32)> = name_re
        .captures_iter(content)
        .map(|c: regex::Captures| {
            let m = c.get(0).unwrap();
            let start = m.start();
            let line = content[..start].chars().filter(|&c| c == '\n').count() as u32;
            (start, m.end(), c[1].to_string(), line)
        })
        .collect();

    // ── Collect params/args array ranges ─────────────────────────────────────
    let mut params_ranges: Vec<std::ops::Range<usize>> = Vec::new();
    for m in params_re.find_iter(content) {
        let start = m.start();
        let mut depth = 0i32;
        for (i, c) in content[start..].char_indices() {
            if c == '[' {
                depth += 1;
            } else if c == ']' {
                depth -= 1;
                if depth == 0 {
                    params_ranges.push(start..start + i);
                    break;
                }
            }
        }
    }

    // ── Filter: keep only top-level function name: declarations ──────────────
    let func_names: Vec<_> = name_matches
        .into_iter()
        .filter(|m| !params_ranges.iter().any(|r| r.contains(&m.0)))
        .collect();

    // ── Build Function values ─────────────────────────────────────────────────
    let mut functions = Vec::new();

    for i in 0..func_names.len() {
        let (_, end_pos, raw_name, line) = &func_names[i];
        let chunk_end = if i + 1 < func_names.len() {
            func_names[i + 1].0
        } else {
            content.len()
        };
        let chunk = &content[*end_pos..chunk_end];

        // Ensure $ prefix
        let name = if raw_name.starts_with('$') {
            raw_name.clone()
        } else {
            format!("${}", raw_name)
        };

        let description = desc_re
            .captures(chunk)
            .and_then(|c: regex::Captures| c.get(1).or(c.get(2)).or(c.get(3)))
            .map(|m: regex::Match| m.as_str().to_string())
            .unwrap_or_else(|| "Custom function".to_string());

        let brackets = brackets_re
            .captures(chunk)
            .map(|c: regex::Captures| &c[1] == "true");

        let output: Option<Vec<String>> = output_re.captures(chunk).map(|c: regex::Captures| {
            c[1].split(',')
                .map(|s: &str| {
                    s.trim()
                        .trim_matches(|c: char| c == '\'' || c == '"')
                        .to_string()
                })
                .filter(|s: &String| !s.is_empty())
                .collect()
        });

        // Parse args from the params block that belongs to this function chunk
        let args: Option<Vec<crate::types::Arg>> = params_ranges
            .iter()
            .find(|r| r.start >= *end_pos && r.start < chunk_end)
            .and_then(|p_range| {
                let p_content = &content[p_range.clone()];
                let mut parsed_args: Vec<crate::types::Arg> = Vec::new();
                let mut search = 0;

                while let Some(bstart) = p_content[search..].find('{') {
                    let abs = search + bstart;
                    let mut depth = 0i32;
                    for (j, c) in p_content[abs..].char_indices() {
                        if c == '{' {
                            depth += 1;
                        } else if c == '}' {
                            depth -= 1;
                            if depth == 0 {
                                let body = &p_content[abs + 1..abs + j];
                                if let Some(n_cap) = p_name_re.captures(body) {
                                    let raw_type = type_re
                                        .captures(body)
                                        .map(|c: regex::Captures| {
                                            let t = c[1]
                                                .trim()
                                                .trim_matches(|c: char| c == '\'' || c == '"');
                                            t.strip_prefix("ArgType.").unwrap_or(t).to_string()
                                        })
                                        .unwrap_or_else(|| "String".to_string());

                                    parsed_args.push(crate::types::Arg {
                                        name: n_cap[1].to_string(),
                                        description: desc_re
                                            .captures(body)
                                            .and_then(|c: regex::Captures| {
                                                c.get(1).or(c.get(2)).or(c.get(3))
                                            })
                                            .map(|m: regex::Match| m.as_str().to_string())
                                            .unwrap_or_default(),
                                        rest: rest_re
                                            .captures(body)
                                            .map(|c: regex::Captures| &c[1] == "true")
                                            .unwrap_or(false),
                                        required: required_re
                                            .captures(body)
                                            .map(|c: regex::Captures| &c[1] == "true"),
                                        arg_type: JsonValue::String(raw_type),
                                        ..Default::default()
                                    });
                                }
                                search = abs + j + 1;
                                break;
                            }
                        }
                    }
                }

                if parsed_args.is_empty() {
                    None
                } else {
                    Some(parsed_args)
                }
            });

        functions.push(Function {
            name,
            version: Some(JsonValue::String("1.0.0".to_string())),
            description,
            brackets: brackets.or(if args.is_some() { Some(true) } else { None }),
            unwrap: false,
            args,
            output,
            category: Some("custom".to_string()),
            local_path: Some(std::path::PathBuf::from(file_path)),
            line: Some(*line),
            ..Default::default()
        });
    }

    functions
}

// ============================================================================
// FetchStats
// ============================================================================

/// Statistics from a fetch operation
#[derive(Debug, Clone)]
pub struct FetchStats {
    pub functions: usize,
    pub enums: usize,
    pub events: usize,
    pub errors: Vec<String>,
}

impl std::fmt::Display for FetchStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Fetched {} functions, {} enums, {} events",
            self.functions, self.enums, self.events
        )?;
        if !self.errors.is_empty() {
            write!(f, " ({} errors)", self.errors.len())?;
        }
        Ok(())
    }
}

// ============================================================================
// Caching Support
// ============================================================================

/// Serializable cache format
#[derive(Debug, Serialize, Deserialize)]
pub struct MetadataCache {
    pub functions: Vec<Function>,
    pub enums: HashMap<String, Vec<String>>,
    pub events: Vec<Event>,
    pub version: u32,
}

impl MetadataCache {
    const VERSION: u32 = 1;

    pub fn new(
        functions: Vec<Function>,
        enums: HashMap<String, Vec<String>>,
        events: Vec<Event>,
    ) -> Self {
        Self {
            functions,
            enums,
            events,
            version: Self::VERSION,
        }
    }
}

impl MetadataManager {
    pub fn export_cache(&self) -> MetadataCache {
        MetadataCache::new(
            self.all_functions().iter().map(|f| (**f).clone()).collect(),
            self.all_enums(),
            self.all_events(),
        )
    }

    pub fn import_cache(&self, cache: MetadataCache) -> Result<()> {
        if cache.version != MetadataCache::VERSION {
            return Err(MetadataError::CacheError(format!(
                "Incompatible cache version: expected {}, got {}",
                MetadataCache::VERSION,
                cache.version
            )));
        }
        self.clear();
        self.add_functions(cache.functions);
        for (name, values) in cache.enums {
            self.enums.insert(name, values);
        }
        for event in cache.events {
            self.events.insert(event.name.clone(), event);
        }
        Ok(())
    }

    pub fn cache_to_json(&self) -> Result<String> {
        serde_json::to_string(&self.export_cache())
            .map_err(|e| MetadataError::CacheError(format!("Serialization failed: {}", e)))
    }

    pub fn cache_from_json(&self, json: &str) -> Result<()> {
        let cache: MetadataCache = serde_json::from_str(json)
            .map_err(|e| MetadataError::CacheError(format!("Deserialization failed: {}", e)))?;
        self.import_cache(cache)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl MetadataManager {
    pub fn save_cache_to_file(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        use std::io::Write;
        let json = self.cache_to_json()?;
        let mut file = std::fs::File::create(path)
            .map_err(|e| MetadataError::CacheError(format!("Failed to create file: {}", e)))?;
        file.write_all(json.as_bytes())
            .map_err(|e| MetadataError::CacheError(format!("Failed to write file: {}", e)))?;
        Ok(())
    }

    pub fn load_cache_from_file(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let json = std::fs::read_to_string(path)
            .map_err(|e| MetadataError::CacheError(format!("Failed to read file: {}", e)))?;
        self.cache_from_json(&json)
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Create a metadata source from a GitHub repository
pub fn github_source(extension: impl Into<String>, repo: &str, branch: &str) -> MetadataSource {
    let base = format!("https://raw.githubusercontent.com/{}/{}/", repo, branch);
    MetadataSource::new(extension)
        .with_functions(format!("{}functions.json", base))
        .with_enums(format!("{}enums.json", base))
        .with_events(format!("{}events.json", base))
}

/// Create a metadata source from custom URLs
pub fn custom_source(extension: impl Into<String>) -> MetadataSource {
    MetadataSource::new(extension)
}
