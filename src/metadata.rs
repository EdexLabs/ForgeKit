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

        // Normalize to lowercase for case-insensitive lookup
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
    ///   - `get_prefix("$pingmsoko")`    → Some(("$ping", …))
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
                // No further match possible from this path — stop immediately.
                None => break,
            }
        }

        last_match
    }

    /// Get all functions with a given prefix
    pub fn get_completions(&self, prefix: &str) -> Vec<Arc<Function>> {
        let mut node = &self.root;

        // Navigate to prefix
        for ch in prefix.to_lowercase().chars() {
            match node.children.get(&ch) {
                Some(next) => node = next,
                None => return Vec::new(),
            }
        }

        // Collect all functions under this prefix
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
        // Make request
        let response =
            self.client.get(url).send().await.map_err(|e| {
                MetadataError::NetworkError(format!("Failed to fetch {}: {}", url, e))
            })?;

        // Check status
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

        // Parse JSON
        let text = response.text().await.map_err(|e| {
            MetadataError::NetworkError(format!("Failed to read response from {}: {}", url, e))
        })?;

        serde_json::from_str(&text).map_err(|e| {
            // Include a preview of the raw JSON to help debug which field is malformed
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

        // Parse the outer array as raw values first
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
                    // Log and skip the bad entry — don't abort the whole file
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
            // Fetch functions — don't abort on error; continue to enums/events
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
                    Err(MetadataError::NotFound(_)) => {
                        // 404 is fine — optional
                    }
                    Err(e) => {
                        errors.push(format!("Functions from {}: {}", source.extension, e));
                    }
                }
            }

            // Fetch enums — always continue regardless of functions result
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

            // Fetch events — always continue regardless of functions/enums result
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

    /// Add functions to the manager
    fn add_functions(&self, functions: Vec<Function>) {
        let mut trie = self.trie.write().unwrap();

        for func in functions {
            let arc_func = Arc::new(func.clone());

            // Insert main name
            trie.insert(&func.name, arc_func.clone());

            // Insert aliases
            if let Some(aliases) = &func.aliases {
                for alias in aliases {
                    let alias_name = if alias.starts_with('$') {
                        alias.clone()
                    } else {
                        format!("${}", alias)
                    };

                    // Create alias function
                    let mut alias_func = (*arc_func).clone();
                    alias_func.name = alias_name.clone();
                    trie.insert(&alias_name, Arc::new(alias_func));
                }
            }
        }
    }

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

        // Try exact match first
        if let Some(func) = trie.get_exact(name) {
            return Some(func);
        }

        // Try longest-prefix match from position 0
        trie.get_prefix(name).map(|(_, func)| func)
    }

    /// Get function with match info (for compatibility)
    pub fn get_with_match(&self, name: &str) -> Option<(String, Arc<Function>)> {
        let trie = self.trie.read().unwrap();

        // Try exact match first
        if let Some(func) = trie.get_exact(name) {
            return Some((name.to_string(), func));
        }

        // Try prefix match
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
// Caching Support (Optional)
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

    /// Create a new cache
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
    /// Export metadata to cache
    pub fn export_cache(&self) -> MetadataCache {
        MetadataCache::new(
            self.all_functions().iter().map(|f| (**f).clone()).collect(),
            self.all_enums(),
            self.all_events(),
        )
    }

    /// Import metadata from cache
    pub fn import_cache(&self, cache: MetadataCache) -> Result<()> {
        if cache.version != MetadataCache::VERSION {
            return Err(MetadataError::CacheError(format!(
                "Incompatible cache version: expected {}, got {}",
                MetadataCache::VERSION,
                cache.version
            )));
        }

        // Clear existing data
        self.clear();

        // Add functions
        self.add_functions(cache.functions);

        // Add enums
        for (name, values) in cache.enums {
            self.enums.insert(name, values);
        }

        // Add events
        for event in cache.events {
            self.events.insert(event.name.clone(), event);
        }

        Ok(())
    }

    /// Serialize cache to JSON
    pub fn cache_to_json(&self) -> Result<String> {
        let cache = self.export_cache();
        serde_json::to_string(&cache)
            .map_err(|e| MetadataError::CacheError(format!("Serialization failed: {}", e)))
    }

    /// Deserialize cache from JSON
    pub fn cache_from_json(&self, json: &str) -> Result<()> {
        let cache: MetadataCache = serde_json::from_str(json)
            .map_err(|e| MetadataError::CacheError(format!("Deserialization failed: {}", e)))?;
        self.import_cache(cache)
    }
}

// ============================================================================
// Native Filesystem Caching
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
impl MetadataManager {
    /// Save cache to file
    pub fn save_cache_to_file(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        use std::io::Write;

        let json = self.cache_to_json()?;
        let mut file = std::fs::File::create(path)
            .map_err(|e| MetadataError::CacheError(format!("Failed to create file: {}", e)))?;

        file.write_all(json.as_bytes())
            .map_err(|e| MetadataError::CacheError(format!("Failed to write file: {}", e)))?;

        Ok(())
    }

    /// Load cache from file
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
