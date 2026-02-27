use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Function {
    pub name: String,
    #[serde(default)]
    pub version: Option<JsonValue>,
    #[serde(default)]
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brackets: Option<bool>,
    #[serde(default)]
    pub unwrap: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<Arg>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<Vec<String>>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub aliases: Option<Vec<String>>,
    #[serde(default)]
    pub experimental: Option<bool>,
    #[serde(default)]
    pub examples: Option<Vec<String>>,
    #[serde(default)]
    pub deprecated: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extension: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_path: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    /// Capture any unrecognized keys so future JSON additions don't break deserialization
    #[serde(flatten)]
    pub extra: HashMap<String, JsonValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Arg {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub rest: bool,
    #[serde(default)]
    pub required: Option<bool>,
    #[serde(rename = "type", default)]
    pub arg_type: JsonValue,
    #[serde(default)]
    pub condition: Option<bool>,
    #[serde(rename = "enum", default)]
    pub arg_enum: Option<Vec<String>>,
    #[serde(default)]
    pub enum_name: Option<String>,
    #[serde(default)]
    pub pointer: Option<i64>,
    #[serde(default)]
    pub pointer_property: Option<String>,
    /// Capture unrecognized keys for forward compatibility
    #[serde(flatten)]
    pub extra: HashMap<String, JsonValue>,
}

/// Event definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub fields: Option<Vec<EventField>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventField {
    pub name: String,
    #[serde(default)]
    pub description: String,
}
