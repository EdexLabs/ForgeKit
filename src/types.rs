use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Function {
    pub name: String,
    pub version: JsonValue,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brackets: Option<bool>,
    pub unwrap: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<Arg>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<Vec<String>>,
    pub category: Option<String>,
    pub aliases: Option<Vec<String>>,
    pub experimental: Option<bool>,
    pub examples: Option<Vec<String>>,
    pub deprecated: Option<bool>,
    #[serde(skip)]
    pub extension: Option<String>,
    #[serde(skip)]
    pub source_url: Option<String>,
    #[serde(skip)]
    pub local_path: Option<PathBuf>,
    #[serde(skip)]
    pub line: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Arg {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub rest: bool,
    pub required: Option<bool>,
    pub arg_type: JsonValue,
    pub condition: Option<bool>,
    pub arg_enum: Option<Vec<String>>,
    pub enum_name: Option<String>,
    pub pointer: Option<i64>,
    pub pointer_property: Option<String>,
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
